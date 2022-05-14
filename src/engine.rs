use crate::error::EngineError;
use interprete_tanques::tank_status::TankStatus;
use interprete_tanques::Interpreter;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct TanquesEngine {
    interpretes: Vec<Interpreter<'static>>,
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct GameStatus {
    tanks_status: Vec<TankStatus>,
}

impl GameStatus {
    pub fn set_tanks_status(&mut self, new_status: Vec<TankStatus>) {
        self.tanks_status = new_status;
    }
}

impl TanquesEngine {
    pub fn new(progs: Vec<&'static str>) -> Result<Self, EngineError> {
        let interpretes: Vec<Interpreter> = progs
            .iter()
            .enumerate()
            .map(|(idx, p)| Interpreter::new(p).map_err(|e| EngineError::InitError(idx, e)))
            .collect::<Result<Vec<Interpreter>, EngineError>>()?;

        Ok(TanquesEngine { interpretes })
    }

    pub fn step(&mut self, game_status: GameStatus) -> Result<GameStatus, EngineError> {
        let mut new_game_status = game_status.clone();
        let new_status: Vec<TankStatus> = self
            .interpretes
            .iter_mut()
            .enumerate()
            .zip(game_status.tanks_status)
            .map(|((idx, interprete), status)| {
                interprete
                    .step_inst(&status)
                    .map_err(|e| EngineError::RuntimeError(idx, e))
            })
            .collect::<Result<Vec<TankStatus>, EngineError>>()?;

        new_game_status.set_tanks_status(new_status);

        Ok(new_game_status)
    }
}

#[wasm_bindgen]
pub struct EngineApi {
    inner_engine: TanquesEngine,
}

#[wasm_bindgen]
pub struct ApiInitWrapper(Vec<&'static str>);

#[wasm_bindgen]
impl EngineApi {
    pub fn new(progs: ApiInitWrapper) -> Result<EngineApi, String> {
        let progs = progs.0;
        let inner_engine = TanquesEngine::new(progs).map_err(|e| format!("{e}"))?;
        Ok(EngineApi { inner_engine })
    }

    pub fn step(&mut self, status: GameStatus) -> Result<GameStatus, String> {
        let status = self.inner_engine.step(status).map_err(|e| format!("{e}"))?;
        Ok(status)
    }
}
