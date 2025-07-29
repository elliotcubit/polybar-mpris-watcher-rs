mod watcher;

use crate::watcher::Watcher;
use std::time::Duration;
use clap::Parser;

#[derive(Parser)]
#[command(version, about = "A good music status display for polybar", long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 1500, help="The frequency, in milliseconds, that the display window will update")]
    update_frequency: u64,
    #[arg(short, long, default_value_t = 15, help="The length of the display window")]
    banner_size: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut watcher = Watcher::new()?;
    watcher.watch(
        Duration::from_millis(args.update_frequency),
        args.banner_size
    )?;
    Ok(())
}
