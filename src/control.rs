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

use crate::args::Operation;
use mpris::{
    PlayerFinder
};

pub fn control(
    player_name: String,
    command: Operation,
) -> Result<(), Box<dyn std::error::Error>> {

    let player = PlayerFinder::new()?
        .iter_players()?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .find(|player| {
            println!("{}", player.bus_name_player_name_part());
            player.bus_name_player_name_part() == player_name
        })
        .expect("Player not found");

    match command {
        Operation::PREVIOUS => player.previous()?,
        Operation::TOGGLE => player.play_pause()?,
        Operation::NEXT => player.next()?,
    }

    Ok(())
}
