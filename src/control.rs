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
