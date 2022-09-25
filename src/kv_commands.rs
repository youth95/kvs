use indicatif::ProgressIterator;
use std::{
    collections::HashMap,
    io::Read,
    net::{TcpListener, TcpStream},
    time::Duration,
};
use xshell::{cmd, Shell};

use clap::Subcommand;

use crate::{
    actions::{
        CreateAction, DeleteAction, KeyMeta, ListAction, LocalFileMeta, ReadAction,
        RemoteVersionAction, UpdateAction,
    },
    config::{
        get_or_create_data_dir, get_or_create_jwt_secret, get_or_create_repository_config,
        get_or_create_secret, get_or_create_token, get_or_create_user_config_dir,
        get_or_create_user_config_kv_dir,
    },
    errors::{KVSError, KVSResult},
    kv_server::service,
    kv_session::KVSSession,
    secret::Secret,
    spec::KVSAction,
    utils::{sha256, to_addr},
};

#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    #[clap(long_about = "Start kvs server")]
    Start {
        #[clap(short, long, help = "reset the jwt_secret")]
        reset_jwt_secret: bool,
        #[clap(short, long, help = "start kvs server in bg")]
        detach: bool,
    },
    #[clap(long_about = "Stop kvs server")]
    Stop,
    #[clap(long_about = "Restart kvs server")]
    Restart {
        #[clap(short, long, help = "reset the jwt_secret")]
        reset_jwt_secret: bool,
    },
    #[clap(long_about = "Login to kvs")]
    Login,
    #[clap(long_about = "Create key value")]
    Create {
        key: String,

        value: Option<String>,

        #[clap(short, long, help = "Use file content")]
        file: Option<Option<String>>,

        #[clap(short, long, help = "As public key")]
        public: bool,
        #[clap(short, long, help = "Value Type", default_value = "text/plain")]
        value_type: String,
    },

    #[clap(long_about = "Update key value")]
    Update {
        key: String,

        value: Option<String>,

        #[clap(short, long, help = "Use file content")]
        file: Option<Option<String>>,

        #[clap(short, long, help = "As public key")]
        public: bool,

        #[clap(short, long, help = "Value Type", default_value = "text/plain")]
        value_type: String,
    },

    #[clap(long_about = "Read key content")]
    Read { key: String },
    #[clap(long_about = "Delete key")]
    Delete { key: String },

    #[clap(
        long_about = "Upload all file in current directory and use the relative directory as key"
    )]
    Sync {
        #[clap(help = "Dir path", default_value = ".")]
        path: String,

        #[clap(short, long, help = "As public key")]
        public: bool,
    },

    #[clap(long_about = "List all keys info")]
    List {
        #[clap(short, long, help = "add scope in public key")]
        public: bool,
    },

    #[clap(long_about = "Show remote info")]
    Remote,
    #[clap(long_about = "Show local info")]
    Local,

    #[clap(long_about = "Set client config")]
    Set { key: String, value: String },

    #[clap(long_about = "Get client config")]
    Get { key: String },

    #[clap(long_about = "Remote data dir")]
    Clear,

    #[clap(long_about = "Encrypt content with pub_key,input text, output base64")]
    En { content: String },

    #[clap(long_about = "Decrypt content with priv_key,input base64, output text")]
    De { content: String },
}

impl Commands {
    pub fn run(&self, repository: &Option<String>) -> KVSResult<()> {
        let repository = &match repository {
            Some(repository) => repository.to_string(),
            None => get_or_create_repository_config()?,
        };
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
                let repository = "0.0.0.0:8888";
                if *detach {
                    let args = std::env::args().map(|x| x).collect::<Vec<String>>();
                    let detach_command_args = args[1..]
                        .to_vec()
                        .into_iter()
                        .filter(|arg| *arg != "-d" && *arg != "--detach")
                        .map(|item| {
                            if item == "restart" {
                                "start".to_string()
                            } else {
                                item
                            }
                        })
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
                            .spawn();
                    match child {
                        Ok(child) => {
                            let kvs_pid_file_path = get_or_create_user_config_dir()?.join("pid");
                            std::fs::write(kvs_pid_file_path, child.id().to_string())?;
                            tracing::info!("kvs started PID: {}", child.id());
                            tracing::info!("The logs saved to ./kvs.log and ./kvs.errors.log");
                        }
                        Err(_) => todo!(),
                    }

                    return Ok(());
                }

                let jwt_secret = get_or_create_jwt_secret(*reset_jwt_secret)?;

                let listener = TcpListener::bind(repository)?;
                tracing::info!("starting with {} successfully!", repository);
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(8)
                    .build()
                    .unwrap();
                for stream in listener.incoming() {
                    match stream {
                        Err(e) => {
                            tracing::error!("{}", e)
                        }
                        Ok(stream) => {
                            let jwt_secret = jwt_secret.clone();
                            stream.set_read_timeout(Some(Duration::from_millis(1000)))?;
                            let mut session = KVSSession::new(stream)?;
                            pool.install(move || service(&mut session, &jwt_secret));
                        }
                    }
                }
            }
            Commands::Stop => {
                let kvs_pid_file_path = get_or_create_user_config_dir()?.clone().join("pid");
                if *&kvs_pid_file_path.exists() {
                    let pid = std::fs::read_to_string(&kvs_pid_file_path)?;
                    tracing::info!("kvs PID: {}", pid);
                    let sh = Shell::new().unwrap();
                    cmd!(sh, "kill -9 {pid}").run().unwrap();
                    std::fs::remove_file(kvs_pid_file_path).unwrap();
                } else {
                    tracing::error!("kvs server not started")
                }
            }
            Commands::Restart { reset_jwt_secret } => {
                Commands::Stop.run(&Some(repository.clone()))?;
                Commands::Start {
                    reset_jwt_secret: *reset_jwt_secret,
                    detach: true,
                }
                .run(&Some(repository.clone()))?;
            }
            Commands::Login => {
                let (_, user_token_file_path) = get_or_create_token(repository, true)?;
                tracing::info!("Save Token file to: {}", user_token_file_path);
            }
            Commands::Create {
                key,
                value,
                public,
                value_type,
                file,
            } => {
                let (token, _) = get_or_create_token(repository, false)?;
                let mut session = get_kvs_session()?;

                let value = match value {
                    Some(value) => value.as_bytes().to_vec(),
                    None => match file {
                        Some(file_path) => match file_path {
                            Some(file_path) => std::fs::read(file_path)?,
                            None => {
                                let mut buf = Vec::<u8>::new();
                                std::io::stdin().read_to_end(&mut buf)?;
                                buf
                            }
                        },
                        None => {
                            return Err(KVSError::LogicError(
                                "value params and file option can not to None in same time"
                                    .to_string(),
                            ))
                        }
                    },
                };

                let size = value.len() as u64;
                let owner = token.id.clone();

                let rand = if *public == false {
                    Some((0..32).map(|_| rand::random::<u8>()).collect::<Vec<u8>>())
                } else {
                    None
                };
                CreateAction {
                    token: token.clone(),
                    key: key.to_string(),
                    value: value.to_vec(),
                    meta: KeyMeta {
                        mime: value_type.to_string(),
                        size,
                        owner,
                        name: key.to_string(),
                        rand,
                        original_hash: sha256(&value[..]),
                    },
                }
                .request(&mut session)?
            }
            Commands::Update {
                key,
                value,
                public,
                value_type,
                file,
            } => {
                let (token, _) = get_or_create_token(repository, false)?;
                let mut session = get_kvs_session()?;
                let value = match value {
                    Some(value) => value.as_bytes().to_vec(),
                    None => match file {
                        Some(file_path) => match file_path {
                            Some(file_path) => std::fs::read(file_path)?,
                            None => {
                                let mut buf = Vec::<u8>::new();
                                std::io::stdin().read_to_end(&mut buf)?;
                                buf
                            }
                        },
                        None => {
                            return Err(KVSError::LogicError(
                                "value params and file option can not to None in same time"
                                    .to_string(),
                            ))
                        }
                    },
                };
                let size = *&value.len() as u64;
                let owner = token.id.clone();

                let rand = if *public == false {
                    Some((0..32).map(|_| rand::random::<u8>()).collect::<Vec<u8>>())
                } else {
                    None
                };
                UpdateAction {
                    token: token.clone(),
                    key: key.to_string(),
                    value: value.to_vec(),
                    meta: KeyMeta {
                        mime: value_type.to_string(),
                        size,
                        owner,
                        name: key.to_string(),
                        rand,
                        original_hash: sha256(&value[..]),
                    },
                }
                .request(&mut session)?;
            }

            Commands::Read { key } => {
                let (token, _) = get_or_create_token(&repository, false)?;
                let mut session = get_kvs_session()?;
                let scope = match key.split(":").next() {
                    Some(scope) => Some(scope.to_string()),
                    None => None,
                };
                let key = match key.split(":").nth(1) {
                    Some(key) => key.to_string(),
                    None => key.to_string(),
                };
                let scope = scope.clone();
                let reply = ReadAction {
                    token: token.clone(),
                    key: key.to_string(),
                    scope,
                }
                .request(&mut session)?;
                let content = reply.content();
                let content_str = String::from_utf8(content.to_vec()).unwrap();
                println!("{}", content_str);
            }
            Commands::Delete { key } => {
                let (token, _) = get_or_create_token(&repository, false)?;
                let mut session = get_kvs_session()?;
                DeleteAction {
                    token: token.clone(),
                    key: key.to_string(),
                }
                .request(&mut session)?;
            }
            Commands::Set { key, value } => {
                let user_config_kv_dir = get_or_create_user_config_kv_dir()?;
                let key_config_file_path = user_config_kv_dir.join(key);
                std::fs::write(key_config_file_path, value.as_bytes())?
            }
            Commands::Get { key } => {
                let user_config_kv_dir = get_or_create_user_config_kv_dir()?;
                let key_config_file_path = user_config_kv_dir.join(key);
                if key_config_file_path.exists() {
                    println!("{}", std::fs::read_to_string(key_config_file_path)?)
                }
            }
            Commands::Remote => {
                let mut session = get_kvs_session()?;
                let version = RemoteVersionAction.request(&mut session)?;
                println!("{}", version);
            }
            Commands::Local => {
                let secret = get_or_create_secret()?;
                let scope = to_addr(&secret.pub_key_bits);
                let pub_key = base64::encode(secret.pub_key_bits.clone());
                println!("scope: {}", scope);
                println!("pub key: {}", pub_key);
            }
            Commands::Sync { path, public } => {
                let (token, _) = get_or_create_token(&repository, false)?;
                let all_files_meta = LocalFileMeta::get_all_files_meta(path)?;
                tracing::info!("analysis remote files");
                let mut session = get_kvs_session()?;
                let remote_key_meta_list = ListAction { token }.request(&mut session)?;
                let remote_key_meta_mapper = HashMap::<String, &KeyMeta>::from_iter(
                    remote_key_meta_list
                        .iter()
                        .map(|meta| (meta.name.clone(), meta)),
                );
                let need_create_keys = all_files_meta
                    .iter()
                    .filter(|meta| !remote_key_meta_mapper.contains_key(&meta.name))
                    .collect::<Vec<_>>();

                if need_create_keys.len() > 0 {
                    tracing::info!("need create keys {}", need_create_keys.len());

                    need_create_keys
                        .iter()
                        .progress_count(need_create_keys.len() as u64)
                        .for_each(|meta| {
                            Commands::Create {
                                key: meta.name.to_string(),
                                value: None,
                                file: Some(Some(meta.path.to_string())),
                                public: *public,
                                value_type: "bin".to_string(),
                            }
                            .run(&Some(repository.clone()))
                            .unwrap_or_else(|error| tracing::error!("{:?}", error));
                        });
                    tracing::info!("created {} keys successfully", need_create_keys.len());
                }
                let need_update_keys = all_files_meta
                    .iter()
                    .filter(|meta| match remote_key_meta_mapper.get(&meta.name) {
                        Some(target) => {
                            let no_same_hash = target.original_hash != meta.original_hash;
                            let no_same_public = target.rand.is_none() != *public;
                            no_same_hash || no_same_public
                        }
                        None => false,
                    })
                    .collect::<Vec<_>>();
                if need_update_keys.len() > 0 {
                    tracing::info!("need update files {}", need_update_keys.len());
                    need_update_keys
                        .iter()
                        .progress_count(need_update_keys.len() as u64)
                        .for_each(|meta| {
                            Commands::Update {
                                key: meta.name.to_string(),
                                value: None,
                                file: Some(Some(meta.path.to_string())),
                                public: *public,
                                value_type: "bin".to_string(),
                            }
                            .run(&Some(repository.clone()))
                            .unwrap_or_else(|error| tracing::error!("{:?}", error));
                        });
                    tracing::info!("upload {} keys successfully", need_update_keys.len());
                }
                tracing::info!("sync finish")
            }
            Commands::List { public } => {
                let (token, _) = get_or_create_token(&repository, false)?;
                let secret = get_or_create_secret()?;
                let scope = to_addr(&secret.pub_key_bits);
                let mut session = get_kvs_session()?;
                let key_meta_list = ListAction { token }.request(&mut session)?;
                key_meta_list.iter().for_each(|meta| {
                    println!(
                        "{} {}\t{}",
                        if meta.rand.is_none() {
                            "public"
                        } else {
                            "private"
                        },
                        meta.size,
                        if meta.rand.is_none() && *public {
                            format!("{}:{}", scope.to_string(), meta.name).to_string()
                        } else {
                            meta.name.to_string()
                        },
                    );
                });
            }
            Commands::Clear => {
                let data_dir = get_or_create_data_dir()?;
                remove_dir_all::remove_dir_all(data_dir)?;
            }
            Commands::En { content } => {
                if content == "pplove" {
                    crate::letter::print_letter();
                    return Ok(());
                }
                let secret = get_or_create_secret()?;
                let result =
                    Secret::encrypt_with_pub_key_bits(&secret.pub_key_bits, content.as_bytes());
                println!("{}", base64::encode(result));
            }
            Commands::De { content } => {
                let secret = get_or_create_secret()?;
                let data = base64::decode(content).unwrap();
                let result = Secret::decrypt_width_priv_key_bits(&secret.priv_key_bits, &data)?;
                println!("{}", String::from_utf8(result).unwrap());
            }
        };
        Ok(())
    }
}
