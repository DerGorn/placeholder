use placeholder::{
    game_engine::{EntityName, ExternalEvent, Scene, SceneName},
    graphics::{UniformBufferName, Visibility},
};

use crate::{EnemyType, Type};

#[derive(Debug)]
pub enum BattleEvent {
    FinishedPlanning,
}

#[derive(Debug)]
pub enum Event {
    EndGame,
    RequestNewScenes(Vec<Scene<Self>>),
    NewScene(SceneName),
    UpdateUniformBuffer(UniformBufferName, Vec<u8>),
    InitiateBattle(EnemyType, EntityName, SceneName),
    AnimationEnded(EntityName),
    RequestSuspendScene(SceneName),
    RequestActivateSuspendedScene(SceneName),
    RequestDeleteScene(SceneName),
    RequestDeleteEntity(EntityName, SceneName),
    RequestSetVisibilityScene(SceneName, Visibility),
    ButtonPressed(EntityName),
    BattleEvent(BattleEvent),
}
impl ExternalEvent for Event {
    type EntityType = Type;
    fn is_request_set_visibility_scene<'a>(&'a self) -> Option<(&'a SceneName, &'a Visibility)> {
        match self {
            Event::RequestSetVisibilityScene(scene, visibility) => Some((
                scene,
                visibility,
            )),
            _ => None,
        }
    }
    fn is_request_suspend_scene<'a>(&'a self) -> Option<&'a SceneName> {
        match self {
            Event::RequestSuspendScene(scene) => Some(scene),
            _ => None,
        }
    }
    fn is_request_activate_suspended_scene<'a>(&'a self) -> Option<&'a SceneName> {
        match self {
            Event::RequestActivateSuspendedScene(scene) => Some(scene),
            _ => None,
        }
    }
    fn is_request_delete_scene<'a>(&'a self) -> Option<&'a SceneName> {
        match self {
            Event::RequestDeleteScene(scene) => Some(scene),
            _ => None,
        }
    }
    fn is_request_new_scenes<'a>(&'a self) -> bool {
        match self {
            Event::RequestNewScenes(_) => true,
            _ => false,
        }
    }

    fn consume_scenes_request(self) -> Option<Vec<Scene<Self>>>
    where
        Self: Sized,
    {
        match self {
            Event::RequestNewScenes(scenes) => Some(scenes),
            _ => None,
        }
    }

    fn new_scene(scene: &Scene<Self>) -> Self
    where
        Self: Sized,
    {
        Self::NewScene(scene.name.clone())
    }

    fn is_update_uniform_buffer<'a>(
        &'a self,
    ) -> Option<(&'a placeholder::graphics::UniformBufferName, &'a [u8])> {
        match self {
            Event::UpdateUniformBuffer(name, contents) => Some((name, contents)),
            _ => None,
        }
    }
    fn is_delete_entity<'a>(&'a self) -> Option<(&'a EntityName, &'a SceneName)> {
        match self {
            Event::RequestDeleteEntity(entity, scene) => Some((entity, scene)),
            _ => None,
        }
    }

    fn is_end_game(&self) -> bool {
        matches!(self, Event::EndGame)
    }
}
