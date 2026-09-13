use super::integrity_classification::{
    IntegrityOperationalRepairOwner, IntegrityRepairClassificationDenial,
    IntegrityRepairClassificationPlan,
};
use sha2::{Digest, Sha256};
use worth_store_layout_indexes::{
    DerivedIndexRepairExecutionDenial, DerivedIndexRepairPlan, LayoutOperationalRepairOwner,
};

use crate::authorization::{
    authorize_lowered_plan, AuthorizationReplayPolicy, AuthorizedOperationalPlan,
    LoweredOperationalPlan,
};
use crate::owner_plan_dag::{
    CanonicalOwnerPlanDag, DestructiveOperationKind, OperationalPlanBinding, OwnerPlanEffect,
    OwnerPlanFootprint, OwnerPlanNode, OwnerPlanNodeIdentity, OwnerPlanPrerequisite,
    StoreOwnerKind,
};
use crate::{
    AuthorizationDenial, AuthorizationRevocationObservation, ExternalOperatorAssertion,
    OperationalAuthorizationPort,
};

use super::{
    CurrentAuthorityPreservingMaintenancePlan, DerivedRepairOperation, EvidenceBoundRepairPlan,
};

#[derive(Debug)]
pub enum RepairLoweringDenial {
    Integrity(IntegrityRepairClassificationDenial),
    Layout(DerivedIndexRepairExecutionDenial),
    DuplicateOwnerTarget,
    OwnerDag(crate::OwnerPlanDagDenial),
    CounterOverflow,
    InvalidFootprint,
}

#[derive(Debug, Clone)]
pub struct LoweredRepairOwnerPlanDag {
    pub(super) operation_id: crate::OperationalOperationId,
    pub(super) authorization: LoweredOperationalPlan<DerivedRepairOperation>,
    pub(super) integrity_node: OwnerPlanNodeIdentity,
    pub(super) integrity: IntegrityRepairClassificationPlan,
    pub(super) layout: Vec<(OwnerPlanNodeIdentity, DerivedIndexRepairPlan)>,
    explanation: crate::CanonicalOwnerPlanDagExplanation,
}

#[derive(Debug)]
pub struct AuthorizedRepairPlan {
    pub(super) operation_id: crate::OperationalOperationId,
    pub(super) authorization: AuthorizedOperationalPlan<DerivedRepairOperation>,
    pub(super) integrity_node: OwnerPlanNodeIdentity,
    pub(super) integrity: IntegrityRepairClassificationPlan,
    pub(super) layout: Vec<(OwnerPlanNodeIdentity, DerivedIndexRepairPlan)>,
}

impl CurrentAuthorityPreservingMaintenancePlan {
    pub fn lower_owners(self) -> Result<LoweredRepairOwnerPlanDag, RepairLoweringDenial> {
        lower_derived(self.plan)
    }
}

impl LoweredRepairOwnerPlanDag {
    pub const fn explanation(&self) -> &crate::CanonicalOwnerPlanDagExplanation {
        &self.explanation
    }
    pub fn owner_node_count(&self) -> u64 {
        self.layout.len() as u64 + 1
    }

    #[allow(clippy::too_many_arguments)]
    pub fn authorize(
        self,
        port: &impl OperationalAuthorizationPort,
        assertion: &ExternalOperatorAssertion,
        requested_at: u64,
        expires_at: u64,
        replay_policy: AuthorizationReplayPolicy,
        revocation: AuthorizationRevocationObservation,
    ) -> Result<AuthorizedRepairPlan, AuthorizationDenial> {
        Ok(AuthorizedRepairPlan {
            operation_id: self.operation_id,
            authorization: authorize_lowered_plan(
                self.authorization,
                port,
                assertion,
                requested_at,
                expires_at,
                replay_policy,
                revocation,
            )?,
            integrity_node: self.integrity_node,
            integrity: self.integrity,
            layout: self.layout,
        })
    }
}

fn lower_derived(
    plan: EvidenceBoundRepairPlan,
) -> Result<LoweredRepairOwnerPlanDag, RepairLoweringDenial> {
    let integrity = IntegrityOperationalRepairOwner::lower(plan.damaged)
        .map_err(RepairLoweringDenial::Integrity)?;
    let mut layout_plans = Vec::new();
    layout_plans
        .try_reserve_exact(plan.requests.len())
        .map_err(|_| RepairLoweringDenial::CounterOverflow)?;
    for request in plan.requests {
        layout_plans.push(
            LayoutOperationalRepairOwner::lower(request).map_err(RepairLoweringDenial::Layout)?,
        );
    }
    layout_plans.sort_by_key(DerivedIndexRepairPlan::fingerprint);
    if layout_plans
        .windows(2)
        .any(|pair| pair[0].target() == pair[1].target())
    {
        return Err(RepairLoweringDenial::DuplicateOwnerTarget);
    }
    let node_count = layout_plans
        .len()
        .checked_add(1)
        .ok_or(RepairLoweringDenial::CounterOverflow)? as u64;
    let classifier_footprint =
        OwnerPlanFootprint::bounded(0, node_count).ok_or(RepairLoweringDenial::InvalidFootprint)?;
    let classifier_node = node(
        StoreOwnerKind::PhysicalIntegrity,
        OwnerPlanEffect::ClassifyQuarantine,
        classifier_footprint,
        integrity.fingerprint(),
        false,
    );
    let integrity_node = classifier_node.identity();
    let mut nodes = vec![classifier_node];
    let mut edges = Vec::new();
    let mut bound_layout = Vec::new();
    for (index, owner_plan) in layout_plans.into_iter().enumerate() {
        let start = index as u64;
        let footprint = OwnerPlanFootprint::bounded(start, start + 1)
            .ok_or(RepairLoweringDenial::InvalidFootprint)?;
        let owner_node = node(
            StoreOwnerKind::LayoutIndexes,
            OwnerPlanEffect::RebuildDerivedLayout,
            footprint,
            owner_plan.fingerprint(),
            true,
        );
        edges.push(OwnerPlanPrerequisite::new(
            integrity_node,
            owner_node.identity(),
            true,
        ));
        bound_layout.push((owner_node.identity(), owner_plan));
        nodes.push(owner_node);
    }
    let dag = CanonicalOwnerPlanDag::admit(nodes, edges).map_err(RepairLoweringDenial::OwnerDag)?;
    let explanation = dag.explanation().clone();
    let target_identity = target_set_identity(&bound_layout);
    let binding = OperationalPlanBinding::bind(
        DestructiveOperationKind::DerivedRepair,
        dag,
        plan.authority_identity,
        plan.security_scope,
        plan.basis_identity,
        target_identity,
        integrity.fingerprint(),
    );
    Ok(LoweredRepairOwnerPlanDag {
        operation_id: plan.operation_id,
        authorization: LoweredOperationalPlan::from_binding(binding),
        integrity_node,
        integrity,
        layout: bound_layout,
        explanation,
    })
}

fn node(
    owner: StoreOwnerKind,
    effect: OwnerPlanEffect,
    footprint: OwnerPlanFootprint,
    fingerprint: [u8; 32],
    irreversible: bool,
) -> OwnerPlanNode {
    OwnerPlanNode::from_owner_binding(
        owner,
        effect,
        footprint,
        footprint.end_exclusive().saturating_sub(footprint.start()),
        irreversible,
        fingerprint,
        receipt_fingerprint(owner, fingerprint),
    )
}
fn receipt_fingerprint(owner: StoreOwnerKind, plan: [u8; 32]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-repair-owner-receipt-v1");
    digest.update([match owner {
        StoreOwnerKind::PhysicalBackend => 1,
        StoreOwnerKind::PhysicalIntegrity => 2,
        StoreOwnerKind::RecoveryPhysics => 3,
        StoreOwnerKind::PhysicalIsolation => 4,
        StoreOwnerKind::LayoutIndexes => 5,
        StoreOwnerKind::BlobChunks => 6,
        StoreOwnerKind::Authority => 7,
        StoreOwnerKind::Replication => 8,
    }]);
    digest.update(plan);
    digest.finalize().into()
}
fn target_set_identity(plans: &[(OwnerPlanNodeIdentity, DerivedIndexRepairPlan)]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-repair-target-set-v1");
    for (_, plan) in plans {
        digest.update(plan.fingerprint());
    }
    digest.finalize().into()
}
