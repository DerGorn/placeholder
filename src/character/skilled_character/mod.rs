use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::battle_action::BattleActionManager;
use crate::color::Color;
use crate::ui::button_styles::{ColorPair, UNFOCUS_LOW_COLOR};
use crate::ui::{Button, ButtonStyle, FontSize, ProgressBar};
use crate::RESOLUTION;
use std::fmt::Debug;

use self::ui::CharacterGui;

use super::skills::Skill;

use super::{Character, KIBehavior, CHARACTER_DISPLAY_LINES};

pub mod ui;

pub mod characters;

pub trait CharacterBuilder: Default + Debug {
    fn build(self) -> SkilledCharacter;
}

pub trait CharacterGuiManager {
    fn create_gui(&self, character: &SkilledCharacter) -> Box<CharacterGui>;
}

pub const CHARACTER_FONT_SIZE: u8 = 32;
pub const CHARACTER_TEXT_HEIGHT: f32 =
    (CHARACTER_DISPLAY_LINES + 0.25) * CHARACTER_FONT_SIZE as f32;
pub const ORIGINAL_CHARACTER_PORTRAIT_SIZE: PhysicalSize<u16> = PhysicalSize::new(608, 1080);
const HEIGHT_SCALE_CHARACTER_PORTRAIT: f32 =
    ORIGINAL_CHARACTER_PORTRAIT_SIZE.height as f32 / (3.0 * RESOLUTION.height as f32);
pub const CHARACTER_PORTRAIT_SIZE: PhysicalSize<u16> = PhysicalSize::new(
    (ORIGINAL_CHARACTER_PORTRAIT_SIZE.width as f32 * HEIGHT_SCALE_CHARACTER_PORTRAIT) as u16,
    (ORIGINAL_CHARACTER_PORTRAIT_SIZE.height as f32 * HEIGHT_SCALE_CHARACTER_PORTRAIT) as u16,
);
const BAR_LOW_COLOR: Color = UNFOCUS_LOW_COLOR;
pub struct DefaultGui {}
impl CharacterGuiManager for DefaultGui {
    fn create_gui(&self, character: &SkilledCharacter) -> Box<CharacterGui> {
        Box::new(CharacterGui::new(
            Box::new(Button::new(
                character.character.name.into(),
                character.character.name.into(),
                CHARACTER_PORTRAIT_SIZE,
                Vector::scalar(0.0),
                FontSize::new(CHARACTER_FONT_SIZE),
                false,
                ButtonStyle::default(),
            )),
            vec![
                Box::new(ProgressBar::new(
                    "health".into(),
                    PhysicalSize::new((CHARACTER_PORTRAIT_SIZE.width as f32 * 0.8) as u16, 20),
                    Vector::scalar(0.0),
                    character.character.health,
                    character.character.max_health,
                    ColorPair::new(Color::new_rgba(255, 0, 0, 255), BAR_LOW_COLOR),
                    10,
                )),
                Box::new(ProgressBar::new(
                    "stamina".into(),
                    PhysicalSize::new((CHARACTER_PORTRAIT_SIZE.width as f32 * 0.8) as u16, 20),
                    Vector::scalar(0.0),
                    character.character.stamina,
                    character.character.max_stamina,
                    ColorPair::new(Color::new_rgba(255, 255, 0, 255), BAR_LOW_COLOR),
                    10,
                )),
                Box::new(ProgressBar::new(
                    "exhaustion".into(),
                    PhysicalSize::new((CHARACTER_PORTRAIT_SIZE.width as f32 * 0.8) as u16, 20),
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

pub struct SkilledCharacter {
    pub character: Character,

    skills: Vec<Box<dyn Skill>>,
    ki: Box<dyn KIBehavior>,

    pub gui: Box<dyn CharacterGuiManager>,
}
impl SkilledCharacter {
    pub fn new(character: Character, skills: Vec<Box<dyn Skill>>, ki: Box<dyn KIBehavior>) -> Self {
        Self {
            character,
            skills,
            ki,
            gui: Box::new(DefaultGui {}),
        }
    }

    pub fn get_skills(&self) -> &[Box<dyn Skill>] {
        &self.skills
    }

    pub fn activate_skill(&mut self, skill_index: usize, target: Option<&mut SkilledCharacter>) {
        self.skills[skill_index].evaluate(target.map(|c| &mut c.character), &mut self.character);
    }

    pub fn ki(
        &mut self,
        source_index: usize,
        characters: &[&SkilledCharacter],
        actions: &mut BattleActionManager,
        current_time: f32,
    ) {
        self.ki.ki(
            source_index,
            &self.character,
            &self.skills,
            actions,
            &characters,
            current_time,
        );
    }
}
impl Debug for SkilledCharacter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkilledCharacter")
            .field("character", &self.character)
            .finish()
    }
}
