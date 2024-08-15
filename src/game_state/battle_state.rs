use crate::{battle_action::BattleActionManager, SkilledCharacter};

pub struct BattleState {
    pub characters: Vec<SkilledCharacter>,
    pub current_time: f32,
    pub actions: BattleActionManager,
}
