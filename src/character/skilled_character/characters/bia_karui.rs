use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{
    character::{
        skilled_character::ui::CharacterGui,
        skills::{AttackSkill, HealSkill},
        Character, CharacterAlignment, NoKI, SkilledCharacter,
    },
    ui::{button_styles::BackgroundImageStyle, Button, ButtonStyle, FontSize},
};

use crate::character::CharacterBuilder;
use crate::character::CharacterGuiManager;
use crate::character::CHARACTER_FONT_SIZE;
use crate::character::CHARACTER_TEXT_HEIGHT;

const MAX_HEALTH: u16 = 100;
const MAX_STAMINA: u16 = 100;
const EXHAUSTION_THRESHOLD: u16 = 100;

#[derive(Debug)]
/// Protagonist, Bia Karui (from Bianca, latin = white, Akarui, japanese = bright)
pub struct BiaKarui {
    health: u16,
    stamina: u16,
    exhaustion: u16,
}
impl Default for BiaKarui {
    fn default() -> Self {
        Self {
            health: MAX_HEALTH,
            stamina: MAX_STAMINA,
            exhaustion: EXHAUSTION_THRESHOLD,
        }
    }
}
impl CharacterBuilder for BiaKarui {
    fn build(self) -> crate::character::SkilledCharacter {
        SkilledCharacter {
            ki: Box::new(NoKI),
            skills: vec![Box::new(AttackSkill {}), Box::new(HealSkill {})],
            character: Character {
                name: "Bia Karui",
                alignment: CharacterAlignment::Friendly,

                max_health: MAX_HEALTH,
                health: self.health,
                max_stamina: MAX_STAMINA,
                stamina: self.stamina,
                exhaustion_threshold: EXHAUSTION_THRESHOLD,
                exhaustion: self.exhaustion,

                speed: 10,
                attack: 10,
            },

            gui: Box::new(BiaKaruiGui),
        }
    }
}

pub struct BiaKaruiGui;
impl CharacterGuiManager for BiaKaruiGui {
    fn create_gui(&self, character: &SkilledCharacter) -> Box<CharacterGui> {
        Box::new(CharacterGui::new(
            Box::new(Button::new(
                character.character.to_string(),
                character.character.name.into(),
                PhysicalSize::new(400, CHARACTER_TEXT_HEIGHT as u16),
                Vector::scalar(0.0),
                FontSize::new(CHARACTER_FONT_SIZE),
                false,
                ButtonStyle::BackgroundImage(BackgroundImageStyle::default()), // ButtonStyle::default(),
            )),
            vec![],
        ))
    }
    fn update_gui(&self, character: &SkilledCharacter) {
        println!("Update Bia");
    }
}
