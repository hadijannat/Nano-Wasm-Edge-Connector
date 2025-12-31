//! Policy Runtime - Wasmtime-based policy evaluation engine

use crate::error::{ConnectorError, ConnectorResult};
use std::sync::Arc;
use wasmtime::{Caller, Config, Engine, Extern, Linker, Module, OptLevel, Store, Trap};

// Input buffer offset in Wasm memory
const INPUT_BUFFER_OFFSET: usize = 1024;
const FUEL_LIMIT: u64 = 1_000_000;

/// Host state
pub struct HostState;

/// Policy runtime managing Wasm module execution
pub struct PolicyRuntime {
    engine: Arc<Engine>,
    module: Arc<Module>,
}

impl PolicyRuntime {
    /// Create a new policy runtime from Wasm bytes
    pub fn new(wasm_bytes: &[u8]) -> ConnectorResult<Self> {
        let engine = create_edge_engine()?;
        let module = Module::new(&engine, wasm_bytes).map_err(|e| {
            ConnectorError::WasmLoadError(format!("Failed to compile module: {}", e))
        })?;

        Ok(Self {
            engine: Arc::new(engine),
            module: Arc::new(module),
        })
    }

    /// Evaluate a policy with the given request data
    pub fn evaluate_policy(&self, request_data: &[u8]) -> ConnectorResult<bool> {
        let mut store = Store::new(&self.engine, HostState);
        
        // Set fuel limit for DoS protection
        store.set_fuel(FUEL_LIMIT).map_err(|e| {
            ConnectorError::PolicyExecutionError(format!("Failed to set fuel: {}", e))
        })?;

        let mut linker: Linker<HostState> = Linker::new(&self.engine);
        
        // Register host log function - access memory via caller
        linker
            .func_wrap("host", "log", |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    let start = ptr as usize;
                    let end = start.saturating_add(len as usize).min(data.len());
                    if start < end {
                        if let Ok(msg) = std::str::from_utf8(&data[start..end]) {
                            println!("[WASM] {}", msg);
                        }
                    }
                }
            })
            .map_err(|e| ConnectorError::PolicyExecutionError(format!("Failed to register log: {}", e)))?;

        // Instantiate module
        let instance = linker.instantiate(&mut store, &self.module).map_err(|e| {
            ConnectorError::PolicyExecutionError(format!("Failed to instantiate: {}", e))
        })?;

        // Get the module's memory export
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| ConnectorError::FunctionNotFound("memory".to_string()))?;

        let input_ptr = match instance.get_typed_func::<(), i32>(&mut store, "get_input_buffer") {
            Ok(func) => func
                .call(&mut store, ())
                .map_err(|e| {
                    ConnectorError::PolicyExecutionError(format!(
                        "Failed to get input buffer: {}",
                        e
                    ))
                })? as usize,
            Err(_) => INPUT_BUFFER_OFFSET,
        };

        let required_len = input_ptr.saturating_add(request_data.len());
        if required_len > memory.data_size(&store) {
            return Err(ConnectorError::MemoryOutOfBounds { offset: input_ptr });
        }

        // Write request data to memory at the input buffer
        memory
            .write(&mut store, input_ptr, request_data)
            .map_err(|_| ConnectorError::MemoryOutOfBounds { offset: input_ptr })?;

        // Call policy evaluation with pointer and length
        let evaluate = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, "evaluate_access")
            .map_err(|e| ConnectorError::FunctionNotFound(format!("evaluate_access: {}", e)))?;

        let len_i32 = i32::try_from(request_data.len()).map_err(|_| {
            ConnectorError::PolicyExecutionError("Request too large".to_string())
        })?;

        match evaluate.call(&mut store, (input_ptr as i32, len_i32)) {
            Ok(result) => Ok(result != 0),
            Err(e) => {
                if let Some(trap) = e.downcast_ref::<Trap>() {
                    if matches!(trap, Trap::OutOfFuel) {
                        return Err(ConnectorError::FuelExhausted {
                            consumed: FUEL_LIMIT,
                        });
                    }
                }

                let err_str = format!("{}", e);
                if err_str.contains("fuel") || err_str.contains("Fuel") {
                    return Err(ConnectorError::FuelExhausted {
                        consumed: FUEL_LIMIT,
                    });
                }
                Err(ConnectorError::PolicyExecutionError(format!("Policy execution failed: {}", e)))
            }
        }
    }
}

/// Create an engine optimized for edge devices
fn create_edge_engine() -> ConnectorResult<Engine> {
    let mut config = Config::new();
    
    // Resource limiting for DoS protection
    config.consume_fuel(true);
    config.epoch_interruption(false);
    
    // Memory optimization for edge
    config.max_wasm_stack(64 * 1024);
    config.memory_guaranteed_dense_image_size(0);
    
    // Disable unused features for smaller footprint
    config.wasm_simd(false);
    config.wasm_bulk_memory(true);
    config.wasm_multi_value(true);
    config.wasm_tail_call(false);
    config.wasm_relaxed_simd(false);
    
    // Compilation optimization
    config.cranelift_opt_level(OptLevel::SpeedAndSize);
    
    Engine::new(&config).map_err(|e| ConnectorError::WasmLoadError(e.to_string()))
}
