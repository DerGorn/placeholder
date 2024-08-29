use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::{
    character::{
        skilled_character::{ui::CharacterGui, BAR_LOW_COLOR, CHARACTER_PORTRAIT_SIZE},
        skills::{AttackSkill, HealSkill},
        Character, CharacterAlignment, NoKI, SkilledCharacter,
    }, color::Color, ui::{button_styles::{BackgroundImageStyle, ColorPair}, Button, ButtonStyle, FontSize, ProgressBar}
};

use crate::character::CharacterBuilder;
use crate::character::CharacterGuiManager;
use crate::character::CHARACTER_FONT_SIZE;

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
        let bar_size = PhysicalSize::new((CHARACTER_PORTRAIT_SIZE.width as f32 * 0.8) as u16, 20);
        Box::new(CharacterGui::new(
            Box::new(Button::new(
                String::new(),
                character.character.name.into(),
                PhysicalSize::new(203, 360),
                Vector::scalar(0.0),
                FontSize::new(CHARACTER_FONT_SIZE),
                false,
                ButtonStyle::BackgroundImage(BackgroundImageStyle {
                    sprite_sheet: "characters\\bia_karui".into(),
                    ..Default::default()
                }),
            )),
            vec![
                Box::new(ProgressBar::new(
                    "health".into(),
                    bar_size,
                    Vector::scalar(0.0),
                    character.character.health,
                    character.character.max_health,
                    ColorPair::new(Color::new_rgba(255, 0, 0, 255), BAR_LOW_COLOR),
                    10,
                )),
                Box::new(ProgressBar::new(
                    "stamina".into(),
                    bar_size,
                    Vector::scalar(0.0),
                    character.character.stamina,
                    character.character.max_stamina,
                    ColorPair::new(Color::new_rgba(255, 255, 0, 255), BAR_LOW_COLOR),
                    10,
                )),
                Box::new(ProgressBar::new(
                    "exhaustion".into(),
                    bar_size,
                    Vector::scalar(0.0),
                    character.character.exhaustion,
                    character.character.exhaustion_threshold,
                    ColorPair::new(Color::new_rgba(0, 255, 255, 255), BAR_LOW_COLOR),
                    10,
                )),
            ],
        ))
    }
}
