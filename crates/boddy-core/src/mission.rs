use crate::Error;

/// El porqué, el resultado buscado y la condición de aceptación de una tarea.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mission<'a> {
    reason: &'a str,
    objective: &'a str,
    acceptance: &'a str,
}

impl<'a> Mission<'a> {
    pub fn new(reason: &'a str, objective: &'a str, acceptance: &'a str) -> Result<Self, Error> {
        if [reason, objective, acceptance]
            .iter()
            .any(|s| s.trim().is_empty())
        {
            return Err(Error::EmptyField);
        }
        Ok(Self {
            reason,
            objective,
            acceptance,
        })
    }

    pub const fn reason(&self) -> &'a str {
        self.reason
    }
    pub const fn objective(&self) -> &'a str {
        self.objective
    }
    pub const fn acceptance(&self) -> &'a str {
        self.acceptance
    }
}
