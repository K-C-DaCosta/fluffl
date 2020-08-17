pub mod components;
pub mod factories;

use ec_composer::*;
use std::cell::*;
use std::ops;
use std::rc::*;

use self::components::{ball_collider::*, ball_renderer::*, transform::*};

use self::factories::billiard_ball::*;

type ComponentList<State> =
    dyn SystemComponentList<GameId, GameType, GameHeader, GamePointer, GameEntity, State>;
type Factory<State> =
    dyn SystemFactory<GameId, GamePointer, GameType, GameHeader, GameEntity, State>;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct GameId {
    id: u32,
}
impl ops::AddAssign for GameId {
    fn add_assign(&mut self, rhs: GameId) {
        self.id += rhs.id;
    }
}
impl From<u32> for GameId {
    fn from(id: u32) -> Self {
        Self { id }
    }
}
impl Into<u32> for GameId {
    fn into(self) -> u32 {
        self.id
    }
}
#[derive(Copy, Clone, Default)]
pub struct GamePointer {
    row: u32,
    col: u32,
    id: GameId,
}
impl SystemPtr<GameId> for GamePointer {
    fn set_ptr(&mut self, table_loc: (u32, u32), id: GameId) {
        let (tr, tc) = table_loc;
        self.row = tr;
        self.col = tc;
        self.id = id;
    }
    fn get_id_part(&self) -> u32 {
        self.id.into()
    }
    fn get_index_pair_part(&self) -> (u32, u32) {
        (self.row, self.col)
    }
}

#[derive(Default)]
pub struct GameHeader {
    id: GameId,
}

impl SystemHeader<GameId> for GameHeader {
    fn get_id(&self) -> GameId {
        self.id
    }
    unsafe fn set_id(&mut self, new_id: GameId) {
        self.id = new_id;
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum GameType {
    Unknown = 0,
    BallColliderComponent,
    BallRendererComponent,
    TransformComponent,
    BilliardBallEntity,
}

pub struct GameEntity {
    pointer_list: Vec<GamePointer>,
    header: GameHeader,
    entity_type: GameType,
}
impl GameEntity {
    fn new() -> Self {
        Self {
            pointer_list: Vec::new(),
            header: GameHeader::default(),
            entity_type: GameType::Unknown,
        }
    }
}

impl SystemEntity<GameId, GamePointer, GameHeader, GameType, GameState> for GameEntity {
    fn get_component_chain(&self) -> &Vec<GamePointer> {
        &self.pointer_list
    }
    fn get_component_chain_mut(&mut self) -> &mut Vec<GamePointer> {
        &mut self.pointer_list
    }
    fn get_header(&self) -> &GameHeader {
        &self.header
    }
    fn get_header_mut(&mut self) -> &mut GameHeader {
        &mut self.header
    }
    fn get_type(&self) -> GameType {
        self.entity_type
    }
}

pub struct GameData {
    pub ball_collider: Vec<BallCollider>,
    pub ball_renderer: Vec<BallRenderer>,
    pub transform: Vec<Transform>,
    pub billiardball_factory: BilliardBallFactory,
    pub comp_id_ticker: GameId,
    pub entity_id_ticker: GameId,
}

impl GameData {
    pub fn new() -> Rc<RefCell<Box<Self>>> {
        Rc::new(RefCell::new(Box::new(Self {
            ball_collider: Vec::new(),
            ball_renderer: Vec::new(),
            transform: Vec::new(),
            billiardball_factory: BilliardBallFactory::new(),
            comp_id_ticker: GameId::default(),
            entity_id_ticker: GameId::default(),
        })))
    }
}

impl SystemData<GameType, GameHeader, GameId, GameEntity, GamePointer, GameState>
    for Rc<RefCell<Box<GameData>>>
{
    fn init_tables(&self) -> GameState {
        let data_ref = &mut *self.borrow_mut();
        GameState {
            comp_table: vec![
                (
                    GameType::TransformComponent,
                    (&mut data_ref.transform) as *mut ComponentList<GameState>,
                ),
                (
                    GameType::BallRendererComponent,
                    (&mut data_ref.ball_renderer) as *mut ComponentList<GameState>,
                ),
                (
                    GameType::BallColliderComponent,
                    (&mut data_ref.ball_collider) as *mut ComponentList<GameState>,
                ),
            ],
            factory_table: vec![(&mut data_ref.billiardball_factory) as *mut Factory<GameState>],
            game_data_ref: Some(self.clone()),
        }
    }

    fn gen_comp_id(&mut self) -> GameId {
        let data_borrow = &mut *self.borrow_mut(); 
        let id = data_borrow.comp_id_ticker;
        data_borrow.comp_id_ticker += GameId::from(1);
        id
    }

    fn gen_entity_id(&mut self) -> GameId {
        let data_borrow = &mut *self.borrow_mut(); 
        let id = data_borrow.entity_id_ticker;
        data_borrow.entity_id_ticker += GameId::from(1);
        id
    }
}

pub struct GameState {
    pub comp_table: Vec<(GameType, *mut ComponentList<Self>)>,
    pub factory_table: Vec<*mut Factory<Self>>,
    pub game_data_ref: Option<Rc<RefCell<Box<GameData>>>>,
}

impl SystemState<GameType, GameHeader, GameId, GameEntity, GamePointer> for GameState {
    fn gen_comp_id(&mut self) -> GameId {
        self.game_data_ref.as_mut().unwrap().gen_comp_id()
    }

    fn gen_entity_id(&mut self) -> GameId {
        self.game_data_ref.as_mut().unwrap().gen_entity_id()
    }

    fn get_comp_table(&self) -> &Vec<(GameType, *mut ComponentList<Self>)> {
        &self.comp_table
    }

    fn get_comp_table_mut(&mut self) -> &mut Vec<(GameType, *mut ComponentList<Self>)> {
        &mut self.comp_table
    }

    fn get_factory_table(&self) -> &Vec<*mut Factory<Self>> {
        &self.factory_table
    }

    fn get_factory_table_mut(&mut self) -> &mut Vec<*mut Factory<Self>> {
        &mut self.factory_table
    }
}

#[allow(dead_code)]
pub struct DataBorrow;
impl <Mutator> DataBorrower<Rc<RefCell<Box<GameData>>>,Box<GameData>,Mutator> for DataBorrow
where Mutator: FnMut(&mut Box<GameData>),
{
    fn borrow_mut(data_in: &Rc<RefCell<Box<GameData>>>,mut  mutator: Mutator) {
        let mut_borrow = &mut *data_in.borrow_mut();
        mutator(mut_borrow);
    }
}