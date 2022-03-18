// TODO server and client need two bin

use std::{io::Write, net::TcpStream, time::Duration};

use clap::{ArgEnum, Parser, Subcommand};
use kvs::{
    cat, create_key_value, fetch_token, start_server, KVSResult, KVSSession, KVSToken, Secret,
};
use tracing_subscriber::prelude::*;

#[derive(Parser, Debug, Clone)]
#[clap(author="zmp <zhaoqian.ipp@gmail.com>", version, about="aeolus screen tools", long_about = None)]
struct KVS {
    #[clap(subcommand)]
    command: Commands,

    #[clap(short, long, help = "Use verbose output")]
    verbose: bool,

    #[clap(short, long, help = "Set Repository", default_value = "0.0.0.0:8888")]
    repository: String,
}

#[derive(Debug, Subcommand, Clone)]
enum Commands {
    #[clap(long_about = "start kvs server")]
    Start {
        #[clap(short, long, help = "reset the jwt_secret")]
        reset_jwt_secret: bool,
        #[clap(short, long, help = "start kvs server in bg")]
        detach: bool,
    },
    #[clap(long_about = "login to kvs")]
    Login,
    #[clap(long_about = "create key value")]
    Create { key: String, value: String },

    #[clap(long_about = "cat key content")]
    Cat { key: String },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum URLType {
    Embed,
}

fn main() -> KVSResult<()> {
    let kvs_cli = KVS::parse();

    tracing_subscriber::registry()
        // Filter spans based on the RUST_LOG env var.
        .with(tracing_subscriber::EnvFilter::new(if kvs_cli.verbose {
            "error,kvs=debug"
        } else {
            "error,kvs=info"
        }))
        // Send a copy of all spans to stdout as JSON.
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_level(true)
                .compact(),
        )
        .try_init()
        .unwrap();

    let get_kvs_session = || -> KVSResult<KVSSession> {
        let stream = TcpStream::connect(&kvs_cli.repository)?;
        stream
            .set_read_timeout(Some(Duration::from_millis(1000)))
            .expect("fail");

        KVSSession::new(stream)
    };

    let fetch_token = || -> KVSResult<KVSToken> {
        let kvs_dir_path = std::path::Path::new(".kvs");
        let secret_file_path = kvs_dir_path.join("secret");
        let secret = if secret_file_path.exists() {
            Secret::from(std::fs::read_to_string(secret_file_path).unwrap())
        } else {
            let secret = Secret::default();
            std::fs::create_dir_all(kvs_dir_path).unwrap();
            let mut file = std::fs::File::create(secret_file_path).unwrap();
            file.write_all(&secret.to_string().as_bytes()).unwrap();
            secret
        };
        let session = get_kvs_session()?;
        fetch_token(session, secret)
    };

    let get_token = || -> KVSResult<KVSToken> {
        let kvs_dir_path = std::path::Path::new(".kvs");
        let token_file_path = kvs_dir_path.join("token");
        if token_file_path.exists() {
            Ok(bincode::deserialize(
                &std::fs::read(token_file_path).unwrap(),
            )?)
        } else {
            fetch_token()
        }
    };

    match &kvs_cli.command {
        Commands::Start {
            reset_jwt_secret,
            detach,
        } => {
            let user_kvs_config_dir_path = std::path::Path::new(".kvs");

            if *detach {
                let args = std::env::args().map(|x| x).collect::<Vec<String>>();
                let detach_command_args = args[1..]
                    .to_vec()
                    .into_iter()
                    .filter(|arg| *arg != "-d" && *arg != "--detach")
                    .collect::<Vec<String>>();
                let detach_commands_str = detach_command_args.join(" ");
                println!("{}", detach_commands_str);

                let stdout_file = if let Ok(stdout_file) = std::fs::File::open("kvs.log") {
                    stdout_file
                } else {
                    std::fs::File::create("kvs.log")?
                };

                let stderr_file = if let Ok(stderr_file) = std::fs::File::open("kvs.errors.log") {
                    stderr_file
                } else {
                    std::fs::File::create("kvs.errors.log")?
                };

                let child =
                    std::process::Command::new(std::env::current_exe()?.display().to_string())
                        .args(detach_command_args)
                        .stdout(stdout_file)
                        .stderr(stderr_file)
                        .spawn()
                        .expect("kvs detach process failed to start.");
                // recored child pid

                let kvs_pid_file_path = user_kvs_config_dir_path.join("pid");
                std::fs::write(kvs_pid_file_path, child.id().to_string())?;
                tracing::info!("kvs started PID: {}", child.id());
                tracing::info!("The logs saved to ./kvs.log and ./kvs.errors.log");

                return Ok(());
            }

            if !user_kvs_config_dir_path.exists() {
                std::fs::create_dir_all(user_kvs_config_dir_path)?;
            }
            let jwt_secret_file_path = user_kvs_config_dir_path.join("jwt_secret");

            let jwt_secret = if jwt_secret_file_path.exists() && *reset_jwt_secret == false {
                std::fs::read(jwt_secret_file_path)?
            } else {
                let jwt_secret = (0..256).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
                std::fs::write(jwt_secret_file_path, &jwt_secret)?;
                jwt_secret
            };

            start_server(&kvs_cli.repository, &jwt_secret);
        }

        Commands::Login => {
            let user_kvs_config_dir_path = std::path::Path::new(".kvs");
            if !user_kvs_config_dir_path.exists() {
                std::fs::create_dir_all(user_kvs_config_dir_path)?;
            }
            let path = user_kvs_config_dir_path.join("token");
            let token = fetch_token()?;
            let token_bytes = bincode::serialize(&token)?;
            std::fs::File::create(&path)?.write_all(&token_bytes)?;

            tracing::info!("Token: {}", base64::encode(&token_bytes));
            tracing::info!("Save Token file to: {}", path.display());
            // TODO token str;
        }
        Commands::Create { key, value } => {
            let token = check_and_auto_fetch_token_to_disk(&kvs_cli)?;
            let session = get_kvs_session()?;
            create_key_value(session, &token, key, value).unwrap();
        }
        Commands::Cat { key } => {
            let token = get_token()?;
            let session = get_kvs_session()?;
            let content = cat(session, &token, key)?;
            let content_str = String::from_utf8(content).unwrap();
            println!("{}", content_str);
        }
    };

    Ok(())
}

fn check_and_auto_fetch_token_to_disk(kvs_cli: &KVS) -> KVSResult<KVSToken> {
    // paths
    let user_kvs_config_dir_path = std::path::Path::new(".kvs");
    let user_secret_file_path = user_kvs_config_dir_path.join("secret");
    let user_token_file_path = user_kvs_config_dir_path.join("token");

    if user_token_file_path.exists() {
        Ok(bincode::deserialize(&std::fs::read(user_token_file_path).unwrap()).unwrap())
    } else {
        let secret = if user_secret_file_path.exists() {
            Secret::from(std::fs::read_to_string(user_secret_file_path).unwrap())
        } else {
            let secret = Secret::default();
            std::fs::create_dir_all(user_kvs_config_dir_path)?;
            let mut file = std::fs::File::create(user_secret_file_path).unwrap();
            file.write_all(&secret.to_string().as_bytes()).unwrap();
            secret
        };
        let session = {
            let stream = TcpStream::connect(&kvs_cli.repository)?;
            stream.set_read_timeout(Some(Duration::from_millis(1000)))?;
            KVSSession::new(stream)
        }?;
        fetch_token(session, secret)
    }
}
