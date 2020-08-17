use super::super::*;
use ec_composer::*;

#[derive(Default)]
pub struct BallCollider {
    header: GameHeader,
    ball_transform_ptr: GamePointer,
    radius:f32,
}
impl BallCollider {
    fn new() -> Self {
        Self ::default()
    }
}

impl SystemComponent<GameId, GameType, GameHeader, GamePointer, GameEntity, GameState>
    for BallCollider
{
    fn init(&mut self, state: &mut GameState, entity: &GameEntity) {
        self.ball_transform_ptr = state
            .query_entity_chain(entity, |_ptr, comp| {
                comp.get_type() == GameType::TransformComponent
            })
            .unwrap();
            
        self.radius = 32.0;
    }

    fn reconnect(&mut self, state: &mut GameState) {
        update_pointer_by_id(
            state,
            &mut self.ball_transform_ptr,
            "ball transform pointer not found",
        );
    }

    fn update(&mut self, state: &mut GameState) {
        let id:u32 = self.header.get_id().into();
        println!("[id = {}],ball colluider says hi, but does nothing else",id);
    }

    fn get_header(&self) -> &GameHeader {
        &self.header
    }

    fn get_header_mut(&mut self) -> &mut GameHeader {
        &mut self.header
    }

    fn get_type(&self) -> GameType {
        GameType::BallColliderComponent
    }
}
