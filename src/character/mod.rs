use std::fmt::{Debug, Display};

mod skilled_character;
pub use skilled_character::{CharacterBuilder, CharacterGuiManager, SkilledCharacter, CHARACTER_FONT_SIZE};

mod ki;
pub use ki::{KIBehavior, NoKI, SimpleKI};

pub mod skills;

pub mod characters {
    pub use super::skilled_character::characters::*;
}

pub mod ui {
    pub use super::skilled_character::ui::*;
}

#[derive(PartialEq, Clone)]
pub enum CharacterAlignment {
    Friendly,
    Enemy,
}

#[derive(Clone)]
pub struct Character {
    name: &'static str,
    alignment: CharacterAlignment,

    max_health: u16,
    health: u16,
    max_stamina: u16,
    stamina: u16,
    exhaustion_threshold: u16,
    exhaustion: u16,

    speed: u16,
    attack: u16,
}
impl Character {
    pub fn new(
        name: &'static str,
        alignment: CharacterAlignment,
        max_health: u16,
        max_stamina: u16,
        exhaustion_threshold: u16,
        speed: u16,
        attack: u16,
    ) -> Self {
        Self {
            name,
            alignment,
            max_health,
            health: max_health,
            max_stamina,
            stamina: max_stamina,
            exhaustion_threshold,
            exhaustion: 0,
            speed,
            attack,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn alignment(&self) -> &CharacterAlignment {
        &self.alignment
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }
}
impl Debug for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {{ HP: {}/{}, ST: {}/{}, EX: {}  ||  SPD: {}, ATK: {} }}",
            self.name,
            self.health,
            self.max_health,
            self.stamina,
            self.max_stamina,
            self.exhaustion,
            self.speed,
            self.attack
        )
    }
}

pub const CHARACTER_DISPLAY_LINES: f32 = 4.0;
impl Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}
HP: {}/{}
ST: {}/{}
EX: {}",
            self.name,
            self.health,
            self.max_health,
            self.stamina,
            self.max_stamina,
            self.exhaustion,
        )
    }
}
