#![forbid(unsafe_code)]

pub mod agent;
pub mod execution;
pub mod governor;
pub mod model;
pub mod roles;
pub mod runtime;
pub mod tools;
pub mod types;

pub use execution::{ExecutionEngine, ExecutionError, ExecutionOutcome};
pub use model::{model_from_env, MockModel, Model, ModelError};
pub use runtime::AgentRuntime;
pub use tools::{Tool, ToolRegistry};
pub use types::*;

#[cfg(test)]
mod tests;
