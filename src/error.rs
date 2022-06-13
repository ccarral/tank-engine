use interprete_tanques::error::ErrorInterprete;
use interprete_tanques::LineColLocation;
use std::error::Error;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug)]
pub enum EngineError {
    InitError(usize, LineColLocation),
    RuntimeError(usize, ErrorInterprete),
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct JsEngineError {
    pub engine_idx: usize,
    pub line: usize,
}

impl JsEngineError {
    pub fn from_engine_error(e: EngineError) -> Self {
        match e {
            EngineError::InitError(engine_idx, LineColLocation::Pos((line, _col))) => {
                JsEngineError { engine_idx, line }
            }
            EngineError::InitError(_engine_idx, LineColLocation::Span(_, _)) => {
                panic!("Error de SPAN (linea 27 error.rs)");
            }
            EngineError::RuntimeError(_engine_idx, _) => todo!(),
        }
    }
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::InitError(i, pos) => {
                f.write_fmt(format_args!("Error  en el interprete {i} en {pos:#?}"))
            }
            EngineError::RuntimeError(i, e) => f.write_fmt(format_args!(
                "Error en el ejecución de internprete {i}:  en {e}"
            )),
        }
    }
}

impl std::fmt::Display for JsEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Error en el interprete {} en linea {}",
            self.engine_idx, self.line
        ))
    }
}

impl Error for EngineError {}
