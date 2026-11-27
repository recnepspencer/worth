//! Named installation refusals for grant tenure, extent liveness, the
//! elevation timeline and declarative bodies that would grant to themselves.
use std::collections::BTreeSet;

use crate::application_capability::{
    ApplicationCapabilityCurrentnessDefinition, ApplicationCapabilityDelegationRule,
    ApplicationCapabilityElevationRule, ApplicationCapabilitySelfGrantRule,
    ErasedApplicationCapabilityContract,
};
use crate::application_schema::{
    ApplicationAuthorizationTraversalDirection as Direction,
    ApplicationOperationBodyCreationLinkSource, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember,
};

pub(super) fn validate_tenure(
    contract: &ErasedApplicationCapabilityContract,
) -> Result<(), Denial> {
    let capability = || contract.name().to_owned();
    if !tenure_is_coherent(contract) {
        return Err(Denial::InvalidApplicationCapabilityTenure {
            capability: capability(),
        });
    }
    if !extent_status_is_reachable(contract) {
        return Err(Denial::InvalidApplicationCapabilityExtentStatus {
            capability: capability(),
        });
    }
    if !open_validity_is_standing(contract) {
        return Err(Denial::OpenApplicationCapabilityValidity {
            capability: capability(),
        });
    }
    if !elevation_timeline_is_supported(contract) {
        return Err(Denial::UnsupportedApplicationCapabilityElevationTimeline {
            capability: capability(),
        });
    }
    Ok(())
}

/// A declarative body may not link its own admitted principal through a
/// grantee relation whose grant kind forbids self-grant.
pub(super) fn validate_self_granting_bodies(
    members: &[ApplicationSchemaMember],
    contracts: &[&ErasedApplicationCapabilityContract],
) -> Result<(), Denial> {
    let forbidden = self_grant_forbidden_relations(contracts);
    if forbidden.is_empty() {
        return Ok(());
    }
    for member in members {
        let ApplicationSchemaMember::OperationBody { operation, body } = member else {
            continue;
        };
        let self_granting = body
            .creation()
            .into_iter()
            .flat_map(|creation| creation.creation_roots())
            .flat_map(|creation| creation.links())
            .any(|link| {
                matches!(
                    link.source,
                    ApplicationOperationBodyCreationLinkSource::Principal { .. }
                ) && forbidden.contains(&(
                    link.relation.as_str(),
                    link.from.as_str(),
                    link.to.as_str(),
                ))
            });
        if self_granting {
            return Err(Denial::SelfGrantingOperationBody {
                operation: operation.clone(),
            });
        }
    }
    Ok(())
}

fn self_grant_forbidden_relations<'contract>(
    contracts: &[&'contract ErasedApplicationCapabilityContract],
) -> BTreeSet<(&'contract str, &'contract str, &'contract str)> {
    contracts
        .iter()
        .map(|contract| contract.delegation())
        .filter(|delegation| {
            delegation.self_grant() == ApplicationCapabilitySelfGrantRule::Forbidden
        })
        .map(|delegation| {
            let grantee = delegation.grantee();
            (grantee.relation(), grantee.from(), grantee.to())
        })
        .collect()
}

/// Workflow binding and lineage appear together; a standing grant is never
/// delegated or activated, and only a standing grant may forbid self-grant.
fn tenure_is_coherent(contract: &ErasedApplicationCapabilityContract) -> bool {
    let delegation = contract.delegation();
    let standing = contract.constraints().currentness().workflow().is_none();
    if standing != delegation.lineage().is_none() {
        return false;
    }
    if !standing {
        return delegation.self_grant() == ApplicationCapabilitySelfGrantRule::Permitted;
    }
    delegation.activation().is_none()
        && contract.composition().propagation().delegation()
            == ApplicationCapabilityDelegationRule::Forbidden
}

/// Every extent kind is distinct, is not the grant itself, and is crossed by
/// the grant path at its head (or root) or a later resource-path step.
fn extent_status_is_reachable(contract: &ErasedApplicationCapabilityContract) -> bool {
    let statuses = contract.constraints().currentness().extent_status();
    if statuses.len() > ApplicationCapabilityCurrentnessDefinition::MAXIMUM_EXTENT_STATUS {
        return false;
    }
    let target = contract.target();
    let crossed = std::iter::once(target.resource().to())
        .chain(
            target
                .resource_path()
                .traversals()
                .iter()
                .map(|step| match step.direction() {
                    Direction::Forward => step.to(),
                    Direction::Reverse => step.from(),
                }),
        )
        .collect::<BTreeSet<_>>();
    let mut kinds = BTreeSet::new();
    statuses.iter().all(|status| {
        let entity = status.field().entity();
        entity != contract.grant_entity() && crossed.contains(entity) && kinds.insert(entity)
    })
}

/// An open validity bound belongs to a standing grant: a workflow-tenure
/// grant may be delegated, and a child's window is proven inside its
/// parent's only when both windows are closed.
fn open_validity_is_standing(contract: &ErasedApplicationCapabilityContract) -> bool {
    let currentness = contract.constraints().currentness();
    currentness.validity().is_closed() || currentness.workflow().is_none()
}

fn elevation_timeline_is_supported(contract: &ErasedApplicationCapabilityContract) -> bool {
    match contract.elevation() {
        ApplicationCapabilityElevationRule::Governed(elevation) => {
            elevation.validity().is_closed()
                && contract.constraints().currentness().validity().is_closed()
                && elevation.validity().timeline().counts_unix_epoch_units()
                && contract
                    .constraints()
                    .currentness()
                    .validity()
                    .timeline()
                    .counts_unix_epoch_units()
        }
        _ => true,
    }
}
