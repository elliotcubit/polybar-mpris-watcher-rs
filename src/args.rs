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

use clap::{
    ValueEnum,
    Parser,
    Subcommand,
};

#[derive(Parser)]
#[command(version, about = "A good music status display for polybar", long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = 1500, help="The frequency, in milliseconds, that the display window will update")]
    pub update_frequency: u64,

    #[arg(short, long, default_value_t = 15, help="The length of the display window")]
    pub banner_size: usize,

    #[arg(short, long, help="Include player controls in the module")]
    pub include_controls: bool,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Operation {
    PREVIOUS,
    TOGGLE,
    NEXT,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Control the player instead of running the display daemon")]
    Control {
        #[arg(short, long, help="The player to control")]
        player: String,

        #[arg(short, long, help="The operation to execute")]
        operation: Operation,
    },
}

