use std::sync::Arc;
use anyhow::Result;
use may::config::{Config as MayConfig, GuardContext};
use parking_lot::RwLock;

/// Runtime type for query execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeType {
    /// Use May runtime with coroutines
    May,
    /// Use Tokio with async/await
    Tokio,
    AsyncStd,
    /// Use Glommio with io_uring
    Glommio,
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
            runtime_type: RuntimeType::May,
            dedicated_threads: num_cpus::get() / 4, // Use 1/4 of available cores
            coroutine_stack_size: 0x1000, // 4KB stack size
            max_coroutines_per_thread: 1000, // Maximum coroutines per thread
        }
    }
}

/// Runtime manager that handles thread allocation and runtime initialization
pub struct RuntimeManager {
    config: RuntimeConfig,
    may_config: Arc<RwLock<Option<MayConfig>>>,
}

impl RuntimeManager {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            may_config: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Initialize the runtime with dedicated threads
    pub fn initialize(&self) -> Result<()> {
        match self.config.runtime_type {
            RuntimeType::May => {
                // Configure May runtime for cooperative scheduling
                let mut config = MayConfig::default();
                
                // Set number of worker threads
                config.set_workers(self.config.dedicated_threads);
                
                // Set stack size for coroutines
                config.set_stack_size(self.config.coroutine_stack_size);
                
                // Disable work stealing for cooperative scheduling
                config.disable_work_stealing();
                
                // Set scheduling parameters for cooperative mode
                config.set_io_workers(self.config.dedicated_threads);
                config.set_pool_capacity(self.config.max_coroutines_per_thread * self.config.dedicated_threads);
                
                // Store config for later use
                *self.may_config.write() = Some(config.clone());
                
                // Initialize May runtime
                may::config().set_config(&config);
                Ok(())
            }
            RuntimeType::Tokio => {
                // For Tokio, we'll just use the existing runtime
                // but limit the number of threads in the executor
                Ok(())
            }
            RuntimeType::Glommio => {
                // For Glommio, configure thread-per-core
                Ok(())
            }
        }
    }
    
    /// Get the May runtime configuration if available
    pub fn may_config(&self) -> Option<MayConfig> {
        self.may_config.read().clone()
    }
    
    /// Create a new coroutine guard for May runtime
    pub fn create_guard(&self) -> Option<GuardContext> {
        match self.config.runtime_type {
            RuntimeType::May => Some(GuardContext::new()),
            _ => None,
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