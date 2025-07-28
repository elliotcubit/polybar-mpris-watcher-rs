mod watcher;
use crate::watcher::Watcher;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut watcher = Watcher::new()?;
    watcher.watch()?;
    Ok(())
}


