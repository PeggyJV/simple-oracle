use clap::Parser;
use simple_oracle_client::Config;
use tracing::error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    if args.config.is_empty() {
        panic!("config file path is required");
    }

    let config: Config = confy::load(&args.config, None).expect("failed to load config");

    if let Err(err) = simple_oracle_client::start(&config).await {
        error!("fatal error: {err}");
    }
}
