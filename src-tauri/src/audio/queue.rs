use crate::db::queries::Track;

#[derive(Debug, Clone, PartialEq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

pub struct Queue {
    pub tracks: Vec<Track>,
    pub current_index: Option<usize>,
    pub repeat: RepeatMode,
    pub shuffle: bool,
}

impl Queue {
    pub fn new() -> Self {
        Queue {
            tracks: Vec::new(),
            current_index: None,
            repeat: RepeatMode::Off,
            shuffle: false,
        }
    }

    // Replace entire queue with new tracks
    pub fn load(&mut self, tracks: Vec<Track>, start_index: usize) {
        self.tracks = tracks;
        self.current_index = Some(start_index);
    }

    // Get current track
    pub fn current(&self) -> Option<&Track> {
        self.current_index.and_then(|i| self.tracks.get(i))
    }

    // Move to next track - returns the next track if there is one
    pub fn next(&mut self) -> Option<&Track> {
        let len = self.tracks.len();
        if len == 0 {
            return None;
        }

        match self.repeat {
            RepeatMode::One => {
                // stay on same track
                self.current()
            }
            RepeatMode::All => {
                // wrap around to start when reaching end
                let next_index = match self.current_index {
                    Some(i) => (i + 1) % len,
                    None => 0,
                };
                self.current_index = Some(next_index);
                self.tracks.get(next_index)
            }
            RepeatMode::Off => {
                // stop at end
                let next_index = match self.current_index {
                    Some(i) if i + 1 < len => i + 1,
                    _ => return None,
                };
                self.current_index = Some(next_index);
                self.tracks.get(next_index)
            }
        }
    }

    // Move to previous track
    pub fn previous(&mut self) -> Option<&Track> {
        let len = self.tracks.len();
        if len == 0 {
            return None;
        }

        let prev_index = match self.current_index {
            Some(i) if i > 0 => i - 1,
            _ => 0,
        };

        self.current_index = Some(prev_index);
        self.tracks.get(prev_index)
    }

    // Jump to specific track by index
    pub fn jump_to(&mut self, index: usize) -> Option<&Track> {
        if index < self.tracks.len() {
            self.current_index = Some(index);
            self.tracks.get(index)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }
}
