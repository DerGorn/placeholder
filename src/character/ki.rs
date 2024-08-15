use super::skills::Skill;
use crate::battle_action::{BattleAction, BattleActionManager};

use super::{Character, CharacterAlignment, SkilledCharacter};

pub trait KIBehavior {
    fn ki(
        &mut self,
        character_index: usize,
        character: &Character,
        skills: &[Box<dyn Skill>],
        action_manager: &mut BattleActionManager,
        characters: &[&SkilledCharacter],
        current_time: f32,
    );
}
pub struct NoKI;
impl KIBehavior for NoKI {
    fn ki(
        &mut self,
        _character_index: usize,
        _character: &Character,
        _skills: &[Box<dyn Skill>],
        _action_manager: &mut BattleActionManager,
        _characters: &[&SkilledCharacter],
        _current_time: f32,
    ) {
        // Do nothing
    }
}
pub struct SimpleKI;
impl KIBehavior for SimpleKI {
    fn ki(
        &mut self,
        character_index: usize,
        character: &Character,
        skills: &[Box<dyn Skill>],
        action_manager: &mut BattleActionManager,
        characters: &[&SkilledCharacter],
        current_time: f32,
    ) {
        let target_index = characters
            .iter()
            .position(|c| c.character.alignment == CharacterAlignment::Friendly)
            .expect("No friendly character found");
        let target = &characters[target_index];
        let skill_index = 0;
        let skill = skills.get(skill_index).expect("No skill found");
        let action = BattleAction::new(
            skill.get_time(Some(&target.character), &character, current_time),
            character_index,
            skill_index,
            target_index,
        );
        action_manager.queue_action(action);
    }
}
