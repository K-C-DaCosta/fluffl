use super::super::*;

pub struct BilliardBallFactory{
    pub billiard_balls:Vec<GameEntity>,
}

impl BilliardBallFactory{
    pub fn new()->Self{
        Self{
            billiard_balls:Vec::new(),
        }
    }
}

impl SystemFactory<GameId,GamePointer,GameType,GameHeader,GameEntity,GameState> for BilliardBallFactory{
    fn get_entity_table(&self) -> &Vec<GameEntity>{
        &self.billiard_balls
    }

    fn get_entity_table_mut(&mut self) -> &mut Vec<GameEntity>{
        &mut self.billiard_balls
    } 

    fn entity_constructor(&self) -> fn(&mut GameState) -> GameEntity{
        |state|{
            let mut  entity = GameEntity::new();
            state.add_component(&mut entity, GameType::TransformComponent);
            state.add_component(&mut entity, GameType::BallColliderComponent);
            state.add_component(&mut entity, GameType::BallRendererComponent);
            entity
        }
    }
}