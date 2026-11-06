#![no_std]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    EmptyField,
    InvalidBudget,
    InvalidStage,
    MissingEvidence,
    EvidenceFull,
    ToolDenied,
    BudgetExhausted,
    NotVerified,
}
