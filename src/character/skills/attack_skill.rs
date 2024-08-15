use super::*;

pub struct AttackSkill {}
impl Skill for AttackSkill {
    fn target_groups(&self) -> Vec<TargetGroup> {
        vec![TargetGroup::Enemies]
    }
    fn name(&self) -> SkillName {
        "Attack".into()
    }
    fn evaluate(&self, target: Option<&mut Character>, source: &mut Character) {
        let attack = source.attack;
        if let Some(target) = target {
            target.health -= attack.min(target.health);
        } else {
            source.health -= attack.min(source.health);
        }
    }
    fn get_time(&self, _target: Option<&Character>, source: &Character, current_time: f32) -> f32 {
        current_time + source.speed as f32
    }
}
