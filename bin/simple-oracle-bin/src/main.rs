use clap::Parser;
use simple_oracle_client::Config;
use tracing::{debug, error};

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

    let config: Config = confy::load_path(&args.config).expect("failed to load config");
    debug!("config: {config:?}");

    let mnemonic = std::env::var("COSMOS_WALLET").expect("COSMOS_WALLET mnemonic is required");

    check_required_fields(&config);

    if let Err(err) = simple_oracle_client::start(&config, mnemonic).await {
        error!("fatal error: {err}");
    }
}

fn check_required_fields(config: &Config) {
    if config.assets.is_empty() {
        panic!("assets is required");
    }

    for a in &config.assets {
        if config.contract_map.get(&a.ethereum_contract).is_none() {
            panic!("contract_map is missing entry for {}", a.ethereum_contract);
        }
    }
}
