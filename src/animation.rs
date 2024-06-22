use std::time::Duration;

use placeholder::game_engine::SpriteSheetName;

use crate::Event;

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

    pub fn update(&mut self, delta_t: &Duration) -> Vec<Event> {
        if !self.running {
            return vec![];
        }
        self.time_since_frame_start += *delta_t;
        if self.time_since_frame_start >= self.keyframes[self.current_keyframe].0 {
            self.current_keyframe = (self.current_keyframe + 1) % self.keyframes.len();
            if self.once && self.current_keyframe == 0 {
                self.current_keyframe = self.keyframes.len() - 1;
                self.running = false;
                return vec![Event::AnimationEnded];
            }
            self.time_since_frame_start -= self.keyframes[self.current_keyframe].0;
        }
        vec![]
    }

    pub fn keyframe(&self) -> &T {
        &self.keyframes[self.current_keyframe].1
    }

    pub fn sprite_sheet(&self) -> &SpriteSheetName {
        &self.sprite_sheet
    }
}
