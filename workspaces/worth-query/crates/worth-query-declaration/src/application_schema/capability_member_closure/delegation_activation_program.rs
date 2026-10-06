//! Application-program cutoff for delegation activation operations.

use std::collections::BTreeSet;

use crate::application_capability::ErasedApplicationCapabilityContract;
use crate::application_schema::ApplicationSchemaMember;

pub(super) fn activation_programs_are_framework_owned(
    members: &[ApplicationSchemaMember],
    contracts: &[&ErasedApplicationCapabilityContract],
) -> bool {
    let activation_operations = contracts
        .iter()
        .filter_map(|contract| {
            contract
                .delegation()
                .activation()
                .map(|activation| activation.operation().operation())
        })
        .collect::<BTreeSet<_>>();
    members.iter().all(|member| {
        !matches!(
            member,
            ApplicationSchemaMember::OperationProgram { operation, .. }
                if activation_operations.contains(operation.as_str())
        )
    })
}

/// Every activation keys its child grant by a unique identity field, so a
/// second activation for one child id observes the first grant and is denied.
pub(super) fn activation_identities_are_unique(
    members: &[ApplicationSchemaMember],
    contracts: &[&ErasedApplicationCapabilityContract],
) -> bool {
    contracts
        .iter()
        .filter_map(|contract| contract.delegation().activation())
        .all(|activation| {
            let identity = activation.identity();
            members.iter().any(|member| {
                matches!(
                    member,
                    ApplicationSchemaMember::Field {
                        entity,
                        aspect,
                        field,
                        unique: true,
                        ..
                    } if entity == identity.entity()
                        && aspect == identity.aspect()
                        && field == identity.field()
                )
            })
        })
}
