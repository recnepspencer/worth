use crate::equality_type_id::EqualityTypeId as TypeId;
use worth_query_installation::facade::ApplicationSchema;
mod cold_source;
mod world_observation;

use super::admission::validate_prepared_source_carrier;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputDemandSource, WorthQueryApplicationReadObservation,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind, WorthQueryOutputSourceEvidence,
    WorthQueryPreparedRequiredOutputSource, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn resolve_output_source_replay(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        selector: crate::domain_computation::primary_graph::WorthQueryOutputSourceReplay<'_>,
        kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind,
        root_identity: &str,
    ) -> Result<WorthQueryOutputSourceEvidence, WorthQueryOutputDemandDenial> {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "registered-source-replay",
                )
            })?;
        let resolved = execution.run_native_with_checkpoint(|context| {
            self.runtime.output_demands.resolve_replay_source(context, selector, kind.clone())
        }).map_err(|cause| WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ExecutionRequest(crate::domain_computation::primary_graph::application_contribution::advancement_denial(cause)), "registered-source-replay"
        ))?.0;
        match resolved {
            Ok(receipt) => Ok(receipt),
            Err(cause)
                if matches!(
                    cause.kind(),
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
                ) =>
            {
                let crate::domain_computation::primary_graph::WorthQueryOutputSourceReplay::Recorded(original) = selector else { return Err(cause); };
                self.readmit_recorded_output_source(
                    &execution,
                    original,
                    kind.clone(),
                    root_identity,
                )?;
                execution.run_native_with_checkpoint(|context| {
                    self.runtime.output_demands.resolve_replay_source(context, selector, kind)
                }).map_err(|cause| WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(crate::domain_computation::primary_graph::application_contribution::advancement_denial(cause)), "registered-source-replay"
                ))?.0
            }
            Err(cause) => Err(cause),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn validate_recovered_output_root_kind(
        &self,
        receipt: &WorthQueryOutputSourceEvidence,
        kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.runtime
            .output_demands
            .validate_recovery_root_kind(receipt, kind)
    }

    pub(in crate::domain_computation::primary_graph) fn complete_prepared_output_source(
        &self,
        receipt: &WorthQueryOutputSourceEvidence,
    ) {
        self.runtime
            .output_demands
            .complete_prepared_source(receipt);
    }

    pub(in crate::domain_computation::primary_graph) fn recover_prepared_output_source(
        &self,
        receipt: &WorthQueryOutputSourceEvidence,
        root_type: TypeId,
    ) -> Option<(
        std::sync::Arc<WorthQueryApplicationReadObservation>,
        WorthQueryPreparedRequiredOutputSource,
    )> {
        let observation = self
            .runtime
            .output_demands
            .retained_prepared_source_observation(receipt, root_type)?;
        let retained = WorthQueryApplicationReadObservation::from_product(
            self,
            crate::basis::WorthQueryProductObservationLease::new(observation),
        );
        Some((
            retained,
            WorthQueryPreparedRequiredOutputSource {
                runtime_authority: self.runtime.authority_identity().as_u64(),
                source_commit: receipt.source_commit().clone(),
                product_occurrence: receipt.product_branch().occurrence(),
                owner: self.runtime.output_demands.clone(),
            },
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn ensure_recovered_output_source_bound<
        Query,
        Value,
    >(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        source: &WorthQueryApplicationOutputDemandSource<Query, Value>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let observed = source
            .observed_sources()
            .first()
            .filter(|_| source.rows().len() == 1 && source.observed_sources().len() == 1)
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "recovered source did not return one owner-paired occurrence",
                )
            })?;
        validate_prepared_source_carrier(
            self.runtime.authority_identity().as_u64(),
            prepared,
            observed,
        )?;
        self.publish_output_source_bindings(phase, prepared, &[observed])?;
        self.runtime.output_demands.ensure_prepared_output_source_bound(
            &prepared.source_commit,
            crate::domain_computation::primary_graph::application_output_demand::BoundOutputSource {
                scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(observed.source_root()),
                identity: observed.output_source_epoch().ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::ForeignSource,
                        "recovered source has no product epoch",
                    )
                })?,
            },
        )
    }

    pub(in crate::domain_computation::primary_graph) fn validate_recovered_output_source_currentness<
        Query,
        Value,
    >(
        &self,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        retained: &WorthQueryApplicationOutputDemandSource<Query, Value>,
        current: &WorthQueryApplicationOutputDemandSource<Query, Value>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let retained_observed = paired_source(retained).ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "retained recovery query did not return one source occurrence",
            )
        })?;
        let current_observed = paired_source(current).ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "current recovery query did not return one source occurrence",
            )
        })?;
        validate_prepared_source_carrier(
            self.runtime.authority_identity().as_u64(),
            prepared,
            retained_observed,
        )?;
        if current_observed.selected_product_occurrence() != Some(prepared.product_occurrence) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "current recovery query belongs to another product occurrence",
            ));
        }
        self.runtime
            .output_demands
            .validate_prepared_recovery_currentness(
                &prepared.source_commit,
                retained_observed.output_source_epoch().ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::ForeignSource,
                        "retained source has no product epoch",
                    )
                })?,
                current_observed.output_source_epoch().ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::ForeignSource,
                        "current source has no product epoch",
                    )
                })?,
            )
    }
}

fn paired_source<Query, Value>(
    source: &WorthQueryApplicationOutputDemandSource<Query, Value>,
) -> Option<&crate::domain_computation::primary_graph::WorthQueryObservedSource<Query>> {
    source
        .observed_sources()
        .first()
        .filter(|_| source.rows().len() == 1 && source.observed_sources().len() == 1)
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(in crate::domain_computation::primary_graph) fn publish_output_source_bindings<Q>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        sources: &[&crate::domain_computation::primary_graph::WorthQueryObservedSource<Q>],
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(owner) = prepared.source_commit.owner_identity().durable() else {
            return Ok(());
        };
        let request = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "source binding publication",
                )
            })?;
        let history = self
            .installed_program_support()
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "source support unavailable",
                )
            })?
            .obligation_history();
        let bindings = sources
            .iter()
            .map(|source| {
                let epoch = source.output_source_epoch().ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::ForeignSource,
                        "source has no original epoch",
                    )
                })?;
                let root = source.source_root();
                Ok((
                    epoch.checkpoint_identity().bytes(),
                    (
                        root.partition_value(),
                        root.local_slot_value(),
                        root.generation_value(),
                    ),
                ))
            })
            .collect::<Result<Vec<_>, WorthQueryOutputDemandDenial>>()?;
        history
            .bind_output_source(
                &request,
                worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
                    owner,
                    prepared.source_commit.ordinal(),
                ),
                bindings,
            )
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ObligationPublication(cause),
                    "source binding publication",
                )
            })
    }
}
