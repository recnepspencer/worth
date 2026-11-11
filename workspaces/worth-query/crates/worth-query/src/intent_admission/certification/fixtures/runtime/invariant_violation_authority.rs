use crate::facade::foundation::WorthQueryWorkspaceError;
use crate::facade::runtime::{
    WorthQueryIntentAuthorityAdapter, WorthQueryIntentDeclaration, WorthQueryIntentExecution,
};
use crate::identity::hash_parts;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::RuntimeBridge;

use super::super::certification_snapshot_identity;

pub(super) struct InvariantViolationCertificationIntentAuthority;

impl WorthQueryIntentAuthorityAdapter for InvariantViolationCertificationIntentAuthority {
    fn execute_intent(
        &mut self,
        _bridge: &RuntimeBridge,
        _relational_runtime: Option<&RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError> {
        Ok(WorthQueryIntentExecution::invariant_violation(
            declaration.strategy_name(),
            declaration.strategy_version(),
            "certification-strategy-descriptor-digest",
            declaration.input_digest(),
            hash_parts(&[
                "certification-invariant-violation".to_string(),
                declaration.name().to_string(),
            ]),
            [
                "certification-invariant:violated",
                "certification-invariant:authority-lane",
            ],
            certification_snapshot_identity("certification-invariant-snapshot"),
        ))
    }
}
