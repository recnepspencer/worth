use worth_query_installation::facade::ApplicationSchema;

mod dependent;
mod restoration;
mod retained_observation;
mod selected_custody;
mod selection_mode;
mod source_admission;
mod source_custody;
pub(super) use selection_mode::SourceAdmissionSelection;
pub(super) use source_custody::validate_prepared_source_carrier;

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
    pub fn output_demand_resource_profile(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile {
        self.runtime.output_demand_resource_profile()
    }

    pub fn admit_output_demand<Family>(
        &self,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
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
            None,
            profile_kind,
            limits,
            None,
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Ordinary,
            None,
            None,
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
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
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
            None,
            profile_kind,
            limits,
            None,
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Required,
            None,
            None,
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
        current_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
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
        let (_, current_observed_source) =
            current_result.into_single_source().ok_or_else(|| {
                denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "current recovered output source query did not return one owner-paired occurrence",
            )
            })?;
        let retained_epoch = observed_source.output_source_epoch().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            )
        })?;
        let current_epoch = current_observed_source
            .output_source_epoch()
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Family::IDENTITY,
                )
            })?;
        if !retained_epoch.same_semantic_source(&current_epoch) {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "current recovered output source differs from retained source",
            ));
        }
        let profile_kind = Family::profile_kind(&source);
        self.admit_output_demand_with_source::<Family>(
            source,
            observed_source,
            Some(&current_observed_source),
            profile_kind,
            limits,
            None,
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Recovery,
            Some(source_receipt.committed_product_publication().composite_commit()),
            None,
            None,
        )
    }

    pub fn admit_performed_output_demand<Family>(
        &self,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
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
            None,
            profile_kind,
            limits,
            Some(prepared.source_commit.clone()),
            crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Required,
            None,
            None,
            None,
        )
    }

    pub(super) fn admit_output_demand_with_source<Family>(
        &self,
        source: FamilySourceValue<Schema, Family>,
        observed_source: WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
        selection_source: Option<&WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>>,
        profile_kind: &'static str,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
        performed_source: Option<worth_runtime_world::facade::CompositeCommitIdentity>,
        admission_kind: crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind,
        expected_source_commit: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
        successor_of: Option<
            crate::domain_computation::primary_graph::application_output_demand::OutputRefreshPredecessor<'_>,
        >,
        retained_program_basis: Option<
            std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >,
        >,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let mut admission = self.demand_request_admission();
        let result = self
            .admit_output_demand_with_source_admitted::<Family>(
                &source,
                observed_source,
                selection_source,
                profile_kind,
                limits,
                performed_source,
                admission_kind,
                expected_source_commit,
                successor_of,
                retained_program_basis,
                SourceAdmissionSelection::Ordinary,
                &mut admission,
            )
            .map_err(|stop| self.starting_custody_stop(stop, &mut admission));
        #[cfg(feature = "test-query-execution-observer")]
        super::caller_pass_observation::observe_admission_work(&admission);
        result
    }
}
