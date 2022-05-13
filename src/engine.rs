use crate::error::EngineError;
use interprete_tanques::{Interpreter, LineColLocation};

pub struct TanquesEngine<'a> {
    interpretes: Vec<Interpreter<'a>>,
}

impl TanquesEngine<'_> {
    pub fn new(progs: [&str; 4]) -> Result<Self, EngineError> {
        let interpretes: Vec<Interpreter> = progs
            .iter()
            .enumerate()
            .map(|(idx, p)| Interpreter::new(p).map_err(|e| EngineError::InitError(idx, e)))
            .collect::<Result<Vec<Interpreter>, EngineError>>()?;

        let interpretes: [Interpreter; 4] = interpretes.try_into().unwrap();

        Ok(TanquesEngine { interpretes })
    }
}
