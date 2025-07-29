use std::{
    thread,
    time,
    sync::{
        Arc,
        RwLock,
    },
    cmp::min,
};

use mpris::{
    Event,
    FindingError,
    Metadata,
    MetadataValue,
    Player,
    PlayerFinder,
};

use unicode_segmentation::UnicodeSegmentation;

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

    pub fn watch(
        &mut self,
        update_interval: time::Duration,
        max_size: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let playing = Arc::new(RwLock::new(None));
        let playing_clone = Arc::clone(&playing);

        thread::spawn(move || {
            loop {
                {
                    let mut v = playing_clone.write().unwrap();
                    v.as_mut().map(|info: &mut PlayingInfo| {
                        println!("{}", info.next());
                    });
                }
                thread::sleep(update_interval);
            }
        });

        loop {
            if self.player.is_none() {
                self.player = Self::find_player(&self.finder)?;
                continue;
            }

            // Separate scope so that p is dropped, releasing the lock.
            {
                let mut p = playing.write().expect("OMGWTFBBQ");
                if p.is_none() {
                    *p = Some(PlayingInfo::new(self.player.as_mut().unwrap().get_metadata()?, max_size))
                }
            }

            let events = self.player.as_mut().unwrap().events()?;

            for evt in events {
                match evt {
                    Ok(Event::TrackChanged(m)) => {
                        let mut p = playing.write().expect("OMGWTFBBQ");
                        *p = Some(PlayingInfo::new(m, max_size))
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

pub struct PlayingInfo {
    arr: Vec<String>,
    start: usize,
    end: usize,
    size: usize,
}

impl PlayingInfo {
    fn new(metadata: Metadata, size: usize) -> Self{
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

        let s = format!(
            "{} - {} || ",
            artist.unwrap_or(&String::from(UNKNOWN)),
            title.unwrap_or(&String::from(UNKNOWN)),
        );

        let arr  = s.graphemes(true).map(|x| x.to_string()).collect::<Vec<String>>();
        let len = arr.len();

        Self{
            arr: arr,
            start: 0,
            end: min(size, len),
            size: min(size, len),
        }
    }

    fn get_window(&mut self) -> String {
        if self.end > self.start {
            self.arr[self.start .. self.end].join("")
        } else {
            self.arr[self.start..].join("") + &self.arr[0..self.end].join("")
        }
    }

    fn next(&mut self) -> String {
        let retv = self.get_window();
        self.start = (self.start + self.size) % self.arr.len();
        self.end = (self.end + self.size) % self.arr.len();
        retv
    }
}
