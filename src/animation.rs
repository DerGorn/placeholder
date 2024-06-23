use std::time::Duration;

use placeholder::game_engine::SpriteSheetName;

pub struct Animation<T> {
    sprite_sheet: SpriteSheetName,
    keyframes: Vec<(Duration, T)>,
    current_keyframe: usize,
    time_since_frame_start: Duration,
    once: bool,
    running: bool,
}
impl<T> Animation<T> {
    pub fn new(sprite_sheet: SpriteSheetName, keyframes: Vec<(Duration, T)>, once: bool) -> Self {
        Self {
            sprite_sheet,
            keyframes,
            current_keyframe: 0,
            time_since_frame_start: Duration::from_millis(0),
            once,
            running: true,
        }
    }

    /// Returns true if the animation just ended, false otherwise.
    pub fn update(&mut self, delta_t: &Duration) -> bool {
        if !self.running {
            return false;
        }
        self.time_since_frame_start += *delta_t;
        if self.time_since_frame_start >= self.keyframes[self.current_keyframe].0 {
            self.current_keyframe = (self.current_keyframe + 1) % self.keyframes.len();
            if self.once && self.current_keyframe == 0 {
                self.current_keyframe = self.keyframes.len() - 1;
                self.running = false;
                return true;
            }
            self.time_since_frame_start -= self.keyframes[self.current_keyframe].0;
        }
        false
    }

    pub fn keyframe(&self) -> &T {
        &self.keyframes[self.current_keyframe].1
    }

    pub fn sprite_sheet(&self) -> &SpriteSheetName {
        &self.sprite_sheet
    }
}
