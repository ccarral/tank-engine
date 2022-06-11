use crate::error::EngineError;
use console_error_panic_hook;
use interprete_tanques::tank_status::{Position, TankStatus};
use interprete_tanques::Interpreter;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct TanquesEngine {
    interpretes: Vec<Interpreter<'static>>,
}

#[derive(Clone)]
#[wasm_bindgen(inspectable)]
pub struct GameStatus {
    tanks_status: Vec<TankStatus>,
}

#[wasm_bindgen]
impl GameStatus {
    pub fn set_tank_status(&mut self, status: TankStatus, idx: usize) {
        self.tanks_status[idx] = status;
    }

    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            tanks_status: vec![TankStatus::default(); 4],
        }
    }

    #[wasm_bindgen(getter)]
    pub fn get_tanks_status_0(&self) -> TankStatus {
        self.tanks_status[0]
    }

    #[wasm_bindgen(getter)]
    pub fn get_tanks_status_1(&self) -> TankStatus {
        self.tanks_status[1]
    }

    #[wasm_bindgen(getter)]
    pub fn get_tanks_status_2(&self) -> TankStatus {
        self.tanks_status[2]
    }

    #[wasm_bindgen(getter)]
    pub fn get_tanks_status_3(&self) -> TankStatus {
        self.tanks_status[3]
    }
}

impl Default for GameStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl TanquesEngine {
    pub const SHOT_DAMAGE: usize = 20;
    pub const WALL_COLLISION_DAMAGE: usize = 15;
    pub const DEFAULT_COLLISION_DAMAGE: usize = 15;
    pub fn new(progs: Vec<&'static str>) -> Result<Self, EngineError> {
        let interpretes: Vec<Interpreter> = progs
            .iter()
            .enumerate()
            .map(|(idx, p)| Interpreter::new(p).map_err(|e| EngineError::InitError(idx, e)))
            .collect::<Result<Vec<Interpreter>, EngineError>>()?;

        Ok(TanquesEngine { interpretes })
    }

    pub fn step(&mut self, game_status: &GameStatus) -> Result<GameStatus, EngineError> {
        let mut new_game_status = game_status.clone();

        let mut new_status: Vec<TankStatus> = self
            .interpretes
            .iter_mut()
            .enumerate()
            .zip(new_game_status.tanks_status.iter_mut())
            .map(|((idx, interprete), status)| {
                // Clear last shot status
                status.set_shot(false);
                interprete
                    .step_inst(status)
                    .map_err(|e| EngineError::RuntimeError(idx, e))
            })
            .collect::<Result<Vec<TankStatus>, EngineError>>()?;

        // Check if a tank just shot, and if that shot hit another tank that was in its line of
        // sight, decrease the tanks health status
        let shot_damage_list: Vec<Option<Vec<usize>>> = new_status
            .iter()
            .enumerate()
            .map(|(idx, ts)| -> Option<Vec<usize>> {
                if ts.shot() {
                    // Get the tank's direction
                    let (my_x, my_y) = ts.get_pos();
                    // Get a list of the tanks it actually hit
                    let mut tanks_hit = vec![];
                    for i in 0..4 {
                        // Skip over self
                        if idx != i {
                            let (other_x, other_y) = new_status[i].get_pos();
                            let hit = match ts.get_dir() {
                                interprete_tanques::tank_status::TankDirection::North => {
                                    my_y > other_y && my_x == other_x
                                }
                                interprete_tanques::tank_status::TankDirection::West => {
                                    my_x > other_x && my_y == other_y
                                }
                                interprete_tanques::tank_status::TankDirection::South => {
                                    my_y < other_y && my_x == other_x
                                }
                                interprete_tanques::tank_status::TankDirection::East => {
                                    my_x < other_x && my_y == other_y
                                }
                            };
                            if hit {
                                tanks_hit.push(i);
                            }
                        }
                    }
                    Some(tanks_hit)
                } else {
                    None
                }
            })
            .collect();

        // Apply damage from shots
        shot_damage_list
            .iter()
            .for_each(|damage_item| match damage_item {
                Some(damaged_tanks) => {
                    for i in damaged_tanks {
                        new_status[*i].apply_damage(TanquesEngine::SHOT_DAMAGE);
                    }
                }
                None => {}
            });

        // Apply damage from collisions with walls
        game_status
            .tanks_status
            .iter()
            .zip(new_status.iter_mut())
            .for_each(|(old, new)| {
                if old.get_pos() == new.get_pos() {
                    new.apply_damage(TanquesEngine::WALL_COLLISION_DAMAGE);
                }
            });

        // Apply damage from collisions with other tanks
        let collision_other_tanks: Vec<bool> = new_status
            .iter()
            .enumerate()
            .map(|(i, my_status)| {
                let mut collision = false;
                for (j, other_status) in new_status.iter().enumerate() {
                    if i != j && my_status.get_pos() == other_status.get_pos() {
                        collision = true;
                        break;
                    }
                }
                collision
            })
            .collect();

        for (collided, new_stat) in collision_other_tanks.iter().zip(new_status.iter_mut()) {
            if *collided {
                new_stat.apply_damage(TanquesEngine::DEFAULT_COLLISION_DAMAGE);
            }
        }

        new_game_status.set_tank_status(new_status[0], 0);
        new_game_status.set_tank_status(new_status[1], 1);
        new_game_status.set_tank_status(new_status[2], 2);
        new_game_status.set_tank_status(new_status[3], 3);

        Ok(new_game_status)
    }
}

#[wasm_bindgen]
pub struct EngineApi {
    inner_engine: TanquesEngine,
}

#[wasm_bindgen]
pub struct ApiInitWrapper {
    programs: Vec<&'static str>,
}

#[wasm_bindgen]
impl ApiInitWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            programs: vec![
                "var x = 4; mientras ( x > 0 ){ si( radar == 0 ){ gira derecha; }otro{ avanza; }}",
                "var y = 10;",
                "var z = 3;",
                "var w = 0;",
            ],
        }
    }
}

#[wasm_bindgen]
impl EngineApi {
    #[wasm_bindgen(constructor)]
    pub fn new(progs: ApiInitWrapper) -> Result<EngineApi, String> {
        let progs = progs.programs;
        let inner_engine = TanquesEngine::new(progs).map_err(|e| format!("{e}"))?;
        Ok(EngineApi { inner_engine })
    }

    pub fn step(&mut self, status: &GameStatus) -> Result<GameStatus, String> {
        let status = self.inner_engine.step(status).map_err(|e| format!("{e}"))?;
        Ok(status)
    }
}
