//! Application-program cutoff for capability revocation operations.

use std::collections::BTreeSet;

use crate::application_capability::ErasedApplicationCapabilityContract;
use crate::application_schema::ApplicationSchemaMember;

pub(super) fn revocation_programs_are_framework_owned(
    members: &[ApplicationSchemaMember],
    contracts: &[&ErasedApplicationCapabilityContract],
) -> bool {
    let operations = contracts
        .iter()
        .filter_map(|contract| {
            contract
                .delegation()
                .revocation()
                .map(|revocation| revocation.operation().operation())
        })
        .collect::<BTreeSet<_>>();
    members.iter().all(|member| match member {
        ApplicationSchemaMember::OperationProgram { operation, .. }
        | ApplicationSchemaMember::OperationDecisionRead { operation, .. }
        | ApplicationSchemaMember::OperationDecisionReadScope { operation, .. } => {
            !operations.contains(operation.as_str())
        }
        _ => true,
    })
}
