use crate::error::{EngineError, JsEngineError};
use interprete_tanques::tank_status::{Position, TankStatus};
use interprete_tanques::Interpreter;
use wasm_bindgen::prelude::*;

pub struct TanquesEngine<'a> {
    interpretes: Vec<Interpreter<'a>>,
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

    pub fn get_tank_status(&self, idx: usize) -> TankStatus {
        self.tanks_status[idx]
    }
}

impl Default for GameStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TanquesEngine<'a> {
    pub const SHOT_DAMAGE: usize = 20;
    pub const WALL_COLLISION_DAMAGE: usize = 15;
    pub const DEFAULT_COLLISION_DAMAGE: usize = 15;
    pub fn new(progs: Vec<&'a str>) -> Result<Self, EngineError> {
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
                status.set_got_shot(false);
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
            .map(|(idx, current_tank_status)| -> Option<Vec<usize>> {
                if current_tank_status.shot() {
                    // Get the tank's direction
                    let (my_i, my_j) = current_tank_status.get_pos();
                    // Get a list of the tanks it actually hit
                    let mut tanks_hit = vec![];
                    for k in 0..4 {
                        // Skip over self
                        if idx != k {
                            let (other_i, other_j) = new_status[k].get_pos();
                            let hit = match current_tank_status.get_dir() {
                                interprete_tanques::tank_status::TankDirection::West => {
                                    my_j > other_j && my_i == other_i
                                }
                                interprete_tanques::tank_status::TankDirection::North => {
                                    my_i > other_i && my_j == other_j
                                }
                                interprete_tanques::tank_status::TankDirection::East => {
                                    my_j < other_j && my_i == other_i
                                }
                                interprete_tanques::tank_status::TankDirection::South => {
                                    my_i < other_i && my_j == other_j
                                }
                            };
                            if hit {
                                tanks_hit.push(k);
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
                        new_status[*i].set_got_shot(true);
                    }
                }
                None => {}
            });

        // Apply damage from collisions with walls
        // game_status
        // .tanks_status
        // .iter()
        // .zip(new_status.iter_mut())
        // .for_each(|(old, new)| {
        // if old.get_pos() == new.get_pos() {
        // new.apply_damage(TanquesEngine::WALL_COLLISION_DAMAGE);
        // }
        // });

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
    step: usize,
    programs: Vec<String>,
    first_status_storage: Option<GameStatus>,
}

#[wasm_bindgen]
pub struct ApiInitWrapper {
    programs: Vec<String>,
}

#[wasm_bindgen]
#[allow(clippy::new_without_default)]
impl ApiInitWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            programs: vec![String::default(); 4],
        }
    }

    pub fn set_prog(&mut self, prog: String, idx: usize) {
        self.programs[idx] = prog;
    }
}

#[wasm_bindgen]
impl EngineApi {
    #[wasm_bindgen(constructor)]
    pub fn new(progs: ApiInitWrapper) -> Result<EngineApi, JsEngineError> {
        let progs = progs.programs;
        let v: Vec<&str> = progs.iter().map(|s| &**s).collect();
        TanquesEngine::new(v).map_err(JsEngineError::from_engine_error)?;
        Ok(EngineApi {
            step: 0,
            programs: progs,
            first_status_storage: None,
        })
    }

    pub fn step(&mut self, status: &GameStatus) -> Result<GameStatus, String> {
        let progs = self.programs.clone();
        let v: Vec<&str> = progs.iter().map(|s| &**s).collect();
        let mut inner_engine = TanquesEngine::new(v).map_err(|e| format!("{e}"))?;

        if self.step == 0 {
            self.first_status_storage = Some(status.clone());
        }

        let mut status = self.first_status_storage.as_ref().unwrap().to_owned();
        self.step += 1;
        for _i in 0..self.step {
            status = inner_engine.step(&status).map_err(|e| format!("{e}"))?;
        }
        Ok(status)
    }
}
