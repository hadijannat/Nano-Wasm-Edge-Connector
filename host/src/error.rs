//! Error types for Nano-Wasm Edge Connector

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Failed to load WASM module: {0}")]
    WasmLoadError(String),

    #[error("Policy execution failed: {0}")]
    PolicyExecutionError(String),

    #[error("Fuel limit exceeded after {consumed} units")]
    FuelExhausted { consumed: u64 },

    #[error("Memory access out of bounds at offset {offset}")]
    MemoryOutOfBounds { offset: usize },

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[allow(dead_code)]
    #[error("Function signature mismatch for '{function}': expected {expected}, got {actual}")]
    SignatureMismatch {
        function: String,
        expected: String,
        actual: String,
    },

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    WasmtimeError(#[from] wasmtime::Error),
}

pub type ConnectorResult<T> = Result<T, ConnectorError>;
