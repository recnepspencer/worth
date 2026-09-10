use std::sync::Arc;

use crate::basis_lifecycle::BasisOperationLane;

use super::classification::{cause_for, classify_disposition, continuation_for, locality_for};
use super::{
    WorthQueryConsumerInvalidationAuthority, WorthQueryConsumerInvalidationCause,
    WorthQueryConsumerInvalidationCounters, WorthQueryConsumerInvalidationDelta,
    WorthQueryConsumerInvalidationDeltaStop, WorthQueryConsumerInvalidationDeltaStopKind,
    WorthQueryConsumerInvalidationDisposition,
};

impl<D: 'static, O, F, L: BasisOperationLane>
    crate::domain_installation::WorthQuerySharedLiveProjectionLease<D, O, F, L>
{
    pub fn consumer_invalidation_delta(
        &self,
        delivery: crate::domain_installation::WorthQuerySharedProjectionDelivery,
    ) -> Result<WorthQueryConsumerInvalidationDelta, WorthQueryConsumerInvalidationDeltaStop> {
        let epoch_counters = delivery.invalidation_seed().counters();
        let mut counters = WorthQueryConsumerInvalidationCounters::default();
        let readmission = self.readmission();
        counters.lease_impact_readmission_attempts = 1;
        let impact = match delivery.readmit_impact_for_lease(&readmission) {
            Ok(impact) => impact,
            Err(
                crate::domain_installation::operation_execution::WorthQuerySharedImpactReadmissionDenial::Lease,
            ) => {
                return Err(stop(
                    WorthQueryConsumerInvalidationDeltaStopKind::ForeignOrStaleLease,
                    counters,
                    epoch_counters,
                ))
            }
            Err(
                crate::domain_installation::operation_execution::WorthQuerySharedImpactReadmissionDenial::Impact,
            ) => {
                return Err(stop(
                    WorthQueryConsumerInvalidationDeltaStopKind::ImpactDeliveryMismatch,
                    counters,
                    epoch_counters,
                ))
            }
        };
        counters.semantic_delivery_checks = 1;
        if delivery.counters().this_lease_semantic_delivery == 0 {
            return Err(stop(
                WorthQueryConsumerInvalidationDeltaStopKind::NoSemanticDelivery,
                counters,
                epoch_counters,
            ));
        }
        for dimension in [
            crate::domain_installation::WorthQueryConsumerSupportDimension::Invalidation,
            crate::domain_installation::WorthQueryConsumerSupportDimension::DependencyImpact,
        ] {
            counters.consumer_support_checks += 1;
            let posture = self
                .snapshot()
                .consumer_contract()
                .support_posture(dimension);
            if posture != crate::domain_installation::WorthQueryConsumerSupportPosture::Supported {
                return Err(WorthQueryConsumerInvalidationDeltaStop::unsupported(
                    dimension,
                    posture,
                    counters,
                    epoch_counters,
                ));
            }
        }
        self.derive_invalidation_delta(&delivery, readmission, impact, counters, epoch_counters)
    }

    fn derive_invalidation_delta(
        &self,
        delivery: &crate::domain_installation::WorthQuerySharedProjectionDelivery,
        readmission: crate::domain_installation::operation_execution::WorthQuerySharedProjectionLeaseReadmission<'_>,
        impact: &Arc<crate::domain_installation::WorthQueryImpactDecision>,
        mut counters: WorthQueryConsumerInvalidationCounters,
        epoch_counters: super::WorthQueryConsumerInvalidationEpochCounters,
    ) -> Result<WorthQueryConsumerInvalidationDelta, WorthQueryConsumerInvalidationDeltaStop> {
        counters.disposition_classifications = 1;
        let mut disposition = classify_disposition(impact.class()).ok_or_else(|| {
            stop(
                WorthQueryConsumerInvalidationDeltaStopKind::NoSemanticDelivery,
                counters,
                epoch_counters,
            )
        })?;
        counters.targeted_lease_deliveries = 1;
        let seed = delivery.invalidation_seed();
        counters.native_access_layout_lookups = 1;
        let (affected_native_keys, narrowing) = self
            .snapshot()
            .native_access_layout()
            .map(|layout| layout.affected_keys(seed.touches()))
            .unwrap_or_default();
        counters.native_key_index_lookups = narrowing.index_lookups;
        counters.native_path_index_probes = narrowing.path_index_probes;
        counters.targeted_native_key_visits = narrowing.targeted_key_visits;
        counters.native_key_overlap_deduplications = narrowing.overlap_deduplications;
        let narrowing_unavailable = disposition
            == WorthQueryConsumerInvalidationDisposition::LocalPatch
            && affected_native_keys.is_empty();
        if narrowing_unavailable {
            disposition = WorthQueryConsumerInvalidationDisposition::Unsupported;
        }
        let collection = self.snapshot().consumer_contract().collection();
        let cause = if narrowing_unavailable {
            WorthQueryConsumerInvalidationCause::NativeNarrowingUnavailable(
                seed.delivery_causes().to_vec(),
            )
        } else {
            cause_for(disposition, seed.delivery_causes())
        };
        Ok(WorthQueryConsumerInvalidationDelta {
            authority: WorthQueryConsumerInvalidationAuthority::from_lease(
                &readmission,
                self.snapshot()
                    .bound_operation()
                    .operation()
                    .installation_generation(),
            ),
            maintenance_ordinal: delivery.maintenance_ordinal(),
            impact: Arc::clone(impact),
            conditional_provenance: Arc::clone(delivery.conditional_provenance_arc()),
            sharing: Arc::clone(delivery.sharing_admission()),
            epoch_work: Arc::clone(seed),
            affected_native_keys,
            disposition,
            cause,
            locality: locality_for(disposition, collection),
            continuation: continuation_for(collection),
            counters,
        })
    }
}

fn stop(
    kind: WorthQueryConsumerInvalidationDeltaStopKind,
    counters: WorthQueryConsumerInvalidationCounters,
    epoch_counters: super::WorthQueryConsumerInvalidationEpochCounters,
) -> WorthQueryConsumerInvalidationDeltaStop {
    WorthQueryConsumerInvalidationDeltaStop::new(kind, counters, epoch_counters)
}
