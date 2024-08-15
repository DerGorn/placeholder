use crate::battle_action::BattleActionManager;

use super::skills::Skill;

use super::{Character, KIBehavior};

pub struct SkilledCharacter {
    pub character: Character,

    skills: Vec<Box<dyn Skill>>,
    ki: Box<dyn KIBehavior>,
}
impl SkilledCharacter {
    pub fn new(character: Character, skills: Vec<Box<dyn Skill>>, ki: Box<dyn KIBehavior>) -> Self {
        Self {
            character,
            skills,
            ki,
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
