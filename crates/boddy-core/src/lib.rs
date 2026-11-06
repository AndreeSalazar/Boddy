#![no_std]
//! Orchestration facade; platform services remain external.
mod task;
pub use boddy_contracts::Error;
pub use boddy_mission::Mission;
pub use boddy_policy::{Policy, Tool};
pub use task::{Stage, Task, MAX_EVIDENCE};
