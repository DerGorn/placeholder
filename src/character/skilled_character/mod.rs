use threed::Vector;
use winit::dpi::PhysicalSize;

use crate::battle_action::BattleActionManager;
use crate::ui::{Button, ButtonStyle, FontSize};
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
    fn update_gui(&self, character: &SkilledCharacter);
}

pub const CHARACTER_FONT_SIZE: u8 = 32;
pub const CHARACTER_TEXT_HEIGHT: f32 =
    (CHARACTER_DISPLAY_LINES + 0.25) * CHARACTER_FONT_SIZE as f32;
pub struct DefaultGui {}
impl CharacterGuiManager for DefaultGui {
    fn create_gui(&self, character: &SkilledCharacter) -> Box<CharacterGui> {
        Box::new(CharacterGui::new(
            Box::new(Button::new(
                character.character.to_string(),
                character.character.name.into(),
                PhysicalSize::new(400, CHARACTER_TEXT_HEIGHT as u16),
                Vector::scalar(0.0),
                FontSize::new(CHARACTER_FONT_SIZE),
                false,
                ButtonStyle::default(),
            )),
            vec![],
        ))
    }
    fn update_gui(&self, character: &SkilledCharacter) {
        println!("Updating GUI");
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
