use super::integrity_classification::{
    IntegrityRepairClassificationPlan, IntegrityRepairRegionClass,
};
use crate::workflow::restore::BackupRestoreReplayPlan;
use sha2::{Digest, Sha256};
use worth_store_physical_backend::LoweredNonCurrentStagingPlan;

use crate::owner_plan_dag::{
    CanonicalOwnerPlanDag, OwnerPlanEffect, OwnerPlanFootprint, OwnerPlanNode,
    OwnerPlanNodeIdentity, OwnerPlanPrerequisite, StoreOwnerKind,
};

use super::authority_affecting::AuthorityAffectingRepairLoweringDenial;

#[derive(Debug, Clone, Copy)]
pub(super) struct RepairOwnerNodes {
    pub(super) integrity: OwnerPlanNodeIdentity,
    pub(super) backend: OwnerPlanNodeIdentity,
    pub(super) recovery: OwnerPlanNodeIdentity,
    pub(super) layout: Option<OwnerPlanNodeIdentity>,
    pub(super) blob: Option<OwnerPlanNodeIdentity>,
}

impl RepairOwnerNodes {
    pub(super) const fn count(self) -> u64 {
        3 + self.layout.is_some() as u64 + self.blob.is_some() as u64
    }

    pub(super) fn admits_recovered_receipts(
        self,
        receipts: &[crate::RecoveredRepairOwnerReceipt],
    ) -> bool {
        receipts.iter().all(|receipt| {
            let node = receipt.node_fingerprint();
            (node == self.integrity.fingerprint() && receipt.owner_tag() == 2)
                || (node == self.backend.fingerprint() && receipt.owner_tag() == 1)
                || (node == self.recovery.fingerprint() && receipt.owner_tag() == 3)
                || self
                    .layout
                    .is_some_and(|layout| node == layout.fingerprint() && receipt.owner_tag() == 5)
                || self
                    .blob
                    .is_some_and(|blob| node == blob.fingerprint() && receipt.owner_tag() == 6)
        })
    }

    pub(super) fn admits_recovered_starts(
        self,
        starts: &[crate::RecoveredRepairOwnerStart],
    ) -> bool {
        starts.iter().all(|started| {
            let node = started.node_fingerprint();
            (node == self.integrity.fingerprint() && started.owner_tag() == 2)
                || (node == self.backend.fingerprint() && started.owner_tag() == 1)
                || (node == self.recovery.fingerprint() && started.owner_tag() == 3)
                || self
                    .layout
                    .is_some_and(|layout| node == layout.fingerprint() && started.owner_tag() == 5)
                || self
                    .blob
                    .is_some_and(|blob| node == blob.fingerprint() && started.owner_tag() == 6)
        })
    }
}

pub(super) fn repair_dag(
    integrity: &IntegrityRepairClassificationPlan,
    backend: &LoweredNonCurrentStagingPlan,
    recovery: &BackupRestoreReplayPlan,
    layout: Option<&worth_store_layout_indexes::LayoutRepairConsequencePlan>,
    blob: Option<&worth_store_blob_chunks::BlobRepairConsequencePlan>,
) -> Result<(CanonicalOwnerPlanDag, RepairOwnerNodes), AuthorityAffectingRepairLoweringDenial> {
    let footprint = OwnerPlanFootprint::bounded(0, backend.binding().expected_bytes())
        .ok_or(AuthorityAffectingRepairLoweringDenial::InvalidFootprint)?;
    let integrity_effect = if integrity
        .regions()
        .iter()
        .any(|region| region.class() == IntegrityRepairRegionClass::QuarantineRequired)
    {
        OwnerPlanEffect::ClassifyQuarantine
    } else {
        OwnerPlanEffect::ValidatePhysicalIntegrity
    };
    let integrity_node = owner_node(
        StoreOwnerKind::PhysicalIntegrity,
        integrity_effect,
        footprint,
        integrity.fingerprint(),
        false,
    );
    let backend_node = owner_node(
        StoreOwnerKind::PhysicalBackend,
        OwnerPlanEffect::CopyBackupComponent,
        footprint,
        backend.binding().fingerprint(),
        true,
    );
    let recovery_node = owner_node(
        StoreOwnerKind::RecoveryPhysics,
        OwnerPlanEffect::ReplayWalToExactFrontier,
        footprint,
        recovery.fingerprint(),
        true,
    );
    let layout_node = layout.map(|plan| {
        let effect = match plan.consequence() {
            worth_store_layout_indexes::LayoutRepairConsequence::RestoreDamagedArtifact => {
                OwnerPlanEffect::RebuildDerivedLayout
            }
            worth_store_layout_indexes::LayoutRepairConsequence::ReplaceQuarantinedArtifact => {
                OwnerPlanEffect::ReplaceQuarantinedLayout
            }
        };
        owner_node(
            StoreOwnerKind::LayoutIndexes,
            effect,
            footprint,
            plan.fingerprint(),
            true,
        )
    });
    let blob_node = blob.map(|plan| {
        let effect = match plan.consequence() {
            worth_store_blob_chunks::BlobRepairConsequence::RestoreDamagedArtifact => {
                OwnerPlanEffect::RebuildBlobReachability
            }
            worth_store_blob_chunks::BlobRepairConsequence::ReplaceQuarantinedArtifact => {
                OwnerPlanEffect::ReplaceQuarantinedBlob
            }
        };
        owner_node(
            StoreOwnerKind::BlobChunks,
            effect,
            footprint,
            plan.fingerprint(),
            true,
        )
    });
    let nodes = RepairOwnerNodes {
        integrity: integrity_node.identity(),
        backend: backend_node.identity(),
        recovery: recovery_node.identity(),
        layout: layout_node.as_ref().map(OwnerPlanNode::identity),
        blob: blob_node.as_ref().map(OwnerPlanNode::identity),
    };
    let mut edges = vec![
        OwnerPlanPrerequisite::new(nodes.integrity, nodes.backend, true),
        OwnerPlanPrerequisite::new(nodes.backend, nodes.recovery, true),
    ];
    let mut owner_nodes = vec![integrity_node, backend_node, recovery_node];
    if let Some(node) = layout_node {
        edges.push(OwnerPlanPrerequisite::new(
            nodes.recovery,
            node.identity(),
            true,
        ));
        owner_nodes.push(node);
    }
    if let Some(node) = blob_node {
        edges.push(OwnerPlanPrerequisite::new(
            nodes.layout.unwrap_or(nodes.recovery),
            node.identity(),
            true,
        ));
        owner_nodes.push(node);
    }
    CanonicalOwnerPlanDag::admit(owner_nodes, edges)
        .map(|dag| (dag, nodes))
        .map_err(AuthorityAffectingRepairLoweringDenial::OwnerDag)
}

fn owner_node(
    owner: StoreOwnerKind,
    effect: OwnerPlanEffect,
    footprint: OwnerPlanFootprint,
    fingerprint: [u8; 32],
    irreversible: bool,
) -> OwnerPlanNode {
    let mut receipt = Sha256::new();
    receipt.update(b"worth-store-authority-repair-owner-receipt-v1");
    receipt.update(fingerprint);
    OwnerPlanNode::from_owner_binding(
        owner,
        effect,
        footprint,
        footprint.end_exclusive().saturating_sub(footprint.start()),
        irreversible,
        fingerprint,
        receipt.finalize().into(),
    )
}
