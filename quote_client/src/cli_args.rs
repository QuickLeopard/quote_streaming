use clap::Parser;

#[derive(Parser)]
#[command(name = "quote_client")]
#[command(about = "Quote Client")]
pub struct CliArgs {
    #[arg(short = 'H', long)]
    pub host: String,

    #[arg(short, long)]
    pub port: u16,

    #[arg(short = 'A', long)]
    pub stream_addr: String,

    #[arg(short = 'T', long)]
    pub tickers: String,
}
