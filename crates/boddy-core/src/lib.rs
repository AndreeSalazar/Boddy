#![no_std]
//! Control determinista de tareas. No ejecuta herramientas ni contiene un modelo.
//! El host es responsable de comprobar la evidencia y aplicar el aislamiento del SO.

pub mod mission;
pub mod policy;
pub mod task;

pub use mission::Mission;
pub use policy::{Policy, Tool};
pub use task::{Error, Stage, Task, MAX_EVIDENCE};
