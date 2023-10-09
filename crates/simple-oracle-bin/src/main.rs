use clap::Parser;
use confy;
use simple_oracle::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config: Config = confy::load(&args.config, None).expect("failed to load config");

    simple_oracle::start(&config).await.expect("fatal error");
}
