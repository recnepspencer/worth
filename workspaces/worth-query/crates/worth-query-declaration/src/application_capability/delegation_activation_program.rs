//! Target-owned effect meaning for one delegation activation.

use std::collections::BTreeSet;

use crate::application_schema::ApplicationOperationProgramTarget;

use super::{
    ApplicationCapabilityFieldBinding, ApplicationCapabilityFieldDimension,
    ApplicationCapabilityRelationBinding, ApplicationCapabilityRelationDimension,
    ErasedApplicationCapabilityContract,
};

/// Derives the exact effect targets for activating one selected capability.
///
/// This is cold declaration meaning. Installation retains the result so warm
/// execution can validate the selected target without reconstructing it.
/// Only a workflow-tenure grant has a lineage to activate; a standing grant
/// yields `None` (installation also refuses a standing activation).
#[doc(hidden)]
pub fn application_capability_delegation_activation_program_targets(
    contract: &ErasedApplicationCapabilityContract,
) -> Option<Vec<ApplicationOperationProgramTarget>> {
    let activation = contract.delegation().activation()?;
    let target = contract.target();
    let constraints = contract.constraints();
    let currentness = constraints.currentness();
    let delegation = contract.delegation();
    let workflow = currentness.workflow()?;
    let lineage = delegation.lineage()?;
    let mut required = BTreeSet::from([
        ApplicationOperationProgramTarget::Create {
            entity: contract.grant_entity().to_owned(),
        },
        write_target(activation.identity()),
        write_target(target.action().field()),
        write_target(target.purpose().field()),
        write_target(currentness.active_status().field()),
        write_target(workflow.grant()),
        write_target(lineage.limit()),
        link_target(target.resource()),
        link_target(lineage.parent()),
        link_target(lineage.grantor()),
        link_target(delegation.grantee()),
    ]);
    required.extend(currentness.validity().bounds().map(write_target));
    extend_optional_field(&mut required, target.field());
    extend_optional_field(&mut required, constraints.magnitude());
    extend_optional_relation(&mut required, target.relation());
    required.extend(activation.context_relations().iter().map(link_target));
    Some(required.into_iter().collect())
}

fn write_target(field: &ApplicationCapabilityFieldBinding) -> ApplicationOperationProgramTarget {
    ApplicationOperationProgramTarget::Write {
        entity: field.entity().to_owned(),
        aspect: field.aspect().to_owned(),
        field: field.field().to_owned(),
    }
}

fn link_target(
    relation: &ApplicationCapabilityRelationBinding,
) -> ApplicationOperationProgramTarget {
    ApplicationOperationProgramTarget::Link {
        relation: relation.relation().to_owned(),
        from: relation.from().to_owned(),
        to: relation.to().to_owned(),
    }
}

fn extend_optional_field(
    required: &mut BTreeSet<ApplicationOperationProgramTarget>,
    dimension: &ApplicationCapabilityFieldDimension,
) {
    if let ApplicationCapabilityFieldDimension::Bound(field) = dimension {
        required.insert(write_target(field));
    }
}

fn extend_optional_relation(
    required: &mut BTreeSet<ApplicationOperationProgramTarget>,
    dimension: &ApplicationCapabilityRelationDimension,
) {
    if let ApplicationCapabilityRelationDimension::Bound(relation) = dimension {
        required.insert(link_target(relation));
    }
}
