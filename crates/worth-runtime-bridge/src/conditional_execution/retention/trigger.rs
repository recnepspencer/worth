use super::semantic_payload::{self as payload, visit};
use super::{
    arc_slice_charge, array_charge, sum, BridgeRetentionDenial as D, BridgeRetentionLedger,
    BridgeRetentionReservation,
};
use crate::correspondence::{
    BridgeCorrespondenceBasis, BridgeDeliveredCorrespondenceChange,
    BridgeDeliveredCorrespondenceChangeSet, BridgeSemanticDependencyCandidate,
    BridgeSemanticLocality,
};
use std::sync::Arc;

/// A prepared immutable trigger stays in the shared decision core through every
/// evidence, seed and reentry. Construction precedes provider or Signal work.
pub(in crate::conditional_execution) struct BridgeRetainedTrigger {
    change_set: BridgeDeliveredCorrespondenceChangeSet,
    _reservation: BridgeRetentionReservation,
}

impl BridgeRetainedTrigger {
    pub(in crate::conditional_execution) fn prepare(
        ledger: &Arc<BridgeRetentionLedger>,
        change_set: &BridgeDeliveredCorrespondenceChangeSet,
    ) -> Result<Self, super::super::BridgeConditionalDenial> {
        let mut work = ledger.maximum_preparation_visits();
        let charge = charge(change_set, &mut work)
            .map_err(super::super::observation_retention::retention_denial)?;
        let reservation = ledger
            .reserve(0, 0, charge)
            .map_err(super::super::observation_retention::retention_denial)?;
        Ok(Self {
            change_set: change_set.clone(),
            _reservation: reservation,
        })
    }

    pub(in crate::conditional_execution) fn retains_same_delivery_as(
        &self,
        candidate: &BridgeDeliveredCorrespondenceChangeSet,
    ) -> bool {
        self.change_set.retains_same_delivery_as(candidate)
    }
}

fn charge(change_set: &BridgeDeliveredCorrespondenceChangeSet, work: &mut usize) -> Result<u64, D> {
    let mut size = sum(&[
        // The inline trigger and reservation are embedded in the decision core.
        // Only separately allocated buffers are incremental retained storage.
        source_identities(change_set.basis(), change_set.dependency(), work)?,
        basis(change_set.basis(), work)?,
        dependency(change_set.dependency(), work)?,
        // Other delivered identities may share backing, but their construction
        // does not promise it. Reserve their independent-allocation upper bound;
        // no payload-wide alias registry or quadratic identity scan is retained.
        identity(change_set.commit_identity())?,
        identity(change_set.patch_identity())?,
        identity(change_set.snapshot_identity())?,
        identity(change_set.branch_identity())?,
        array_charge::<BridgeDeliveredCorrespondenceChange>(change_set.changes().len())?,
    ])?;
    for change in change_set.changes() {
        visit(work)?;
        if let Some(entity) = change.entity_identity() {
            size = sum(&[size, arc_slice_charge::<u8>(entity.len())?])?;
        }
        if let Some(change) = change.semantic_change() {
            size = sum(&[
                size,
                payload::bytes(change.aspect_key().as_str().len())?,
                payload::binding(change.binding())?,
            ])?;
            if let Some(path) = change.field_path() {
                size = sum(&[size, payload::path(path, work)?])?;
            }
        }
    }
    Ok(size)
}

fn identity<Tag>(identity: &crate::identity::BridgeIdentity<Tag>) -> Result<u64, D> {
    let size = arc_slice_charge::<u8>(identity.as_str().len())?;
    match identity.payload() {
        crate::identity::BridgeIdentityPayload::RelationalBranch { branch_id } => {
            sum(&[size, arc_slice_charge::<u8>(branch_id.len())?])
        }
        _ => Ok(size),
    }
}

fn source_identities(
    basis: &BridgeCorrespondenceBasis,
    dependency: &BridgeSemanticDependencyCandidate,
    work: &mut usize,
) -> Result<u64, D> {
    // Correspondence resolution carries these six source identities into the
    // dependency by Arc clone. Compare allocation identity in this fixed set:
    // separately allocated equal text remains separate retained storage.
    let texts = [
        &basis.source_installation_identity,
        &basis.source_basis,
        &basis.source_authority_binding_identity,
        &basis.declared_graph_role,
        &basis.graph_participation_identity,
        &basis.graph_adapter_identity,
        &dependency.source_installation_identity,
        &dependency.source_basis,
        &dependency.source_authority_binding_identity,
        &dependency.declared_graph_role,
        &dependency.graph_participation_identity,
        &dependency.graph_adapter_identity,
    ];
    let mut size = 0;
    for (index, text) in texts.iter().enumerate() {
        visit(work)?;
        if !texts[..index].iter().any(|prior| Arc::ptr_eq(prior, text)) {
            size = sum(&[size, arc_slice_charge::<u8>(text.len())?])?;
        }
    }
    Ok(size)
}

fn basis(basis: &BridgeCorrespondenceBasis, work: &mut usize) -> Result<u64, D> {
    let mut size = 0;
    if let Some(profile) = &basis.authoritative_source_profile {
        visit(work)?;
        size = sum(&[
            size,
            arc_slice_charge::<u8>(profile.adapter_semantic_identity().len())?,
        ])?;
    }
    size = sum(&[
        size,
        array_charge::<worth_signal::facade::PartitionToken>(basis.signal_partitions.len())?,
    ])?;
    for partition in &basis.signal_partitions {
        visit(work)?;
        size = sum(&[size, payload::bytes(partition.0.len())?])?;
    }
    Ok(size)
}

fn dependency(dependency: &BridgeSemanticDependencyCandidate, work: &mut usize) -> Result<u64, D> {
    visit(work)?;
    let mut size = arc_slice_charge::<u8>(dependency.source_node_identity.len())?;
    if let Some(stage) = &dependency.source_stage_identity {
        visit(work)?;
        size = sum(&[size, arc_slice_charge::<u8>(stage.len())?])?;
    }
    if let BridgeSemanticLocality::SourcePartition(partition) = &dependency.locality {
        visit(work)?;
        size = sum(&[size, payload::bytes(partition.as_str().len())?])?;
    }
    for _ in &dependency.relevant_changes {
        visit(work)?;
    }
    sum(&[
        size,
        payload::contract(&dependency.contract, work)?,
        payload::mask(&dependency.projection_mask, work)?,
        payload::binding(&dependency.binding)?,
        array_charge::<worth_foundational::facade::AuthoritativeAspectChangeKind>(
            dependency.relevant_changes.len(),
        )?,
    ])
}
