use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    admit_primary_runtime_granular_batch, admit_primary_runtime_granular_invalidations,
    WorthQueryAdmittedInvalidationBatch, WorthQueryAdmittedInvalidationImpact,
    WorthQueryLiveBoundDomainProjection, WorthQuerySemanticDependencyRole,
};

use super::admission::{scope_for, strategy_for};
use super::{
    bind_performed_invalidation_maintenance, derive_performed_maintenance_effect,
    publish_invalidation_maintenance, WorthQueryMaintenanceScope, WorthQueryMaintenanceStrategy,
};

#[path = "primary_runtime/outcome.rs"]
mod outcome;
pub use outcome::{
    WorthQueryGranularNoChange, WorthQueryPrimaryGranularMaintenanceDenial,
    WorthQueryPrimaryGranularMaintenanceOutcome, WorthQueryPrimaryGranularMaintenancePerformed,
};

pub struct WorthQueryCoalescedMaintenancePlan {
    strategies: Vec<WorthQueryMaintenanceStrategy>,
    scope: WorthQueryMaintenanceScope,
    source_read_basis: Option<crate::runtime::WorthQueryGranularSourceReadBasis>,
    roles: Vec<WorthQuerySemanticDependencyRole>,
    consumer_delivery_count: usize,
}

impl WorthQueryCoalescedMaintenancePlan {
    pub fn strategy(&self) -> WorthQueryMaintenanceStrategy {
        self.strategies[0]
    }

    pub fn strategies(&self) -> &[WorthQueryMaintenanceStrategy] {
        &self.strategies
    }

    pub const fn scope(&self) -> &WorthQueryMaintenanceScope {
        &self.scope
    }

    pub(crate) const fn source_read_basis(
        &self,
    ) -> Option<&crate::runtime::WorthQueryGranularSourceReadBasis> {
        self.source_read_basis.as_ref()
    }

    pub fn roles(&self) -> &[WorthQuerySemanticDependencyRole] {
        &self.roles
    }

    pub const fn consumer_delivery_count(&self) -> usize {
        self.consumer_delivery_count
    }
}

pub fn maintain_primary_runtime_granular_invalidations<
    D: 'static,
    O: 'static,
    F: 'static,
    L: BasisOperationLane,
    Clock,
>(
    live: &WorthQueryLiveBoundDomainProjection<D, O, F, L>,
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    binding: &super::WorthQueryPrimaryRuntimeInvalidationBinding,
    receipt: &mut worth_query_execution::facade::primary_graph::WorthQueryConditionalClockObservationReceipt<
        Clock,
    >,
) -> Result<WorthQueryPrimaryGranularMaintenanceOutcome, WorthQueryPrimaryGranularMaintenanceDenial>
{
    if !binding.readmits_workspace(workspace) {
        return Err(WorthQueryPrimaryGranularMaintenanceDenial::ForeignPrimaryRuntime);
    }
    let admitted = admit_primary_runtime_granular_invalidations(live.snapshot(), binding, receipt)
        .map_err(WorthQueryPrimaryGranularMaintenanceDenial::Admission)?;
    maintain_admitted_batch(live, workspace, admitted, None)
}

/// Perform Query-owned maintenance from an execution-owned primary batch.
///
/// The batch may be transported by a clock, stream, region, or future shard
/// lane. Runtime identity and Query admission are revalidated here before any
/// source read or projection effect.
pub fn maintain_primary_runtime_granular_batch<
    D: 'static,
    O: 'static,
    F: 'static,
    L: BasisOperationLane,
>(
    live: &WorthQueryLiveBoundDomainProjection<D, O, F, L>,
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    binding: &super::WorthQueryPrimaryRuntimeInvalidationBinding,
    batch: worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationDeliveryBatch,
) -> Result<WorthQueryPrimaryGranularMaintenanceOutcome, WorthQueryPrimaryGranularMaintenanceDenial>
{
    if !binding.readmits_workspace(workspace) {
        return Err(WorthQueryPrimaryGranularMaintenanceDenial::ForeignPrimaryRuntime);
    }
    let admitted = admit_primary_runtime_granular_batch(live.snapshot(), binding, batch)
        .map_err(WorthQueryPrimaryGranularMaintenanceDenial::Admission)?;
    maintain_admitted_batch(live, workspace, admitted, None)
}

/// Perform primary granular maintenance against a retained Query collection
/// window. Indexed strategies cannot claim performed work without this state.
pub fn maintain_primary_runtime_granular_collection_batch<
    D: 'static,
    O: 'static,
    F: 'static,
    L: BasisOperationLane,
>(
    live: &WorthQueryLiveBoundDomainProjection<D, O, F, L>,
    collection: &mut crate::domain_installation::WorthQueryCollectionConsumerWindow,
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    binding: &super::WorthQueryPrimaryRuntimeInvalidationBinding,
    batch: worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationDeliveryBatch,
) -> Result<WorthQueryPrimaryGranularMaintenanceOutcome, WorthQueryPrimaryGranularMaintenanceDenial>
{
    if !binding.readmits_workspace(workspace) {
        return Err(WorthQueryPrimaryGranularMaintenanceDenial::ForeignPrimaryRuntime);
    }
    let admitted = admit_primary_runtime_granular_batch(live.snapshot(), binding, batch)
        .map_err(WorthQueryPrimaryGranularMaintenanceDenial::Admission)?;
    maintain_admitted_batch(live, workspace, admitted, Some(collection))
}

fn maintain_admitted_batch<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>(
    live: &WorthQueryLiveBoundDomainProjection<D, O, F, L>,
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    admitted: WorthQueryAdmittedInvalidationBatch,
    collection: Option<&mut crate::domain_installation::WorthQueryCollectionConsumerWindow>,
) -> Result<WorthQueryPrimaryGranularMaintenanceOutcome, WorthQueryPrimaryGranularMaintenanceDenial>
{
    if admitted.is_empty() {
        return Ok(
            WorthQueryPrimaryGranularMaintenanceOutcome::NoRelevantChange(
                WorthQueryGranularNoChange {
                    lower_truth_delivery_count: admitted.lower_truth_delivery_count(),
                    lower_signal_performed_delivery_count: admitted
                        .lower_signal_performed_delivery_count(),
                    duplicate_delivery_count: admitted.duplicate_delivery_count(),
                    already_settled_delivery_count: admitted.already_settled_delivery_count(),
                    irrelevant_delivery_count: admitted.irrelevant_delivery_count(),
                    suppressed_impact_count: 0,
                    admission_counters: admitted.admission_counters(),
                    impact_observations: Vec::new(),
                },
            ),
        );
    }
    let duplicate_delivery_count = admitted.duplicate_delivery_count();
    let performed_promotion_count = admitted.performed_promotion_count();
    let lower_truth_delivery_count = admitted.lower_truth_delivery_count();
    let lower_signal_performed_delivery_count = admitted.lower_signal_performed_delivery_count();
    let already_settled_delivery_count = admitted.already_settled_delivery_count();
    let irrelevant_delivery_count = admitted.irrelevant_delivery_count();
    let admission_counters = admitted.admission_counters();
    let (impacts, _, source_read_basis) = admitted.into_parts();
    let admitted_impact_count = impacts.len();
    let impact_observations = impacts
        .iter()
        .map(WorthQueryAdmittedInvalidationImpact::observation)
        .collect::<Vec<_>>();
    let plan = coalesced_plan(&impacts, source_read_basis)
        .ok_or(WorthQueryPrimaryGranularMaintenanceDenial::MixedMaintenancePosture)?;
    let basis = plan
        .source_read_basis()
        .ok_or(WorthQueryPrimaryGranularMaintenanceDenial::MixedMaintenancePosture)?;
    let refresh = live
        .refresh_granular_scope(plan.scope(), basis, workspace)
        .map_err(WorthQueryPrimaryGranularMaintenanceDenial::Execution)?;
    let maintenance_owner = live.maintenance_owner_identity().to_owned();
    let projection = super::prepare_projection_maintenance(
        workspace,
        super::WorthQueryProjectionMaintenanceRequest {
            owner: &maintenance_owner,
            plan: &plan,
            impacts: &impacts,
            current: live.snapshot(),
            refresh: &refresh,
        },
    );
    let Some(derived) = derive_performed_maintenance_effect(
        &plan,
        &impacts,
        live.snapshot(),
        &refresh,
        collection.as_deref(),
        projection,
    )
    .map_err(WorthQueryPrimaryGranularMaintenanceDenial::Maintenance)?
    else {
        return Ok(
            WorthQueryPrimaryGranularMaintenanceOutcome::NoRelevantChange(
                WorthQueryGranularNoChange {
                    lower_truth_delivery_count,
                    lower_signal_performed_delivery_count,
                    duplicate_delivery_count,
                    already_settled_delivery_count,
                    irrelevant_delivery_count,
                    suppressed_impact_count: admitted_impact_count,
                    admission_counters,
                    impact_observations,
                },
            ),
        );
    };
    let effect = derived.effect;
    let maintenance = bind_performed_invalidation_maintenance(
        impacts,
        &plan,
        live.snapshot(),
        &refresh,
        std::sync::Arc::clone(&effect),
    )
    .map_err(WorthQueryPrimaryGranularMaintenanceDenial::Maintenance)?;
    let deliveries = vec![
        publish_invalidation_maintenance(maintenance, live.snapshot(), &refresh)
            .map_err(WorthQueryPrimaryGranularMaintenanceDenial::Publication)?,
    ];
    workspace.apply_projection_maintenance(&maintenance_owner, derived.projection_commit);
    if let Some(pending) = derived.collection_commit {
        collection
            .expect("collection commits are prepared only for a retained collection")
            .apply_granular_maintenance(pending);
    }
    let maintenance_counters = super::WorthQueryGranularMaintenanceCounters::primary(
        &effect,
        admitted_impact_count,
        deliveries.len(),
    );
    Ok(WorthQueryPrimaryGranularMaintenanceOutcome::Performed(
        WorthQueryPrimaryGranularMaintenancePerformed {
            refresh,
            deliveries,
            admitted_impact_count,
            shared_execution_count: 1,
            duplicate_delivery_count,
            performed_promotion_count,
            lower_truth_delivery_count,
            lower_signal_performed_delivery_count,
            admission_counters,
            maintenance_counters,
            impact_observations,
        },
    ))
}

pub(super) fn coalesced_plan(
    impacts: &[WorthQueryAdmittedInvalidationImpact],
    source_read_basis: Option<crate::runtime::WorthQueryGranularSourceReadBasis>,
) -> Option<WorthQueryCoalescedMaintenancePlan> {
    let first = impacts.first()?;
    let strategies = strategies_for(first)?;
    let scope = scope_for(first.locality.clone());
    let mut roles = first.roles().to_vec();
    for impact in &impacts[1..] {
        if strategies_for(impact)? != strategies || scope_for(impact.locality.clone()) != scope {
            return None;
        }
        roles.extend_from_slice(impact.roles());
    }
    roles.sort_unstable();
    roles.dedup();
    Some(WorthQueryCoalescedMaintenancePlan {
        strategies,
        scope,
        source_read_basis,
        roles,
        consumer_delivery_count: impacts.len(),
    })
}

fn strategies_for(
    impact: &WorthQueryAdmittedInvalidationImpact,
) -> Option<Vec<WorthQueryMaintenanceStrategy>> {
    impact
        .consequence_classes()
        .iter()
        .copied()
        .map(strategy_for)
        .collect()
}
