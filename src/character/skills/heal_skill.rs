use super::*;

pub struct HealSkill {}
impl Skill for HealSkill {
    fn target_groups(&self) -> Vec<TargetGroup> {
        vec![TargetGroup::Friends, TargetGroup::Ownself]
    }
    fn name(&self) -> SkillName {
        "Heal".into()
    }
    fn evaluate(&self, target: Option<&mut Character>, source: &mut Character) {
        let heal = 10;
        if let Some(target) = target {
            target.health += heal.min(target.max_health - target.health);
        } else {
            source.health += heal.min(source.max_health - source.health);
        }
    }
    fn get_time(&self, _target: Option<&Character>, source: &Character, current_time: f32) -> f32 {
        current_time + source.speed as f32
    }
}
