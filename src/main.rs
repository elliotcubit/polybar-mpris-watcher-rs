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

mod args;
mod control;
mod watcher;

use clap::Parser;
use crate::args::{Args, Command};
use crate::watcher::Watcher;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.command.is_some() {
        match args.command.unwrap() {
            Command::Control { player, operation } =>
                control::control(player, operation)?
        }
    } else {
        let mut watcher = Watcher::new()?;
        let mut include_controls_binary = None;
        if args.include_controls {
            include_controls_binary = Some(
                std::env::current_exe()?
                    .into_os_string()
                    .into_string()
                    .map_err(|_| format!("Failed to find current executable path??"))?
                );
        }
        watcher.watch(
            Duration::from_millis(args.update_frequency),
            args.banner_size - if args.include_controls { 4 } else { 0 },
            include_controls_binary,
        )?;
    }

    Ok(())
}
