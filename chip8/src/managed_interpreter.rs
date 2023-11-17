use crate::{
    data::Word,
    error::Result,
    image::Image,
    interpreter::{Interpreter, SCREEN_HEIGHT, SCREEN_WIDTH},
    platform::{Key, Platform, Point, Sprite},
};

use core::time::Duration;
use std::ops::Add;

////////////////////////////////////////////////////////////////////////////////

pub struct FrameBuffer([[bool; SCREEN_WIDTH]; SCREEN_HEIGHT]);

impl Default for FrameBuffer {
    fn default() -> Self {
        Self([[false; SCREEN_WIDTH]; SCREEN_HEIGHT])
    }
}

impl FrameBuffer {
    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < SCREEN_WIDTH && y < SCREEN_HEIGHT
    }
    pub fn iter_rows(&self) -> impl Iterator<Item = &[bool; SCREEN_WIDTH]> {
        self.0.iter()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait RandomNumberGenerator: FnMut() -> Word {}

impl<R: FnMut() -> Word> RandomNumberGenerator for R {}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct ManagedPlatform<R: RandomNumberGenerator> {
    rand: R,
    frame_buffer: FrameBuffer,
    delay_timer: Word,
    sound_timer: Word,
    keys: [bool; 16],
}

impl<R: RandomNumberGenerator> Platform for ManagedPlatform<R> {
    fn draw_sprite(&mut self, pos: Point, sprite: Sprite) -> bool {
        let mut collision_detected = false;
        let pos = Point {
            x: pos.x % SCREEN_WIDTH as u8,
            y: pos.y % SCREEN_HEIGHT as u8,
        };
        for pixel in sprite.iter_pixels() {
            let p = pos.add(pixel);
            if !self.frame_buffer.in_bounds(p.x as usize, p.y as usize) {
                continue;
            }
            collision_detected |= self.frame_buffer.0[p.y as usize][p.x as usize];

            self.frame_buffer.0[p.y as usize][p.x as usize] ^= true
        }
        collision_detected
    }

    fn clear_screen(&mut self) {
        self.frame_buffer = Default::default();
    }

    fn get_delay_timer(&self) -> Word {
        self.delay_timer
    }

    fn set_delay_timer(&mut self, value: Word) {
        self.delay_timer = value;
    }

    fn set_sound_timer(&mut self, value: Word) {
        self.sound_timer = value;
    }

    fn is_key_down(&self, key: Key) -> bool {
        self.keys[key.as_usize()]
    }

    fn consume_key_press(&mut self) -> Option<Key> {
        for (index, x) in self.keys.iter().enumerate().clone() {
            if *x {
                return Some(Key::try_from(index as u8).unwrap());
            }
        }
        None
    }

    fn get_random_word(&mut self) -> Word {
        (self.rand)()
    }
}

impl<R: RandomNumberGenerator> ManagedPlatform<R> {
    fn new(rand: R) -> Self {
        Self {
            rand,
            frame_buffer: Default::default(),
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; 16],
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct ManagedInterpreter<R: RandomNumberGenerator> {
    inner: Interpreter<ManagedPlatform<R>>,
}

impl<R: RandomNumberGenerator> ManagedInterpreter<R> {
    pub const DEFAULT_OPERATION_DURATION: Duration = Duration::from_millis(2);
    pub const DEFAULT_DELAY_TICK_DURATION: Duration = Duration::from_nanos(16666667);
    pub const DEFAULT_SOUND_TICK_DURATION: Duration = Duration::from_nanos(16666667);
    pub const CYCLES_PER_SECOND: f64 = 60.0;
    pub const MS_PER_CYCLE: f64 = 1000.0 / Self::CYCLES_PER_SECOND;

    pub fn new(image: impl Image, rand: R) -> Self {
        Self::new_with_durations(image, rand)
    }

    pub fn new_with_durations(image: impl Image, rand: R) -> Self {
        Self {
            inner: Interpreter::new(image, ManagedPlatform::new(rand)),
        }
    }

    pub fn simulate_one_instruction(&mut self) -> Result<()> {
        self.inner.run_next_instruction()
    }

    pub fn simulate_duration(&mut self, duration: Duration) -> Result<()> {
        let period = (Self::DEFAULT_DELAY_TICK_DURATION.as_secs_f64() * 1000.0)
            / Self::DEFAULT_OPERATION_DURATION.as_millis() as f64;
        let max_iterations = (duration.as_millis() as f64
            / Self::DEFAULT_OPERATION_DURATION.as_millis() as f64)
            as usize;
        (0..max_iterations).for_each(|i| {
            self.simulate_one_instruction().unwrap();
            if i % period as usize == 0 && self.inner.platform().delay_timer > 0 {
                self.inner.platform_mut().delay_timer -= 1;
            }
        });
        Ok(())
    }

    pub fn frame_buffer(&self) -> &FrameBuffer {
        &self.inner.platform().frame_buffer
    }

    pub fn set_key_down(&mut self, key: Key, is_down: bool) {
        self.inner.platform_mut().keys[key.as_usize()] = is_down;
    }
}
