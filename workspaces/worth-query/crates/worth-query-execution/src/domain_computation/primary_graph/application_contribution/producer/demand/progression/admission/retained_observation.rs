//! A demand at a retained observation older than the head.
//!
//! Commits after that observation do not apply to it. It settles on the
//! output current at its own snapshot, with no producer contact and no
//! registry row: the lineage row its observation selects is verified at
//! that snapshot, with everything the row consumed, before the demand is
//! admitted, and its first advance answers with that settlement. An
//! observation whose output is not current there stops `Superseded`:
//! nothing is produced into the past.

use std::sync::Arc;

use worth_relational::facade::{runtime::ProjectionAspectScope, storage::RecordLifecycleState};

use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::{
    application_output_demand::DemandAdmissionKind,
    invariant_projection::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
    },
    output_lineage::RecordedSourceIdentity,
    output_reuse::require_installed_output_dependencies,
    WorthQueryApplicationBasisSelectionIdentity, WorthQueryApplicationOutputDemandSource,
    WorthQueryApplicationReadObservation, WorthQueryOutputDemandRecoveryPosture,
    WorthQueryOutputDemandSettlement, WorthQuerySelectedProductOperation,
};
use crate::domain_computation::{
    authorization::WorthQueryOperationScopeEntityBinding,
    execution_runtime::WorthQueryOutputDemandLimits,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Admit a demand of the output as `observation` reads it. At the head
    /// this is an ordinary demand. At an older retained observation the
    /// demand settles on the output current there, or stops `Superseded`.
    pub fn admit_output_demand_at<Family>(
        &self,
        source_result: WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        limits: WorthQueryOutputDemandLimits,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let at = self
            .select_application_read_observation(observation)
            .map_err(|error| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    format!("retained output basis unavailable: {error:?}"),
                )
            })?;
        let read =
            WorthQueryProductBranchReadIdentity::from_observation(at.product().observation());
        let head = self
            .on_branch(read.product_branch())
            .select()
            .map_err(|selection| {
                WorthQueryOutputDemandDenial::product_selection(
                    selection,
                    "output lifecycle basis could not be selected",
                )
            })?;
        let at_head =
            WorthQueryProductBranchReadIdentity::from_observation(head.product().observation())
                == read;
        drop(head);
        if at_head {
            drop(at);
            return self.admit_output_demand::<Family>(source_result, limits);
        }
        let (_, observed_source) = source_result.into_single_source().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "output source query did not return one owner-paired occurrence",
            )
        })?;
        self.admit_settled_at_observation::<Family>(observed_source, limits, &at, &read)
    }

    fn admit_settled_at_observation<Family>(
        &self,
        observed_source: WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
        limits: WorthQueryOutputDemandLimits,
        at: &WorthQuerySelectedProductOperation<'_, Schema>,
        read: &WorthQueryProductBranchReadIdentity,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let stop = |kind| denial(kind, Family::IDENTITY);
        let retry = |kind| {
            denial(kind, Family::IDENTITY)
                .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable)
        };
        if observed_source.runtime_authority != self.runtime.authority_identity().as_u64()
            || observed_source.schema_binding != self.installed_schema.binding_identity()
            || !matches!(
                &observed_source.selection,
                WorthQueryApplicationBasisSelectionIdentity::Product(source) if source == read
            )
        {
            return Err(stop(WorthQueryOutputDemandDenialKind::ForeignSource));
        }
        let installed = self
            .installed_schema
            .installed_query_binding::<Family::Source>()
            .map_err(|_| stop(WorthQueryOutputDemandDenialKind::ForeignSource))?;
        require_installed_output_dependencies(
            installed.query().output_dependencies(),
            &observed_source,
        )?;
        let limits = self.output_demand_resource_profile().constrain(limits);
        let owner = &self.primary_provider.graph.source_owner.invalidation_owner;
        // The selection and its verification share one meter, no larger than
        // the demand's declared source-currentness work.
        let mut remaining = owner.edit_admission_within(
            std::num::NonZeroUsize::new(limits.source_currentness_work())
                .ok_or_else(|| retry(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))?,
        );
        let observation = at.product().observation();
        let outputs = self
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .outputs_at_observation(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                WorthQueryOperationScopeEntityBinding::from_entity(observed_source.source_root()),
                observation,
                &self.installed_producers.family_output_bindings::<Family>(),
                observed_source.partition_identity(),
                &mut remaining,
            )
            .map_err(|()| retry(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))?;
        let snapshot = at.application_basis().snapshot_handle();
        let mut current = None;
        for output in outputs {
            let role = self
                .installed_producers
                .family_output_role::<Family>(output.binding)?;
            let source_matches =
                output
                    .stable
                    .source_identity()
                    .is_some_and(|identity| match identity {
                        RecordedSourceIdentity::Runtime(runtime) => {
                            runtime == observed_source.idempotency_identity()
                        }
                        RecordedSourceIdentity::Checkpoint(checkpoint) => {
                            checkpoint == observed_source.checkpoint_identity()
                        }
                    });
            let (Some(entity), Some(facts), Some(witness), true) = (
                output
                    .stable
                    .output_correspondence()
                    .active_entity_for_role(role),
                output.facts.as_ref(),
                output.native_output_witness.as_ref(),
                source_matches,
            ) else {
                continue;
            };
            let verified = self.primary_provider.graph.with_runtime(|relational| {
                let unavailable =
                    || stop(WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable);
                let truth = relational.read_truth();
                let live = truth
                    .project_snapshot(snapshot)
                    .ok_or_else(unavailable)?
                    .entity_record_with_projection_scope(
                        entity,
                        ProjectionAspectScope::empty(),
                        |record| Some(record.lifecycle()),
                    )
                    == Some(RecordLifecycleState::Live);
                let selected = truth
                    .positioned_snapshot(snapshot)
                    .map_err(|_| unavailable())?;
                if !live {
                    return Ok(false);
                }
                match ConsumedOutputEvidence::verify_at_observation(
                    output.stable.exact_settlement(),
                    facts,
                    &output.consumed_outputs,
                    output.verification_requirement,
                    witness,
                    owner,
                    relational,
                    snapshot,
                    &selected,
                    &mut remaining,
                ) {
                    Ok(verification) => Ok(verification == ConsumedOutputVerification::Current),
                    // What it consumed was not current at this observation.
                    Err(ConsumedOutputVerificationStop::PendingUpstream) => Ok(false),
                    Err(ConsumedOutputVerificationStop::WorkExhausted) => {
                        Err(retry(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded))
                    }
                    Err(ConsumedOutputVerificationStop::RetryCurrentness(_)) => {
                        Err(retry(WorthQueryOutputDemandDenialKind::PublicationStale))
                    }
                    Err(ConsumedOutputVerificationStop::Interrupted(event)) => Err(denial(
                        WorthQueryOutputDemandDenialKind::of_interruption(event.interruption()),
                        Family::IDENTITY,
                    )),
                    Err(ConsumedOutputVerificationStop::CapacityExhausted) => Err(retry(
                        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    )),
                    Err(ConsumedOutputVerificationStop::Unavailable) => Err(unavailable()),
                }
            })?;
            if verified {
                current = Some(output);
                break;
            }
        }
        let output = current.ok_or_else(|| stop(WorthQueryOutputDemandDenialKind::Superseded))?;
        let (mut selected, entry) = self
            .installed_producers
            .select_exact::<Family>(output.binding, None)?;
        selected.retained_resources = output.resources;
        selected.retained_idempotency_key = Some(output.stable.idempotency_key_identity());
        selected.retained_output_binding = Some(output.binding);
        let installed_entry = Arc::clone(entry);
        let settlement = WorthQueryOutputDemandSettlement::from_stable(
            self,
            &output.stable,
            at,
            &selected.identity,
            Family::IDENTITY,
            0, // This retained-read admission starts a new handle with no executions.
        );
        let mut admission = self.demand_request_admission();
        let observed_source = self
            .output_demands
            .retain_readmission_source(observed_source, limits, None, &mut admission)
            .map_err(|stop| self.starting_custody_stop(stop, &mut admission))?
            .without_row();
        Ok(WorthQueryAdmittedOutputDemand {
            runtime_authority: self.runtime.authority_identity().as_u64(),
            schema_binding: self.installed_schema.binding_identity(),
            selected,
            installed_entry,
            observed_source,
            limits,
            resources: output.resources,
            resources_validated: false,
            producer_contacts_in_this_demand: 0,
            checkpoint_readmission_work_units: 0,
            checkpoint_readmission_work_bound: 0,
            checkpoint_readmission_charged_preparation_bytes: 0,
            settled: false,
            admission_kind: DemandAdmissionKind::Ordinary,
            retained_program_basis: None,
            progression_provenance: Default::default(),
            required_continuations: Default::default(),
            unpublished_selected_checkpoint: None,
            interest: None,
            settled_at_observation: Some(settlement),
        })
    }
}
