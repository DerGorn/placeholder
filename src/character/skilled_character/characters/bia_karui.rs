use threed::Vector;

use crate::{
    character::{
        skilled_character::{
            ui::CharacterGui, BAR_LOW_COLOR, BAR_PADDING, BAR_SIZE, EXHAUSTION_BAR_COLOR,
            HEALTH_BAR_COLOR, STAMINA_BAR_COLOR,
        },
        skills::{AttackSkill, HealSkill},
        Character, CharacterAlignment, NoKI, SkilledCharacter, CHARACTER_FONT_SIZE,
    },
    ui::{
        button_styles::{BackgroundImageStyle, ColorPair},
        Button, ButtonStyle, FlexItem, FontSize, ProgressBar,
    },
    BATTLE_DETAIL_OVERLAY, RESOLUTION,
};

use crate::character::CharacterBuilder;
use crate::character::CharacterGuiManager;

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
    fn create_detail_gui(&self, character: &SkilledCharacter) -> Box<CharacterGui> {
        let mut button = Button::new(
            String::new(),
            BATTLE_DETAIL_OVERLAY.into(),
            RESOLUTION,
            Vector::scalar(0.0),
            FontSize::new(CHARACTER_FONT_SIZE),
            false,
            ButtonStyle::BackgroundImage(BackgroundImageStyle {
                sprite_sheet: "characters\\bia_karui".into(),
                ..Default::default()
            }),
        );
        button.set_focus(true);
        Box::new(CharacterGui::new(
            Box::new(button),
            vec![
                Box::new(ProgressBar::new(
                    "health".into(),
                    BAR_SIZE,
                    Vector::scalar(0.0),
                    character.character.health,
                    character.character.max_health,
                    ColorPair::new(HEALTH_BAR_COLOR, BAR_LOW_COLOR),
                    BAR_PADDING,
                )),
                Box::new(ProgressBar::new(
                    "stamina".into(),
                    BAR_SIZE,
                    Vector::scalar(0.0),
                    character.character.stamina,
                    character.character.max_stamina,
                    ColorPair::new(STAMINA_BAR_COLOR, BAR_LOW_COLOR),
                    BAR_PADDING,
                )),
                Box::new(ProgressBar::new(
                    "exhaustion".into(),
                    BAR_SIZE,
                    Vector::scalar(0.0),
                    character.character.exhaustion,
                    character.character.exhaustion_threshold,
                    ColorPair::new(EXHAUSTION_BAR_COLOR, BAR_LOW_COLOR),
                    BAR_PADDING,
                )),
            ],
        ))
    }
    fn create_gui(&self, character: &SkilledCharacter) -> Box<CharacterGui> {
        Box::new(CharacterGui::with_button_style_and_character(
            ButtonStyle::BackgroundImage(BackgroundImageStyle {
                sprite_sheet: "characters\\bia_karui".into(),
                ..Default::default()
            }),
            &character.character,
        ))
    }
}
