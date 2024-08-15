use placeholder::create_name_struct;

use crate::Character;

mod attack_skill;
pub use attack_skill::AttackSkill;

mod heal_skill;
pub use heal_skill::HealSkill;

create_name_struct!(SkillName);

#[derive(Debug)]
pub enum TargetGroup {
    Ownself,
    Friends,
    Enemies,
}

pub trait Skill {
    fn name(&self) -> SkillName;
    fn evaluate(&self, target: Option<&mut Character>, source: &mut Character);
    /// Relative TargetGroups
    fn target_groups(&self) -> Vec<TargetGroup>;
    fn get_time(&self, target: Option<&Character>, source: &Character, current_time: f32) -> f32;
}
