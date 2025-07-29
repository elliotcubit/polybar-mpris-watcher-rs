mod watcher;

use crate::watcher::Watcher;
use std::time;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut watcher = Watcher::new()?;
    watcher.watch(time::Duration::from_millis(1500), 13)?;
    Ok(())
}


