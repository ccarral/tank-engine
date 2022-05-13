use std::error::Error;

use interprete_tanques::LineColLocation;

#[derive(Debug)]
pub enum EngineError {
    InitError(usize, LineColLocation),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::InitError(i, pos) => {
                f.write_fmt(format_args!("Error  en el interprete {i} en {pos:#?}"))
            }
        }
    }
}

impl Error for EngineError {}
