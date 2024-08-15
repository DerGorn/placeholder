use crate::SkilledCharacter;

#[derive(Debug)]
pub struct BattleAction {
    time: f32,
    character_index: usize,
    skill_index: usize,
    target_character_index: usize,
}
impl BattleAction {
    pub fn new(
        time: f32,
        character_index: usize,
        skill_index: usize,
        target_character_index: usize,
    ) -> Self {
        Self {
            time,
            character_index,
            skill_index,
            target_character_index,
        }
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn character_index(&self) -> usize {
        self.character_index
    }

    pub fn target_index(&self) -> usize {
        self.target_character_index
    }

    pub fn act_out(&self, characters: &mut [SkilledCharacter]) {
        let (source, target) = if self.character_index == self.target_character_index {
            let (_, right) = characters.split_at_mut(self.target_character_index);
            let (left, _) = right.split_at_mut(1);
            let source = &mut left[0];
            (source, None)
        } else {
            let (left, target) = characters.split_at_mut(self.target_character_index);
            let (target, right) = target.split_at_mut(1);
            let target = &mut target[0];
            if self.character_index < self.target_character_index {
                (&mut left[self.character_index], Some(target))
            } else {
                (
                    &mut right[self.character_index - self.target_character_index - 1],
                    Some(target),
                )
            }
        };
        source.activate_skill(self.skill_index, target)
    }
}

pub struct BattleActionManager {
    actions: Vec<BattleAction>,
}
impl BattleActionManager {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    pub fn queue_action(&mut self, action: BattleAction) {
        self.actions.push(action);
        self.actions
            .sort_by(|a, b| a.time.partial_cmp(&b.time).expect("Encountered NaN time"));
    }

    pub fn get_actions(&self) -> &[BattleAction] {
        &self.actions
    }

    pub fn contains_character(&self, character_index: usize) -> bool {
        self.actions
            .iter()
            .any(|a| a.character_index == character_index)
    }

    pub fn remove_character(&mut self, character_index: usize) {
        self.actions
            .retain(|a| a.character_index != character_index);
    }

    pub fn pop(&mut self) -> BattleAction {
        self.actions.remove(0)
    }
}
