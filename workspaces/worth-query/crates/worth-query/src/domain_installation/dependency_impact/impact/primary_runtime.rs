use crate::basis_lifecycle::BasisOperationLane;

use super::{
    admit_current_invalidation_impact, WorthQueryAdmittedInvalidationImpact,
    WorthQueryImpactAdmissionDenial,
};
use crate::domain_installation::WorthQuerySettledDomainProjection;

/// Query-owned admission result for the exact granular deliveries carried by
/// one primary-runtime clock observation.
pub struct WorthQueryAdmittedInvalidationBatch {
    impacts: Vec<WorthQueryAdmittedInvalidationImpact>,
    admission_counters: super::WorthQueryGranularAdmissionCounters,
    lower_truth_delivery_count: usize,
    lower_signal_performed_delivery_count: usize,
    duplicate_delivery_count: usize,
    performed_promotion_count: usize,
    already_settled_delivery_count: usize,
    irrelevant_delivery_count: usize,
    source_read_basis: Option<crate::runtime::WorthQueryGranularSourceReadBasis>,
}

impl WorthQueryAdmittedInvalidationBatch {
    pub const fn len(&self) -> usize {
        self.impacts.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.impacts.is_empty()
    }

    pub fn into_impacts(self) -> Vec<WorthQueryAdmittedInvalidationImpact> {
        self.impacts
    }

    pub const fn admission_counters(&self) -> super::WorthQueryGranularAdmissionCounters {
        self.admission_counters
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<WorthQueryAdmittedInvalidationImpact>,
        super::WorthQueryGranularAdmissionCounters,
        Option<crate::runtime::WorthQueryGranularSourceReadBasis>,
    ) {
        (
            self.impacts,
            self.admission_counters,
            self.source_read_basis,
        )
    }

    pub const fn duplicate_delivery_count(&self) -> usize {
        self.duplicate_delivery_count
    }

    pub const fn lower_truth_delivery_count(&self) -> usize {
        self.lower_truth_delivery_count
    }

    pub const fn lower_signal_performed_delivery_count(&self) -> usize {
        self.lower_signal_performed_delivery_count
    }

    pub const fn performed_promotion_count(&self) -> usize {
        self.performed_promotion_count
    }

    pub const fn already_settled_delivery_count(&self) -> usize {
        self.already_settled_delivery_count
    }

    pub const fn irrelevant_delivery_count(&self) -> usize {
        self.irrelevant_delivery_count
    }
}

/// Consume the primary runtime's lower-owned delivery carrier and perform
/// Query's own candidate selection and current admission. The clock receipt
/// cannot authorize Query work; it only transports Bridge truth.
pub fn admit_primary_runtime_granular_invalidations<D, O, F, L: BasisOperationLane, Clock>(
    current: &WorthQuerySettledDomainProjection<D, O, F, L>,
    binding: &crate::live::WorthQueryPrimaryRuntimeInvalidationBinding,
    receipt: &mut worth_query_execution::facade::primary_graph::WorthQueryConditionalClockObservationReceipt<
        Clock,
    >,
) -> Result<WorthQueryAdmittedInvalidationBatch, WorthQueryImpactAdmissionDenial> {
    let batch = receipt.take_granular_invalidation_batch();
    admit_primary_runtime_granular_batch(current, binding, batch)
}

/// Admit one execution-owned granular batch against the exact current Query
/// projection and primary-runtime binding.
///
/// This is the transport-neutral composition seam. Clock, stream, region, or
/// shard owners may carry the batch, but none of them can replace Query's
/// current admission.
pub fn admit_primary_runtime_granular_batch<D, O, F, L: BasisOperationLane>(
    current: &WorthQuerySettledDomainProjection<D, O, F, L>,
    binding: &crate::live::WorthQueryPrimaryRuntimeInvalidationBinding,
    batch: worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationDeliveryBatch,
) -> Result<WorthQueryAdmittedInvalidationBatch, WorthQueryImpactAdmissionDenial> {
    if !binding.readmits(current, &batch) {
        return Err(WorthQueryImpactAdmissionDenial::new(
            super::WorthQueryImpactAdmissionDenialKind::ForeignRuntime,
            super::WorthQueryImpactCounters {
                runtime_authority_checks: 1,
                ..Default::default()
            },
        ));
    }
    let observation = batch.observation();
    let source_read_basis = batch
        .source_read_basis()
        .map(crate::runtime::WorthQueryGranularSourceReadBasis::from_execution_basis);
    admit_granular_invalidation_deliveries_with_observation(
        current,
        batch.into_bridge_deliveries(),
        observation.direct_truth_delivery_count(),
        observation.signal_performed_delivery_count(),
        Some(binding),
        source_read_basis,
    )
}

fn admit_granular_invalidation_deliveries_with_observation<D, O, F, L: BasisOperationLane>(
    current: &WorthQuerySettledDomainProjection<D, O, F, L>,
    deliveries: Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery>,
    lower_truth_delivery_count: usize,
    lower_signal_performed_delivery_count: usize,
    primary_binding: Option<&crate::live::WorthQueryPrimaryRuntimeInvalidationBinding>,
    source_read_basis: Option<crate::runtime::WorthQueryGranularSourceReadBasis>,
) -> Result<WorthQueryAdmittedInvalidationBatch, WorthQueryImpactAdmissionDenial> {
    let closure = current.semantic_aspect_dependency_closure();
    let mut admission_counters = super::WorthQueryGranularAdmissionCounters::default();
    for delivery in &deliveries {
        preflight_current_delivery(closure, delivery)?;
        admission_counters.inspect_delivery(delivery);
    }
    let converged = super::delivery_convergence::converge_granular_deliveries(deliveries)?;
    let mut impacts = Vec::with_capacity(converged.deliveries.len());
    let mut already_settled_delivery_count = 0;
    let mut irrelevant_delivery_count = 0;
    for delivery in converged.deliveries {
        if delivery_is_already_settled(current, &delivery)? {
            already_settled_delivery_count += 1;
            continue;
        }
        let binding =
            primary_binding.expect("bound-primary admission retains its current Query binding");
        let (consumer_dependencies, index_lookups) = binding.consumer_dependencies_for(
            delivery.truth().change_set().dependency(),
            delivery.truth().change_set().changes(),
        );
        let candidates = super::candidate_set::select_bound_primary_invalidation_candidates(
            closure,
            &consumer_dependencies,
            index_lookups,
            delivery,
        )?;
        admission_counters.retain_candidates(candidates.index_lookups(), candidates.roles());
        if candidates.roles().is_empty() {
            irrelevant_delivery_count += 1;
            admission_counters.reject_candidate();
            continue;
        }
        admission_counters.admit_roles(candidates.roles());
        impacts.push(admit_current_invalidation_impact(current, candidates)?);
    }
    Ok(WorthQueryAdmittedInvalidationBatch {
        impacts,
        admission_counters,
        lower_truth_delivery_count,
        lower_signal_performed_delivery_count,
        duplicate_delivery_count: converged.duplicate_delivery_count,
        performed_promotion_count: converged.performed_promotion_count,
        already_settled_delivery_count,
        irrelevant_delivery_count,
        source_read_basis,
    })
}

fn preflight_current_delivery(
    closure: &crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
    delivery: &worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery,
) -> Result<(), WorthQueryImpactAdmissionDenial> {
    super::classification::preflight_bound_primary_truth(closure, delivery.truth())
}

fn delivery_is_already_settled<D, O, F, L: BasisOperationLane>(
    current: &WorthQuerySettledDomainProjection<D, O, F, L>,
    delivery: &worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery,
) -> Result<bool, WorthQueryImpactAdmissionDenial> {
    let dependency = delivery.truth().change_set().dependency();
    let location =
        crate::domain_installation::conditional_execution::query_location_from_bridge_candidate(
            dependency,
        );
    let Some(provenance) = current
        .conditional_provenance()
        .iter()
        .find(|provenance| provenance.location() == &location)
    else {
        return Ok(false);
    };
    let Some(current_snapshot) = provenance.bridge_evidence().bridge_snapshot_identity() else {
        return Ok(false);
    };
    let Some(current_parts) = current_snapshot.relational_snapshot_parts() else {
        return Ok(false);
    };
    let Some(incoming_parts) = delivery
        .truth()
        .change_set()
        .snapshot_identity()
        .relational_snapshot_parts()
    else {
        return Err(WorthQueryImpactAdmissionDenial::new(
            super::WorthQueryImpactAdmissionDenialKind::ConditionalDeliveryMismatch,
            super::WorthQueryImpactCounters {
                delivery_identity_checks: 1,
                ..Default::default()
            },
        ));
    };
    Ok(current_parts.snapshot_id() == incoming_parts.snapshot_id()
        && incoming_parts.version_id() <= current_parts.version_id())
}
