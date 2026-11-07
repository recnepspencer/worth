use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::ordinary::WorthQueryOrdinaryInspectionPolicy;
use crate::runtime::{
    WorthQueryAspectMutationBuilder, WorthQueryRuntimeError, WorthQueryWriteCommand,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationDeclarationIdentity {
    identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryMutationDeclarationIdentity {
    pub fn as_str(&self) -> &str {
        self.identity.as_str()
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryMutationDeclaration {
    identity: WorthQueryMutationDeclarationIdentity,
    command: WorthQueryWriteCommand,
    inspection_policy: WorthQueryOrdinaryInspectionPolicy,
}

impl WorthQueryMutationDeclaration {
    pub fn identity(&self) -> &WorthQueryMutationDeclarationIdentity {
        &self.identity
    }

    pub fn inspection_policy(&self) -> WorthQueryOrdinaryInspectionPolicy {
        self.inspection_policy
    }

    pub fn with_rich_inspection(mut self) -> Self {
        self.inspection_policy = WorthQueryOrdinaryInspectionPolicy::Rich;
        self
    }

    pub(crate) fn into_parts(self) -> (WorthQueryWriteCommand, WorthQueryOrdinaryInspectionPolicy) {
        (self.command, self.inspection_policy)
    }
}

#[derive(Debug)]
pub struct WorthQueryMutationDeclarationStop {
    error: Box<WorthQueryRuntimeError>,
}

impl WorthQueryMutationDeclarationStop {
    pub fn error(&self) -> &WorthQueryRuntimeError {
        &self.error
    }

    pub fn next_action(&self) -> super::WorthQueryMutationNextAction {
        super::WorthQueryMutationNextAction::ReviseDeclaration
    }
}

pub fn declare<E>(
    author: impl FnOnce(WorthQueryAspectMutationBuilder) -> Result<WorthQueryWriteCommand, E>,
) -> Result<WorthQueryMutationDeclaration, WorthQueryMutationDeclarationStop>
where
    E: Into<Box<WorthQueryRuntimeError>>,
{
    let command = author(WorthQueryAspectMutationBuilder::new()).map_err(|error| {
        WorthQueryMutationDeclarationStop {
            error: error.into(),
        }
    })?;
    let identity = mutation_declaration_identity(&command).map_err(|cause| {
        WorthQueryMutationDeclarationStop {
            error: Box::new(cause.into()),
        }
    })?;
    Ok(WorthQueryMutationDeclaration {
        identity: WorthQueryMutationDeclarationIdentity { identity },
        command,
        inspection_policy: WorthQueryOrdinaryInspectionPolicy::OperationalOnly,
    })
}

fn mutation_declaration_identity(
    command: &WorthQueryWriteCommand,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(crate::intent_admission::authoritative_mutation_input_identity(command)?)
}
