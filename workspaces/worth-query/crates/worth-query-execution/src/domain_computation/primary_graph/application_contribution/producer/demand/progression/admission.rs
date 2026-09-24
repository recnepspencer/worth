use worth_query_installation::facade::ApplicationSchema;

mod dependent;
mod restoration;
mod source_custody;
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
            None,
            profile_kind,
            maximum_work,
            maximum_retained_bytes,
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
            None,
            profile_kind,
            maximum_work,
            maximum_retained_bytes,
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
            maximum_work,
            maximum_retained_bytes,
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
            None,
            profile_kind,
            maximum_work,
            maximum_retained_bytes,
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
        maximum_work: usize,
        maximum_retained_bytes: usize,
        performed_source: Option<worth_runtime_world::facade::CompositeCommitIdentity>,
        admission_kind: crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind,
        expected_source_commit: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
        successor_of: Option<
            &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
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
        let currentness_work_limit =
            std::num::NonZeroUsize::new(maximum_work).ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    Family::IDENTITY,
                )
                .with_recovery_posture(
                    super::super::WorthQueryOutputDemandRecoveryPosture::Retryable,
                )
            })?;
        let selected = self.select_output_producer_with_retained_basis::<Family>(
            selection_source.unwrap_or(&observed_source),
            profile_kind,
            maximum_work,
            retained_program_basis.as_deref(),
        )?;
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
        let source_epoch = observed_source.output_source_epoch().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            )
        })?;
        let key = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey::new(
            selected.identity.clone(),
            source_epoch.clone(),
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
        let source_scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
            observed_source.source_root(),
        );
        if let Some(restored) = self.readmit_checkpoint_output(
            &selected.identity,
            &observed_source,
            source_epoch,
            source_scope,
        )? {
            let resources = restored.checkpoint.resources.ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
                    "restored output has no retained producer resource profile",
                )
            })?;
            super::resources::validate_retained_resources(
                resources,
                &selected.identity,
                maximum_work,
                maximum_retained_bytes,
            )?;
            let (interest, newly_adopted) = self.output_demands.admit_restored(
                key,
                source_scope,
                product_occurrence,
                restored.clone(),
            )?;
            if newly_adopted {
                self.record_restored_output(source_scope, &restored)?;
            }
            return Ok(WorthQueryAdmittedOutputDemand {
                runtime_authority: self.runtime.authority_identity().as_u64(),
                schema_binding: self.installed_schema.binding_identity(),
                selected,
                observed_source,
                currentness_work_limit,
                maximum_retained_bytes,
                resources: Some(resources),
                resources_validated: true,
                producer_contacts_in_this_demand: 0,
                admission_kind,
                retained_program_basis,
                interest: Some(interest),
            });
        }
        // Exact source currentness has already been proved by Query selection.
        // An unchanged output must not contact the provider just to estimate
        // resources for work it will not perform. A racing fresh execution
        // validates the same limits before producer execution.
        let resources = if selected.exact_retained_output {
            let resources = selected.retained_resources.ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
                    "retained output has no producer resource profile",
                )
            })?;
            super::resources::validate_retained_resources(
                resources,
                &selected.identity,
                maximum_work,
                maximum_retained_bytes,
            )?;
            resources
        } else {
            super::resources::validate_demand_resources(
                entry.executor.as_ref(),
                &source,
                &selected.identity,
                maximum_work,
                maximum_retained_bytes,
            )?
        };
        let interest = match performed_source {
            Some(source_commit) => self.output_demands.admit_performed(
                key,
                &source_commit,
                source_scope,
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
                source_scope,
                product_occurrence,
                admission_kind,
                expected_source_commit,
                successor_of,
            )?,
        };
        let resources_validated = !selected.exact_retained_output;
        Ok(WorthQueryAdmittedOutputDemand {
            runtime_authority: self.runtime.authority_identity().as_u64(),
            schema_binding: self.installed_schema.binding_identity(),
            selected,
            observed_source,
            currentness_work_limit,
            maximum_retained_bytes,
            resources: Some(resources),
            resources_validated,
            producer_contacts_in_this_demand: 0,
            admission_kind,
            retained_program_basis,
            interest: Some(interest),
        })
    }
}
