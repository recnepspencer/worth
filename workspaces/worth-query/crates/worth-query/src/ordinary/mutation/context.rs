use crate::runtime::{
    WorthQueryOrdinaryAuthorityAdmission, WorthQueryOrdinaryAuthorityFamily,
    WorthQueryRuntimeError, WorthQueryWorkspace,
};

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryMutationContext {
    pub(crate) authority: WorthQueryOrdinaryAuthorityAdmission,
}

#[derive(Debug)]
pub struct WorthQueryMutationContextStop {
    error: WorthQueryRuntimeError,
}

impl WorthQueryMutationContextStop {
    pub fn error(&self) -> &WorthQueryRuntimeError {
        &self.error
    }

    pub fn next_action(&self) -> super::WorthQueryMutationNextAction {
        super::WorthQueryMutationNextAction::ProvideAuthority
    }
}

pub fn authoritative(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &WorthQueryWorkspace,
) -> Result<WorthQueryMutationContext, WorthQueryMutationContextStop> {
    let authority = workspace
        .capture_ordinary_mutation_authority(execution)
        .map_err(|error| WorthQueryMutationContextStop { error })?;
    debug_assert_eq!(
        authority.family(),
        WorthQueryOrdinaryAuthorityFamily::Mutation
    );
    Ok(WorthQueryMutationContext { authority })
}
