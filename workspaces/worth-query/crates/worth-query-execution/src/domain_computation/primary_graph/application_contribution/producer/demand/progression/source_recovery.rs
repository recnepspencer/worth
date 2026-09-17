use worth_query_installation::facade::ApplicationSchema;

use super::admission::validate_prepared_source_carrier;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationOutputDemandSource,
    WorthQueryApplicationReadObservation, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryPreparedRequiredOutputSource,
    WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn validate_recovered_output_root_kind(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
        kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.output_demands
            .validate_recovery_root_kind(receipt, kind)
    }

    pub(in crate::domain_computation::primary_graph) fn complete_prepared_output_source(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) {
        self.output_demands.complete_prepared_source(receipt);
    }

    pub(in crate::domain_computation::primary_graph) fn recover_prepared_output_source(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
        root_type: std::any::TypeId,
    ) -> Option<(
        std::sync::Arc<WorthQueryApplicationReadObservation>,
        WorthQueryPreparedRequiredOutputSource,
    )> {
        let observation = self
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
                source_commit: receipt
                    .committed_product_publication()
                    .composite_commit()
                    .clone(),
                product_occurrence: receipt.product_branch().occurrence(),
                owner: self.output_demands.clone(),
            },
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn ensure_recovered_output_source_bound<
        Query,
        Value,
    >(
        &self,
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
        self.output_demands.ensure_prepared_output_source_bound(
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
        self.output_demands.validate_prepared_recovery_currentness(
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
