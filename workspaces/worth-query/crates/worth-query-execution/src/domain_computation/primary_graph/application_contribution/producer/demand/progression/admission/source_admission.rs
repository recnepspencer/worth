//! Source selection and registry admission share one request meter.

use super::*;
use crate::domain_computation::primary_graph::application_output_demand::SelectedOutputAdmission;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn admit_output_demand_with_source_admitted<
        Family,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let selected_product = source_selection.selected_product();
        let limits = self.output_demand_resource_profile().constrain(limits);

        let observed_source = self.output_demands.retain_readmission_source(
            observed_source,
            limits,
            retained_program_basis.clone(),
        )?;
        // Selection spends this request's meter on success and failure; it
        // cannot mint a fresh allowance during refresh.
        let (mut selected, entry) = match selected_product {
            Some(selected) => self.select_output_producer_with_remaining_on_selected::<Family>(
                execution,
                selection_source.unwrap_or(&observed_source),
                profile_kind,
                retained_program_basis.as_deref(),
                selected,
            ),
            None => self.select_output_producer_with_remaining::<Family>(
                execution,
                selection_source.unwrap_or(&observed_source),
                profile_kind,
                retained_program_basis.as_deref(),
            ),
        }?;
        if selected_product.is_some() {
            source_selection.admit_selected_producer::<Family>(&selected)?;
        }

        // Selection already returned the exact immutable entry from its sole
        // matching iterator. Retain that entry for this demand's Interest.

        let installed_entry = std::sync::Arc::clone(entry);
        let source_epoch = observed_source.output_source_epoch().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            )
        })?;

        let key = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey::new(
            Family::IDENTITY,
            selected.identity.clone(),
            selected.applicability,
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
                execution,
                &selected.identity,
                output_binding,
                expected_key,
                &observed_source,
                source_epoch,
                source_scope,
                retained_program_basis.is_some(),
            )?
        } else {
            None
        };
        if let Some(readmitted) = restored {
            let restored = &readmitted.restored;
            let (interest, newly_adopted) = self.output_demands.admit_restored(
                key,
                source_scope,
                product_occurrence,
                admission_kind,
                restored.clone(),
                &readmitted.settlement,
            )?;
            if newly_adopted {
                self.record_restored_output(execution, source_scope, &readmitted)?;
            }
            let observed_source = observed_source.install(&interest, installed_entry.clone())?;
            return Ok(WorthQueryAdmittedOutputDemand {
                runtime_authority: self.runtime.authority_identity().as_u64(),
                schema_binding: self.installed_schema.binding_identity(),
                selected,
                installed_entry,
                observed_source,
                limits,

                producer_contacts_in_this_demand: 0,
                settled: false,
                admission_kind,
                retained_program_basis,
                progression_provenance: Default::default(),
                required_continuations: Default::default(),
                terminal_caller_denial: None,
                unpublished_selected_checkpoint: None,
                interest: Some(interest),
                settled_at_observation: None,
            });
        }
        let (interest, join_pause) = match performed_source {
            Some(source_commit) => (
                self.output_demands.admit_performed(
                    key,
                    &source_commit,
                    source_scope,
                    product_occurrence,
                )?,
                None,
            ),
            None => self.output_demands.admit(
                key,
                Some(SelectedOutputAdmission::new(
                    observed_source.selected_product_commit().ok_or_else(|| {
                        denial(
                            WorthQueryOutputDemandDenialKind::ForeignSource,
                            Family::IDENTITY,
                        )
                    })?,
                    selected
                        .exact_retained_output
                        .then_some(selected.retained_idempotency_key)
                        .flatten(),
                )),
                source_scope,
                product_occurrence,
                admission_kind,
                expected_source_commit,
                successor_of,
            )?,
        };
        // Rejoin the posture of the authenticated accepted row.
        selected.applicability = interest.key().applicability();
        let observed_source = observed_source.install(&interest, installed_entry.clone())?;
        crate::domain_computation::primary_graph::application_output_demand::observe_stable_join_admission(join_pause);
        Ok(WorthQueryAdmittedOutputDemand {
            runtime_authority: self.runtime.authority_identity().as_u64(),
            schema_binding: self.installed_schema.binding_identity(),
            selected,
            installed_entry,
            observed_source,
            limits,

            producer_contacts_in_this_demand: 0,
            settled: false,
            admission_kind,
            retained_program_basis,
            progression_provenance: Default::default(),
            required_continuations: Default::default(),
            terminal_caller_denial: None,
            unpublished_selected_checkpoint: None,
            interest: Some(interest),
            settled_at_observation: None,
        })
    }
}
