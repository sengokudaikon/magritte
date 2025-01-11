use anyhow::Result;

/// Runtime type for query execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeType {
    /// Use Tokio with async/await
    Tokio,
    AsyncStd,
}

/// Runtime configuration for query execution
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Type of runtime to use
    pub runtime_type: RuntimeType,
    /// Number of threads to dedicate to the runtime
    pub dedicated_threads: usize,
    /// Stack size for coroutines (May only)
    pub coroutine_stack_size: usize,
    /// Maximum number of coroutines per thread
    pub max_coroutines_per_thread: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            runtime_type: RuntimeType::Tokio,
            dedicated_threads: num_cpus::get() / 4, // Use 1/4 of available cores
            coroutine_stack_size: 0x1000,           // 4KB stack size
            max_coroutines_per_thread: 1000,        // Maximum coroutines per thread
        }
    }
}

/// Runtime manager that handles thread allocation and runtime initialization
pub struct RuntimeManager {
    config: RuntimeConfig,
}

impl RuntimeManager {
    pub fn new(config: RuntimeConfig) -> Self {
        Self { config }
    }

    /// Initialize the runtime with dedicated threads
    pub fn initialize(&self) -> Result<()> {
        match self.config.runtime_type {
            RuntimeType::Tokio => Ok(()),
            RuntimeType::AsyncStd => Ok(()),
        }
    }

    /// Get the number of dedicated threads
    pub fn dedicated_threads(&self) -> usize {
        self.config.dedicated_threads
    }

    /// Get the maximum number of coroutines per thread
    pub fn max_coroutines_per_thread(&self) -> usize {
        self.config.max_coroutines_per_thread
    }
}
