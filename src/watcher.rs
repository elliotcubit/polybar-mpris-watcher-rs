use std::{
    thread,
    time,
    sync::{
        Arc,
        RwLock,
    },
};

use mpris::{
    Event,
    FindingError,
    Metadata,
    MetadataValue,
    Player,
    PlayerFinder,
};

pub struct Watcher {
    finder: PlayerFinder,
    player: Option<Player>,
}

const UNKNOWN: &'static str = "???";

impl Watcher {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let finder = PlayerFinder::new()
            .map_err(|e| format!("Failed to connect to DBUS: {}", e))?;

        let player = Self::find_player(&finder)?;

        Ok(Self{
            finder,
            player,
        })
    }

    pub fn watch(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        let playing = Arc::new(RwLock::new(None));
        let playing_clone = Arc::clone(&playing);

        thread::spawn(move || {
            loop {
                {
                    let v = playing_clone.read().unwrap();

                    v.as_ref().map(|metadata: &Metadata| {
                        let artist = match metadata.get("xesam:artist") {
                            Some(MetadataValue::Array(vvs)) => {
                                match vvs.first() {
                                    Some(MetadataValue::String(s)) => Some(s),
                                    _ => None
                                }
                            },
                            _ => None
                        };

                        let title = match metadata.get("xesam:title") {
                            Some(MetadataValue::String(s)) => Some(s),
                            _ => None
                        };

                        println!(
                            "{} - {}",
                            artist.unwrap_or(&String::from(UNKNOWN)),
                            title.unwrap_or(&String::from(UNKNOWN)),
                        );
                    });
                }

                thread::sleep(time::Duration::from_millis(2000));
            }
        });

        loop {
            if self.player.is_none() {
                self.player = Self::find_player(&self.finder)?;
                continue;
            }

            {
                let mut p = playing.write().expect("OMGWTFBBQ");
                if p.is_none() {
                    *p = Some(self.player.as_mut().unwrap().get_metadata()?);
                }
            }

            let events = self.player.as_mut().unwrap().events()?;

            for evt in events {
                match evt {
                    Ok(Event::TrackChanged(m)) => {
                        let mut p = playing.write().expect("OMGWTFBBQ");
                        *p = Some(m);
                    },
                    Ok(Event::PlayerShutDown) => {
                        self.player = None;
                        break;
                    },
                    Err(e) => {
                        Err(e)?
                    },
                    _ => {}
                }
            }
        }
    }

    fn find_player(finder: &PlayerFinder) -> Result<Option<Player>, Box<dyn std::error::Error>> {
        match finder.find_active() {
            Ok(v) => Ok(Some(v)),
            Err(FindingError::NoPlayerFound) => Ok(None),
            Err(FindingError::DBusError(e)) => Err(format!("Error communicating with DBUS: {}", e))?,
        }
    }
}

