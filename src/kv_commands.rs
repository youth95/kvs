use std::{
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use clap::Subcommand;

use crate::{
    actions::{CatAction, CreateKeyValueAction, KeyMeta},
    config::{get_or_create_jwt_secret, get_or_create_token, get_or_create_user_config_dir},
    errors::KVSResult,
    kv_server::service,
    kv_session::KVSSession,
    spec::KVSAction,
};

#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
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

impl Commands {
    pub fn run(&self, repository: &String) -> KVSResult<()> {
        let get_kvs_session = || -> KVSResult<KVSSession> {
            let stream = TcpStream::connect(repository)?;
            stream
                .set_read_timeout(Some(Duration::from_millis(1000)))
                .expect("fail");

            KVSSession::new(stream)
        };

        match self {
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

                    let stdout_file = if let Ok(stdout_file) = std::fs::File::open("kvs.log") {
                        stdout_file
                    } else {
                        std::fs::File::create("kvs.log")?
                    };

                    let stderr_file = if let Ok(stderr_file) = std::fs::File::open("kvs.errors.log")
                    {
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

                    let kvs_pid_file_path = get_or_create_user_config_dir()?.join("pid");
                    std::fs::write(kvs_pid_file_path, child.id().to_string())?;
                    tracing::info!("kvs started PID: {}", child.id());
                    tracing::info!("The logs saved to ./kvs.log and ./kvs.errors.log");

                    return Ok(());
                }

                let jwt_secret = get_or_create_jwt_secret(*reset_jwt_secret)?;

                let listener = TcpListener::bind(repository)?;
                tracing::info!("starting with {} successfully!", repository);
                for stream in listener.incoming() {
                    match stream {
                        Err(e) => {
                            tracing::error!("{}", e)
                        }
                        Ok(stream) => {
                            let jwt_secret = jwt_secret.clone();
                            stream.set_read_timeout(Some(Duration::from_millis(1000)))?;
                            let mut session = KVSSession::new(stream)?;
                            thread::spawn(move || service(&mut session, &jwt_secret));
                        }
                    }
                }
            }
            Commands::Login => {
                let (_, user_token_file_path) = get_or_create_token(repository, true)?;
                tracing::info!("Save Token file to: {}", user_token_file_path);
            }
            Commands::Create {
                key,
                value,
                private,
            } => {
                let (token, _) = get_or_create_token(repository, false)?;
                let mut session = get_kvs_session()?;
                let value = value.as_bytes().to_vec();
                let size = value.len() as u64;
                let owner = token.id.clone();

                let rand = if *private {
                    Some((0..32).map(|_| rand::random::<u8>()).collect::<Vec<u8>>())
                } else {
                    None
                };
                CreateKeyValueAction {
                    token: token.clone(),
                    key: key.to_string(),
                    value,
                    meta: KeyMeta {
                        mime: "text/plain".to_string(),
                        size,
                        owner,
                        name: key.to_string(),
                        rand,
                    },
                }
                .request(&mut session)?
            }
            Commands::Cat { key } => {
                let (token, _) = get_or_create_token(&repository, false)?;
                let mut session = get_kvs_session()?;
                let reply = CatAction {
                    token: token.clone(),
                    key: key.to_string(),
                }
                .request(&mut session)?;
                let content = reply.content();
                let content_str = String::from_utf8(content.to_vec()).unwrap();
                println!("{}", content_str);
            }
        };
        Ok(())
    }
}
