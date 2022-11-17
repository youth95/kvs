use clap::Parser;
use key_value_service::Commands;
use tracing_subscriber::prelude::*;

#[derive(Parser, Debug, Clone)]
#[clap(author="zmp <zhaoqian.ipp@gmail.com>", version, about="Key Value Service", long_about = None)]
struct KVS {
    #[clap(subcommand)]
    command: Commands,

    #[clap(short, long, help = "Use verbose output")]
    verbose: bool,

    #[clap(short, long, help = "Set Repository")]
    repository: Option<String>,
}

fn main() {
    let kvs_cli = KVS::parse();

    tracing_subscriber::registry()
        // Filter spans based on the RUST_LOG env var.
        .with(tracing_subscriber::EnvFilter::new(if kvs_cli.verbose {
            "error,key_value_service=debug"
        } else {
            "error,key_value_service=info"
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
    match kvs_cli.command.run(&kvs_cli.repository) {
        Ok(_) => (),
        Err(error) => tracing::error!("{:?}", error),
    }
}
