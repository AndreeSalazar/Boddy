use crate::{Mission, Policy, Tool};

pub const MAX_EVIDENCE: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Created,
    Researching,
    Planned,
    Verifying,
    Completed,
    Stopped,
}

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

/// Sin heap, I/O ni dependencias. Referencias de evidencia aportadas por el host.
/// Una referencia es un localizador, no una prueba criptográfica de su contenido.
#[derive(Debug)]
pub struct Task<'a> {
    mission: Mission<'a>,
    stage: Stage,
    policy: Policy,
    tool_limit: u16,
    tool_calls: u16,
    evidence: [Option<&'a str>; MAX_EVIDENCE],
    evidence_count: usize,
    plan: Option<&'a str>,
    verification: Option<&'a str>,
}

impl<'a> Task<'a> {
    pub fn new(mission: Mission<'a>, policy: Policy, tool_limit: u16) -> Result<Self, Error> {
        if tool_limit == 0 {
            return Err(Error::InvalidBudget);
        }
        Ok(Self {
            mission,
            stage: Stage::Created,
            policy,
            tool_limit,
            tool_calls: 0,
            evidence: [None; MAX_EVIDENCE],
            evidence_count: 0,
            plan: None,
            verification: None,
        })
    }

    pub const fn mission(&self) -> Mission<'a> {
        self.mission
    }
    pub const fn stage(&self) -> Stage {
        self.stage
    }
    pub const fn tool_calls(&self) -> u16 {
        self.tool_calls
    }
    pub const fn evidence_count(&self) -> usize {
        self.evidence_count
    }
    pub const fn plan_reference(&self) -> Option<&'a str> {
        self.plan
    }
    pub const fn verification_reference(&self) -> Option<&'a str> {
        self.verification
    }

    pub fn start(&mut self) -> Result<(), Error> {
        self.require(Stage::Created)?;
        self.stage = Stage::Researching;
        Ok(())
    }

    /// Reserva una llamada ANTES de ejecutarla. Los intentos fallidos también gastan.
    pub fn request_tool(&mut self, tool: Tool) -> Result<(), Error> {
        let valid = match tool {
            Tool::ReadGraph | Tool::ReadDocuments | Tool::Network => {
                matches!(self.stage, Stage::Researching | Stage::Planned)
            }
            Tool::ProposePatch | Tool::ExecuteCode => self.stage == Stage::Planned,
        };
        if !valid {
            return Err(Error::InvalidStage);
        }
        if !self.policy.allows(tool) {
            return Err(Error::ToolDenied);
        }
        if self.tool_calls >= self.tool_limit {
            return Err(Error::BudgetExhausted);
        }
        self.tool_calls += 1;
        Ok(())
    }

    pub fn record_evidence(&mut self, reference: &'a str) -> Result<(), Error> {
        self.require(Stage::Researching)?;
        let reference = reference.trim();
        if reference.is_empty() {
            return Err(Error::EmptyField);
        }
        if self.evidence.contains(&Some(reference)) {
            return Ok(());
        }
        if self.evidence_count == MAX_EVIDENCE {
            return Err(Error::EvidenceFull);
        }
        self.evidence[self.evidence_count] = Some(reference);
        self.evidence_count += 1;
        Ok(())
    }

    pub fn set_plan(&mut self, reference: &'a str) -> Result<(), Error> {
        self.require(Stage::Researching)?;
        if reference.trim().is_empty() {
            return Err(Error::EmptyField);
        }
        if self.evidence_count == 0 {
            return Err(Error::MissingEvidence);
        }
        self.plan = Some(reference);
        self.stage = Stage::Planned;
        Ok(())
    }

    pub fn begin_verification(&mut self) -> Result<(), Error> {
        self.require(Stage::Planned)?;
        self.verification = None;
        self.stage = Stage::Verifying;
        Ok(())
    }

    /// Solo el verificador del host debe aportar el resultado; nunca el texto del LLM.
    pub fn record_verification(&mut self, passed: bool, artifact: &'a str) -> Result<(), Error> {
        self.require(Stage::Verifying)?;
        if artifact.trim().is_empty() {
            return Err(Error::EmptyField);
        }
        self.verification = if passed { Some(artifact) } else { None };
        Ok(())
    }

    pub fn revise(&mut self) -> Result<(), Error> {
        self.require(Stage::Verifying)?;
        self.verification = None;
        self.stage = Stage::Planned;
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), Error> {
        self.require(Stage::Verifying)?;
        if self.verification.is_none() {
            return Err(Error::NotVerified);
        }
        self.stage = Stage::Completed;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        if matches!(self.stage, Stage::Completed | Stage::Stopped) {
            return Err(Error::InvalidStage);
        }
        self.verification = None;
        self.stage = Stage::Stopped;
        Ok(())
    }

    fn require(&self, stage: Stage) -> Result<(), Error> {
        if self.stage == stage {
            Ok(())
        } else {
            Err(Error::InvalidStage)
        }
    }
}
