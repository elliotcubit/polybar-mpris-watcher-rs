// Copyright 2025 Elaine Cubit
//
// This file is part of polybar-mpris-watcher-rs.
//
// polybar-mpris-watcher-rs is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// polybar-mpris-watcher-rs is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
// or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
// more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>. 

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
    #[arg(short, long, help="Include player controls in the module")]
    include_controls: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut watcher = Watcher::new()?;
    watcher.watch(
        Duration::from_millis(args.update_frequency),
        args.banner_size - if args.include_controls { 4 } else { 0 },
        args.include_controls,
    )?;
    Ok(())
}
