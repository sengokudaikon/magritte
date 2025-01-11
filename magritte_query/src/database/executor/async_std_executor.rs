use std::sync::Arc;
use async_channel::Sender;
use dashmap::DashMap;
use deadpool_surrealdb::Pool;
use crate::database::executor::{ExecutorConfig, ExecutorMetrics, QueryRequest};
use crate::database::runtime::RuntimeManager;
use crate::database::rw::RwLock;
#[cfg(feature = "rt-async-std")]
use async_std::sync::Notify;

#[cfg(feature = "rt-async-std")]
pub struct AsyncStdExecutor {
    config: ExecutorConfig,
    event_loops: DashMap<usize, (Sender<QueryRequest>, async_std::task::JoinHandle<()>)>,
    pool: Pool,
    runtime: Arc<RuntimeManager>,
    metrics: Arc<RwLock<ExecutorMetrics>>,
    notify_shutdown: Arc<Notify>,
}