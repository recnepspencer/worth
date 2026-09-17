use worth_query_installation::facade::ApplicationSchema;

mod source_custody;

use super::super::{
    FamilySourceQuery, FamilySourceValue, WorthQueryAdmittedOutputDemand,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind, WorthQueryProducerOutputFamily,
};
use super::denial;
use crate::domain_computation::primary_graph::{
    WorthQueryObservedSource, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn admit_output_demand<Family>(
        &self,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let (source, observed_source) = source_result.into_single_source().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "output source query did not return one owner-paired occurrence",
            )
        })?;
        let profile_kind = Family::profile_kind(&source);
        self.admit_output_demand_with_source::<Family>(
            source,
            observed_source,
            profile_kind,
            maximum_work,
            maximum_retained_bytes,
            None,
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Ordinary,
            None,
        )
    }

    #[doc(hidden)]
    pub(in crate::domain_computation) fn admit_required_output_demand<Family>(
        &self,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let (source, observed_source) = source_result.into_single_source().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "required output source query did not return one owner-paired occurrence",
            )
        })?;
        let profile_kind = Family::profile_kind(&source);
        self.admit_output_demand_with_source::<Family>(
            source,
            observed_source,
            profile_kind,
            maximum_work,
            maximum_retained_bytes,
            None,
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Required,
            None,
        )
    }

    #[doc(hidden)]
    pub(in crate::domain_computation) fn admit_recovered_output_demand<Family>(
        &self,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
        source_receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let (source, observed_source) = source_result.into_single_source().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "recovered output source query did not return one owner-paired occurrence",
            )
        })?;
        let profile_kind = Family::profile_kind(&source);
        self.admit_output_demand_with_source::<Family>(
            source,
            observed_source,
            profile_kind,
            maximum_work,
            maximum_retained_bytes,
            None,
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Recovery,
            Some(source_receipt.committed_product_publication().composite_commit()),
        )
    }

    pub fn admit_performed_output_demand<Family>(
        &self,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
        prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let (source, observed_source) = source_result.into_single_source().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "required-output source query did not return one owner-paired occurrence",
            )
        })?;
        validate_prepared_source_carrier(
            self.runtime.authority_identity().as_u64(),
            prepared,
            &observed_source,
        )?;
        let profile_kind = Family::profile_kind(&source);
        self.admit_output_demand_with_source::<Family>(
            source,
            observed_source,
            profile_kind,
            maximum_work,
            maximum_retained_bytes,
            Some(prepared.source_commit.clone()),
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Required,
            None,
        )
    }

    fn admit_output_demand_with_source<Family>(
        &self,
        source: FamilySourceValue<Schema, Family>,
        observed_source: WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
        profile_kind: &'static str,
        maximum_work: usize,
        maximum_retained_bytes: usize,
        performed_source: Option<worth_runtime_world::facade::CompositeCommitIdentity>,
        admission_kind: crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind,
        expected_source_commit: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let selected = self.select_output_producer::<Family>(&observed_source, profile_kind)?;
        let entry = self
            .installed_producers
            .entries
            .get(&selected.identity)
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    format!(
                        "{}: selected producer is absent from the installed registry",
                        selected.identity
                    ),
                )
            })?;
        let resources = entry.executor.resources(&source).ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                &selected.identity,
            )
        })?;
        if resources.work() > maximum_work {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                &selected.identity,
            )
            .with_recovery_posture(
                super::super::WorthQueryOutputDemandRecoveryPosture::Retryable,
            ));
        }
        if resources.retained_bytes() > maximum_retained_bytes {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                &selected.identity,
            )
            .with_recovery_posture(
                super::super::WorthQueryOutputDemandRecoveryPosture::Retryable,
            ));
        }
        let key = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey::new(
            selected.identity.clone(),
            observed_source.output_source_epoch().ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Family::IDENTITY,
                )
            })?,
        );
        let product_occurrence =
            observed_source
                .selected_product_occurrence()
                .ok_or_else(|| {
                    denial(
                        WorthQueryOutputDemandDenialKind::ForeignSource,
                        Family::IDENTITY,
                    )
                })?;
        let interest = match performed_source {
            Some(source_commit) => self.output_demands.admit_performed(
                key,
                &source_commit,
                crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(observed_source.source_root()),
                product_occurrence,
            )?,
            None => self.output_demands.admit(
                key,
                Some(observed_source.selected_product_commit().ok_or_else(|| {
                    denial(
                        WorthQueryOutputDemandDenialKind::ForeignSource,
                        Family::IDENTITY,
                    )
                })?),
                crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
                    observed_source.source_root(),
                ),
                product_occurrence,
                admission_kind,
                expected_source_commit,
            )?,
        };
        Ok(WorthQueryAdmittedOutputDemand {
            runtime_authority: self.runtime.authority_identity().as_u64(),
            schema_binding: self.installed_schema.binding_identity(),
            selected,
            source,
            observed_source,
            interest: Some(interest),
        })
    }
}

pub(super) fn validate_prepared_source_carrier<Query>(
    runtime_authority: u64,
    prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
    observed: &crate::domain_computation::primary_graph::WorthQueryObservedSource<Query>,
) -> Result<(), WorthQueryOutputDemandDenial> {
    if prepared.runtime_authority != runtime_authority {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            "prepared output source belongs to another Query runtime",
        ));
    }
    if observed.selected_product_commit() != Some(&prepared.source_commit) {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            "prepared output source belongs to another product commit",
        ));
    }
    if observed.selected_product_occurrence() != Some(prepared.product_occurrence) {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            "prepared output source belongs to another product occurrence",
        ));
    }
    Ok(())
}
