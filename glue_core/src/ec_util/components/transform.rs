use super::super::*;
#[derive(Default)]
pub struct Transform {
    header: GameHeader,
    pub x: f32,
    pub y: f32,
}

impl SystemComponent<GameId, GameType, GameHeader, GamePointer, GameEntity, GameState>
    for Transform
{
    fn init(&mut self, state: &mut GameState, entity: &GameEntity) {
        self.x = 0.0;
        self.y = 0.0;
    }
    fn reconnect(&mut self, _state: &mut GameState) {}

    fn update(&mut self, _state: &mut GameState) {
        let id: u32 = self.header.get_id().into();
        println!("[id ={}] transform: <{},{}> ", id, self.x, self.y);
    }

    fn get_header(&self) -> &GameHeader {
        &self.header
    }

    fn get_header_mut(&mut self) -> &mut GameHeader {
        &mut self.header
    }

    fn get_type(&self) -> GameType {
        GameType::TransformComponent
    }
}
