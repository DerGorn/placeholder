use std::{
    ops::{Add, Div, Mul, Sub},
    time::Duration,
};

fn lerp<V>(start: V, end: V, steps: u16) -> Vec<V>
where
    for<'a> &'a V: Sub<Output = V> + Mul<f32, Output = V> + Add<V, Output = V>,
    V: Div<f32, Output = V>,
{
    let delta = (&end - &start) / (steps - 1) as f32;
    let mut interpolations = Vec::new();
    for i in 0..steps {
        let step = &start + &delta * (i as f32);
        interpolations.push(step);
    }
    interpolations
}

pub struct Animation<T> {
    keyframes: Vec<(Duration, T)>,
    current_keyframe: usize,
    time_since_frame_start: Duration,
    once: bool,
    running: bool,
}
impl<T> Animation<T> {
    pub fn new(keyframes: Vec<(Duration, T)>, once: bool) -> Self {
        Self {
            keyframes,
            current_keyframe: 0,
            time_since_frame_start: Duration::from_millis(0),
            once,
            running: true,
        }
    }

    pub fn lerp<V>(
        start: V,
        end: V,
        animation_steps: u16,
        animation_time: Duration,
        constructor: &dyn Fn(V) -> T,
        once: bool,
    ) -> Self
    where
        for<'a> &'a V: Sub<Output = V> + Mul<f32, Output = V> + Add<V, Output = V>,
        V: Div<f32, Output = V>,
    {
        let frame_time = (animation_time.as_nanos() / (animation_steps as u128)) as u64;
        Self::new(
            lerp(start, end, animation_steps)
                .into_iter()
                .map(|v| (Duration::from_nanos(frame_time), constructor(v)))
                .collect(),
            once,
        )
    }

    /// Returns true if the animation just ended, false otherwise.
    pub fn update(&mut self, delta_t: &Duration) -> bool {
        if !self.running {
            return false;
        }
        if self.once && self.keyframes.len() == 0 {
            self.running = false;
            return true;
        }
        self.time_since_frame_start += *delta_t;
        while self.time_since_frame_start >= self.keyframes[self.current_keyframe].0 {
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
}
