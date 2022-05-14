use interprete_tanques::error::ErrorInterprete;
use interprete_tanques::LineColLocation;
use std::error::Error;

#[derive(Debug)]
pub enum EngineError {
    InitError(usize, LineColLocation),
    RuntimeError(usize, ErrorInterprete),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::InitError(i, pos) => {
                f.write_fmt(format_args!("Error  en el interprete {i} en {pos:#?}"))
            }
            EngineError::RuntimeError(i, e) => f.write_fmt(format_args!(
                "Error en el ejecución de interprete {i}:  en {e}"
            )),
        }
    }
}

impl Error for EngineError {}
