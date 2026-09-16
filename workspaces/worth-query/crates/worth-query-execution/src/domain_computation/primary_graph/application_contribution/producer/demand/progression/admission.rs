use worth_query_installation::facade::ApplicationSchema;

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

    pub(in crate::domain_computation::primary_graph) fn retain_required_output_source(
        &self,
        receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        change: crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
        preparation: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRequiredOutputSourcePreparation,
    ) -> Result<
        (
            crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
            std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >,
        ),
        WorthQueryOutputDemandDenial,
    > {
        let publication = receipt.committed_product_publication();
        let source_scope = receipt.principal_scope().scope();
        let same_runtime =
            std::sync::Arc::ptr_eq(&change.root_identity, &self.product_runtime.root_identity());
        let same_publication = change.product_branch_identity() == publication.product_branch()
            && change.product_commit() == publication.composite_commit();
        if !same_runtime || !same_publication {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "performed source does not belong to its application publication",
            ));
        }
        let observation = publication
            .take_output_demand_observation()
            .filter(|observation| {
                observation.branch_identity() == change.product_branch_identity()
                    && observation.selected_commit() == change.product_commit()
            })
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "performed source publication did not retain its exact output-demand basis",
                )
            })?;
        let retained = crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation::from_product(
            self,
            crate::basis::WorthQueryProductObservationLease::new(observation.clone()),
        );
        let predecessor_source_identity = receipt.idempotency_binding().source_identity();
        let source_commit = self.output_demands.retain_performed_source(
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryPerformedOutputDemandSource {
                receipt,
                change,
                observation,
                predecessor_source_identity,
                current_source_identity: None,
            },
            preparation,
        )?;
        Ok((
            crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource {
                runtime_authority: self.runtime.authority_identity().as_u64(),
                source_commit,
                source_scope,
                owner: self.output_demands.clone(),
            },
            retained,
        ))
    }

    pub fn bind_prepared_required_output_source<Query, Value>(
        &self,
        prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
        source: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            Query,
            Value,
        >,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let observed = source
            .observed_sources()
            .first()
            .filter(|_| source.rows().len() == 1 && source.observed_sources().len() == 1);
        let Some(observed) = observed else {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "prepared output source query did not return one owner-paired occurrence",
            ));
        };
        if prepared.runtime_authority != self.runtime.authority_identity().as_u64()
            || observed.selected_product_commit() != Some(&prepared.source_commit)
            || crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
                observed.footprint.root,
            ) != prepared.source_scope
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "prepared output source does not match its committed carrier",
            ));
        }
        self.output_demands
            .bind_prepared_output_source(&prepared.source_commit, observed.idempotency_identity())
    }

    pub fn discard_prepared_required_output_source(
        &self,
        prepared: crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
    ) {
        if prepared.runtime_authority == self.runtime.authority_identity().as_u64() {
            self.output_demands
                .discard_prepared_source(&prepared.source_commit);
        }
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
        if prepared.runtime_authority != self.runtime.authority_identity().as_u64()
            || observed_source.selected_product_commit() != Some(&prepared.source_commit)
            || crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
                observed_source.footprint.root,
            ) != prepared.source_scope
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "prepared source does not match the admitted output occurrence",
            ));
        }
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
            ));
        }
        if resources.retained_bytes() > maximum_retained_bytes {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                &selected.identity,
            ));
        }
        let key = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey::new(
            selected.identity.clone(),
            observed_source.idempotency_identity(),
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
                    observed_source.footprint.root,
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
