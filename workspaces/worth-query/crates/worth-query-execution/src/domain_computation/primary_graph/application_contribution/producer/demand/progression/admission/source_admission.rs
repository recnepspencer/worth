//! Source selection and registry admission share one request meter.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn admit_output_demand_with_source_admitted<
        Family,
    >(
        &self,
        source: &FamilySourceValue<Schema, Family>,
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
        source_selection: super::SourceAdmissionSelection<'_, '_, Schema>,
        registry_admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        registry_admission
            .charge_external_work(1)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        let selected_product = source_selection.selected_product();
        let limits = self.output_demand_resource_profile().constrain(limits);
        std::num::NonZeroUsize::new(limits.source_currentness_work()).ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                Family::IDENTITY,
            )
            .with_recovery_posture(crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable)
        })?;
        if retained_program_basis.is_some() {
            registry_admission.charge_external_work(1).map_err(|_| {
                denial(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    Family::IDENTITY,
                )
            })?;
        }
        let observed_source = self.output_demands.retain_readmission_source(
            observed_source,
            limits,
            retained_program_basis.clone(),
        )?;
        let before_selection = registry_admission.remaining_work();
        let mut remaining_source_work = before_selection;
        let selection = match selected_product {
            Some(selected) => self.select_output_producer_with_remaining_on_selected::<Family>(
                selection_source.unwrap_or(&observed_source),
                profile_kind,
                &mut remaining_source_work,
                retained_program_basis.as_deref(),
                selected,
            ),
            None => self.select_output_producer_with_remaining::<Family>(
                selection_source.unwrap_or(&observed_source),
                profile_kind,
                &mut remaining_source_work,
                retained_program_basis.as_deref(),
            ),
        };
        // This synchronous legacy selector carries a bounded remainder on both
        // success and failure. It cannot mint a fresh allowance during refresh.
        registry_admission
            .charge_external_work(
                u64::try_from(before_selection - remaining_source_work).map_err(|_| {
                    denial(
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                        Family::IDENTITY,
                    )
                })?,
            )
            .map_err(|_| {
                denial(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    Family::IDENTITY,
                )
            })?;
        let (selected, entry) = selection?;
        if selected_product.is_some() {
            source_selection.admit_selected_producer(&selected.identity, registry_admission)?;
        }
        // Selection already returned the exact immutable entry from its sole
        // matching iterator. Retain that entry for this demand's Interest.
        registry_admission
            .charge_external_work((std::mem::size_of_val(entry) + 1) as u64)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        let installed_entry = std::sync::Arc::clone(entry);
        let source_epoch = match selected_product {
            Some(_) => observed_source
                .output_source_epoch_admitted(&mut |work, bytes| {
                    registry_admission.charge_external_work(work)?;
                    registry_admission.admit_read_scratch(bytes)
                })
                .map_err(super::selected_custody::epoch_denial)?,
            None => observed_source.output_source_epoch(),
        }
        .ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            )
        })?;
        if selected_product.is_some() {
            super::selected_custody::preclaim_temporary_key(
                &selected.identity,
                registry_admission,
            )?;
        }
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
        // Only the exact retained-candidate selection has compared the full
        // persisted producer facts at the current native basis. A fresh
        // selection may never adopt a matching checkpoint by identity alone.
        let restored = if selected.exact_retained_output {
            let expected_key = selected.retained_idempotency_key.ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
                    "exact retained output has no idempotency identity",
                )
            })?;
            let output_binding = selected.retained_output_binding.ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
                    "exact retained output has no output binding",
                )
            })?;
            self.readmit_checkpoint_output(
                &selected.identity,
                output_binding,
                expected_key,
                &observed_source,
                source_epoch,
                source_scope,
                registry_admission,
            )?
        } else {
            None
        };
        if let Some(restored) = restored {
            let resources = restored.checkpoint.resources.ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
                    "restored output has no retained producer resource profile",
                )
            })?;
            super::super::resources::validate_retained_resources(
                resources,
                &selected.identity,
                limits,
            )?;
            let (interest, newly_adopted) = self.output_demands.admit_restored(
                key,
                source_scope,
                product_occurrence,
                admission_kind,
                restored.clone(),
                registry_admission,
            )?;
            if newly_adopted {
                self.record_restored_output(source_scope, &restored)?;
            }
            let observed_source = observed_source.install(&interest)?;
            return Ok(WorthQueryAdmittedOutputDemand {
                runtime_authority: self.runtime.authority_identity().as_u64(),
                schema_binding: self.installed_schema.binding_identity(),
                selected,
                installed_entry,
                observed_source,
                limits,
                resources: Some(resources),
                resources_validated: true,
                producer_contacts_in_this_demand: 0,
                admission_kind,
                retained_program_basis,
                progression_provenance: Default::default(),
                required_continuations: Default::default(),
                unpublished_selected_checkpoint: None,
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
            super::super::resources::validate_retained_resources(
                resources,
                &selected.identity,
                limits,
            )?;
            resources
        } else {
            super::super::resources::validate_demand_resources(
                entry.executor.as_ref(),
                source,
                &selected.identity,
                limits,
            )?
        };
        let interest = match performed_source {
            Some(source_commit) => self.output_demands.admit_performed(
                key,
                &source_commit,
                source_scope,
                product_occurrence,
                registry_admission,
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
                registry_admission,
            )?,
        };
        let observed_source = observed_source.install(&interest)?;
        let resources_validated = !selected.exact_retained_output;
        Ok(WorthQueryAdmittedOutputDemand {
            runtime_authority: self.runtime.authority_identity().as_u64(),
            schema_binding: self.installed_schema.binding_identity(),
            selected,
            installed_entry,
            observed_source,
            limits,
            resources: Some(resources),
            resources_validated,
            producer_contacts_in_this_demand: 0,
            admission_kind,
            retained_program_basis,
            progression_provenance: Default::default(),
            required_continuations: Default::default(),
            unpublished_selected_checkpoint: None,
            interest: Some(interest),
        })
    }
}
