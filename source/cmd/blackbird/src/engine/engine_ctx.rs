use super::engine::EngineWindow;
use super::engine_queue::EngineQueue;
use super::entity_database::EntityDatabase;

pub struct EngineCtx {
    pub frame: usize,
    pub surface_width: usize,
    pub surface_height: usize,

    pub window: EngineWindow,
    pub queue: EngineQueue,
    pub database: EntityDatabase,
}

impl EngineCtx {}
