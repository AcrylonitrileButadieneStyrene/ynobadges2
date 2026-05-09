#[derive(clap::Parser)]
pub struct Args {
    #[arg(short, long)]
    pub batch: Vec<u16>,
}
