use tracing_subscriber::filter::LevelFilter;

#[derive(clap::Parser)]
pub struct Args {
    #[arg(short, long)]
    pub batch: Vec<u16>,
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbosity: u8,
}

impl Args {
    pub fn level_filter(&self) -> LevelFilter {
        match self.verbosity {
            0 => LevelFilter::INFO,
            1 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    }
}
