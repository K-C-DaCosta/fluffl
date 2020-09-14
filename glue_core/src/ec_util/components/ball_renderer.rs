use super::super::*;
#[derive(Default)]
pub struct BallRenderer {
    header: GameHeader,
    ball_transform_ptr: GamePointer,
}

impl BallRenderer {
    pub fn render_ball() {}
}

impl SystemComponent<GameId, GameType, GameHeader, GamePointer, GameEntity, GameState>
    for BallRenderer
{
    fn init(&mut self, state: &mut GameState, entity: &GameEntity) {
        self.ball_transform_ptr = state
            .query_entity_chain(entity, |_ptr, comp| {
                comp.get_type() == GameType::TransformComponent
            })
            .unwrap()
    }

    fn reconnect(&mut self, state: &mut GameState) {
        update_pointer_by_id(
            state,
            &mut self.ball_transform_ptr,
            "ball transform pointer not found",
        );
    }

    fn update(&mut self, _state: &mut GameState) {
        let id:u32 = self.header.get_id().into();
        println!("[id = {}],ball renderer is round af ",id);
    }

    fn get_header(&self) -> &GameHeader {
        &self.header
    }

    fn get_header_mut(&mut self) -> &mut GameHeader {
        &mut self.header
    }
    fn get_type(&self) -> GameType {
        GameType::BallRendererComponent
    }
}
