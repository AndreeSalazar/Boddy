#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    ReadGraph,
    ReadDocuments,
    ProposePatch,
    ExecuteCode,
    Network,
}

/// Comprobación de capacidades; no constituye una frontera de seguridad del SO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    execute_code: bool,
    network: bool,
}

impl Policy {
    /// Leer contexto y proponer cambios; sin ejecución de código ni acceso a red.
    pub const fn local_research() -> Self {
        Self {
            execute_code: false,
            network: false,
        }
    }

    /// Usar solamente cuando el host tenga un ejecutor con permisos adecuados.
    pub const fn with_capabilities(execute_code: bool, network: bool) -> Self {
        Self {
            execute_code,
            network,
        }
    }

    pub const fn allows(self, tool: Tool) -> bool {
        match tool {
            Tool::ReadGraph | Tool::ReadDocuments | Tool::ProposePatch => true,
            Tool::ExecuteCode => self.execute_code,
            Tool::Network => self.network,
        }
    }
}
