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
    EventError,
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
            .map_err(|e| format!("Failed to connect to DBus: {}", e))?;

        let player = Self::find_player(&finder)
            .map_err(|e| format!("Error communicating with DBus: {}", e))?;

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
                self.player = Self::find_player(&self.finder)
                    .map_err(|e| format!("Error communicating with DBus: {}", e))?;
                if self.player.is_none() {
                    thread::sleep(time::Duration::from_millis(1000));
                }
                continue;
            }

            // Separate scope so that p is dropped, releasing the lock.
            {
                let mut p = playing.write().expect("Poisoned lock");
                if p.is_none() {
                    // safety: self.player is guaranteed to be Some earlier in the loop iteration
                    *p = Some(PlayingInfo::new(self.player.as_mut().unwrap().get_metadata()?, max_size))
                }
            }

            // safety: self.player is guaranteed to be Some earlier in the loop iteration
            let events = self.player.as_mut().unwrap().events()
                .map_err(|e| format!("Error communicating with DBus: {}", e))?;

            for evt in events {
                match evt {
                    Ok(Event::TrackChanged(m)) => {
                        let mut p = playing.write().expect("Poisoned lock");
                        *p = Some(PlayingInfo::new(m, max_size))
                    },
                    Ok(Event::PlayerShutDown) => {
                        self.player = None;
                        let mut p = playing.write().expect("Poisoned lock");
                        *p = None;
                        break;
                    },
                    Err(EventError::DBusError(_)) => {
                        // TODO: Shutting down a player seems to error without
                        // giving PlayerShutDown, contrary to docs.
                        self.player = None;
                        let mut p = playing.write().expect("Poisoned lock");
                        *p = None;
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
    display: Option<String>,
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

        Self::from_info(artist, title, size)
    }

    fn from_info(artist: Option<&String>, title: Option<&String>, size: usize) -> Self {
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
            display: {
                // Remove scrolling separator when it isn't necessary
                if len - 4 <= size {
                    Some(
                        format!(
                            "{:^size$}",
                            format!(
                                "{} - {}",
                                artist.unwrap_or(&String::from(UNKNOWN)),
                                title.unwrap_or(&String::from(UNKNOWN)),
                            ),
                            size = size,
                        )
                    )
                } else {
                    None
                }
            },
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
        self.display.clone().unwrap_or(
            {
                let retv = self.get_window();
                self.start = (self.start + self.size) % self.arr.len();
                self.end = (self.end + self.size) % self.arr.len();
                retv
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(info: &mut PlayingInfo, frames: Vec<&str>) {
        for frame in frames {
            assert_eq!(frame, info.next());
        }
    }

    #[test]
    fn test_no_info() {
        test(
            &mut PlayingInfo::from_info(None, None, 1),
            vec![ "?", "?", "?", " ", "-", " ", "?", "?", "?", " ", "|", "|", " ", "?" ],
        )
    }

    #[test]
    fn test_entire_subject_fits_in_window() {
        test(
            &mut PlayingInfo::from_info(
                Some(&"A".to_string()),
                Some(&"B".to_string()),
                5,
            ),
            vec![ "A - B", "A - B" ],
        )
    }

    #[test]
    // If we have space to spare the subject should be centered and padded out
    // to fill the entire window
    fn test_window_bigger_than_subject() {
        test(
            &mut PlayingInfo::from_info(
                Some(&"A".to_string()),
                Some(&"B".to_string()),
                7,
            ),
            vec![ " A - B ", " A - B " ],
        );
        test(
            &mut PlayingInfo::from_info(
                Some(&"A".to_string()),
                Some(&"B".to_string()),
                8,
            ),
            vec![ " A - B  ", " A - B  " ],
        )
    }


    #[test]
    fn test_scrolling() {
        test(
            &mut PlayingInfo::from_info(
                Some(&"Bob Marley & The Wailers".to_string()),
                Some(&"Easy Skanking".to_string()),
                3,
            ),
            vec![ "Bob", " Ma", "rle", "y &", " Th", "e W", "ail", "ers", " - ", "Eas", "y S", "kan", "kin", "g |", "| B", "ob "],
        )
    }

    // Some visual characters are composed of multiple unicode code-points, so
    // ensure we aren't using bytes or chars for display breaking.
    #[test]
    fn test_unicode_graphemes() {
        test(
            &mut PlayingInfo::from_info(
                Some(&"P\u{0065}\u{0301}n".to_string()),
                Some(&"P\u{0065}\u{0301}n".to_string()),
                5,
            ),
            vec![
                "P\u{0065}\u{0301}n -",
                " P\u{0065}\u{0301}n ",
                "|| P\u{0065}\u{0301}",
            ],
        )
    }

    // Same deal as above but with a real song
    //
    // デストロイ!!!
    // デストロイ!!!
    // デストロイ!!!
    // デストロイ!!!
    //
    #[test]
    fn test_midori() {
        test(
            &mut PlayingInfo::from_info(
                Some(&"ミドリ".to_string()),
                Some(&"ゆきこさん".to_string()),
                5,
            ),
            vec![
                "ミドリ -",
                " ゆきこさ",
                "ん || ",
                "ミドリ -",
                " ゆきこさ",
                "ん || ",
            ],
        )
    }
}
