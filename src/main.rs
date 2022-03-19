// TODO server and client need two bin

use std::{net::TcpStream, time::Duration};

use clap::{ArgEnum, Parser, Subcommand};
use kvs::{
    cat,
    config::{get_or_create_jwt_secret, get_or_create_token, get_or_create_user_config_dir},
    create_key_value, start_server, KVSResult, KVSSession,
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
    Create {
        key: String,
        value: String,

        #[clap(short, long, help = "start kvs server in bg")]
        private: bool,
    },

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

    match &kvs_cli.command {
        Commands::Start {
            reset_jwt_secret,
            detach,
        } => {
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

                let kvs_pid_file_path = get_or_create_user_config_dir().join("pid");
                std::fs::write(kvs_pid_file_path, child.id().to_string())?;
                tracing::info!("kvs started PID: {}", child.id());
                tracing::info!("The logs saved to ./kvs.log and ./kvs.errors.log");

                return Ok(());
            }

            let jwt_secret = get_or_create_jwt_secret(*reset_jwt_secret);

            start_server(&kvs_cli.repository, &jwt_secret)?;
        }

        Commands::Login => {
            let (_, user_token_file_path) = get_or_create_token(&kvs_cli.repository, true);
            tracing::info!("Save Token file to: {}", user_token_file_path);
        }
        Commands::Create {
            key,
            value,
            private,
        } => {
            let (token, _) = get_or_create_token(&kvs_cli.repository, false);
            let mut session = get_kvs_session()?;
            create_key_value(&mut session, &token, key, value, private)?;
        }
        Commands::Cat { key } => {
            let (token, _) = get_or_create_token(&kvs_cli.repository, false);
            let mut session = get_kvs_session()?;
            let content = cat(&mut session, &token, key)?;
            let content_str = String::from_utf8(content).unwrap();
            println!("{}", content_str);
        }
    };

    Ok(())
}
