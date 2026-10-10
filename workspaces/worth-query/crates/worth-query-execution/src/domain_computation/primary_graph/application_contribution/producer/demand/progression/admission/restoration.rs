use worth_query_installation::facade::ApplicationSchema;

mod fact_denial;

use super::{
    denial, WorthQueryObservedSource, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn readmit_checkpoint_output<Query>(
        &self,
        producer: &str,
        output_binding: std::any::TypeId,
        expected_idempotency_key: [u8; 32],
        observed_source: &WorthQueryObservedSource<Query>,
        source_epoch: crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<Option<ReadmittedOutput>, WorthQueryOutputDemandDenial> {
        let preparation_work_before = admission.charged_work();
        let preparation_bytes_before = admission.charged_bytes();
        let checkpoint_source = source_epoch.checkpoint_identity();
        let Some(readmitted) = self
            .recovered_outputs
            .matching_source_partition(source_scope, observed_source.partition_identity())
            .find(|readmitted| {
                readmitted.checkpoint.producer == producer
                    && readmitted.checkpoint.source == checkpoint_source.bytes()
                    && readmitted.checkpoint.idempotency_key == expected_idempotency_key
                    && readmitted.correspondence.binding_type() == Some(output_binding)
            })
        else {
            return Ok(None);
        };
        let authority = self
            .product_runtime
            .recovered_root_authority
            .as_ref()
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "checkpoint output has no World recovery adoption authority",
                )
            })?;
        let observation = authority.product_branch();
        if observed_source.selected_product_occurrence()
            != Some(observation.lifecycle_incarnation())
        {
            return Ok(None);
        }
        // A restored row posts the settlement candidate selection recorded.
        let Some(settlement) = self
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .restored_settlement_identity(
                output_binding,
                self.runtime.authority_identity().as_u64(),
                self.installed_schema.binding_identity(),
                source_scope,
                observation,
                readmitted.checkpoint.source_partition,
            )
        else {
            return Ok(None);
        };
        let Some(fact_bytes) = readmitted.checkpoint.producer_facts.as_deref() else {
            return Ok(None);
        };
        let (observed_source_facts, computation_source) =
            crate::domain_computation::primary_graph::application_checkpoint::decode_checkpoint_computation(
                fact_bytes,
                readmitted.checkpoint.producer_fact_wire_version,
                None,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(fact_denial::decode_denial)?.into_parts();
        // The checkpoint's expected output revisions are original facts, not
        // revisions projected from the recovered root. Select the disclosed
        // source's current Product and verify both the original revisions and
        // decoded producer observations before admitting a restored row.
        admission
            .charge_external_work(8)
            .map_err(restoration_resource_denial)?;
        let source_read = match &observed_source.selection {
            crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(source_read) => source_read,
            crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Relational => return Ok(None),
        };
        let selected = self
            .on_branch(source_read.product_branch())
            .select()
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::Superseded, ""))?;
        admission
            .charge_external_work(4)
            .map_err(restoration_resource_denial)?;
        let selected_observation = selected.product().observation();
        // Both identity/header paths are inspected before measuring the
        // variable branch names used by the branch-occurrence comparison.
        admission
            .charge_external_work(8)
            .map_err(restoration_resource_denial)?;
        let source_width = source_read.branch_identity().name().as_str().len();
        let selected_width = selected_observation.branch_identity().name().as_str().len();
        let identity_copies =
            std::mem::size_of::<crate::basis::WorthQueryProductBranchReadIdentity>()
                .checked_mul(2)
                .ok_or_else(|| {
                    restoration_resource_denial(
                        worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
                    )
                })?;
        let source_comparison = source_width
            .checked_add(selected_width)
            .and_then(|work| work.checked_add(identity_copies))
            .and_then(|work| work.checked_add(8))
            .and_then(|work| u64::try_from(work).ok())
            .ok_or_else(|| {
                restoration_resource_denial(
                    worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
                )
            })?;
        admission
            .charge_external_work(source_comparison)
            .map_err(restoration_resource_denial)?;
        // Another output may have advanced this branch's head since discovery.
        // The checkpoint dependency comparison below decides currentness at
        // that head; the supplied commit is not itself a dependency.
        if !source_read.same_branch_occurrence_observation(selected_observation) {
            return Ok(None);
        }
        let comparison = worth_runtime_world::facade::CurrentProductHead::comparison_work_bound(
            selected_observation,
        );
        admission
            .charge_external_work(
                u64::try_from(comparison.checked_add(5).ok_or_else(|| {
                    restoration_resource_denial(
                        worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
                    )
                })?)
                .map_err(|_| {
                    restoration_resource_denial(
                        worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
                    )
                })?,
            )
            .map_err(restoration_resource_denial)?;
        let mut admission_stop = None;
        let current_head = self
            .product_runtime
            .owner
            .observation_port()
            .while_product_branch_current_admitted(
                selected_observation,
                (),
                &mut |work| match admission.charge_external_work(work) {
                    Ok(()) => true,
                    Err(stop) => {
                        admission_stop = Some(stop);
                        false
                    }
                },
                |(), _head| true,
            );
        match current_head {
            Ok(true) => {}
            Ok(false)
            | Err(worth_runtime_world::facade::ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(())) => {
                return Ok(None);
            }
            Err(worth_runtime_world::facade::ProductBranchCurrentnessFailure::PreparationDenied(())) => {
                return Err(restoration_resource_denial(admission_stop.expect("World preserves the actual admission refusal")));
            }
            Err(worth_runtime_world::facade::ProductBranchCurrentnessFailure::AdmissionDenied { .. }) => {
                return Ok(None);
            }
        }
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                "",
            )
        })?;
        let owner = &self.primary_provider.graph.source_owner.invalidation_owner;
        // World registry/header comparisons debit their owner-declared bounds
        // before traversal. The remaining witness ceiling covers its execution.
        let preparation_prefix = admission.charged_work() - preparation_work_before;
        let Some(witness) = crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness::from_checkpoint_facts(
            &readmitted.correspondence,
            graph.layout(),
            &observed_source_facts,
            owner,
            admission,
        ).map_err(restoration_resource_denial)? else {
            return Ok(None);
        };
        let (current, verified_at) = graph.with_runtime(|relational| {
            let snapshot = selected.application_basis().snapshot_handle();
            let current = witness
                .get()
                .expect("checkpoint constructor sealed its witness")
                .checkpoint_facts_current_in(
                    relational,
                    snapshot,
                    &observed_source_facts,
                    admission,
                );
            (
                current,
                relational.read_truth().positioned_snapshot(snapshot).ok(),
            )
        });
        if !current.map_err(restoration_resource_denial)? {
            return Ok(None);
        }
        // Qualification runs only after original completeness/currentness has
        // accepted. Missing, conflicting or unsupported proof retains the same
        // conservative Fresh decision without a needless ceiling traversal.
        let calculation_start = admission.charged_work();
        let witness_work_bound = crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness::checkpoint_comparison_work_bound(
            &readmitted.correspondence,
            graph.layout(),
            &observed_source_facts,
            admission,
        ).map_err(restoration_resource_denial)?;
        let calculation_work = admission.charged_work() - calculation_start;
        // The verified Arc travels through the restored carrier, registry
        // record and lineage's single witness cell after admission. Fund its
        // initialized Option moves before either owner installs the row.
        type VerifiedWitness = std::sync::Arc<
            std::sync::OnceLock<
                crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness,
            >,
        >;
        let witness_moves = std::mem::size_of::<Option<VerifiedWitness>>()
            .checked_mul(4)
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| {
                restoration_resource_denial(
                    worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
                )
            })?;
        admission
            .charge_external_work(witness_moves)
            .map_err(restoration_resource_denial)?;
        let preparation_work_bound = preparation_prefix
            .checked_add(witness_work_bound)
            .and_then(|work| work.checked_add(calculation_work))
            .and_then(|work| work.checked_add(witness_moves))
            .ok_or_else(|| {
                restoration_resource_denial(
                    worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
                )
            })?;
        let preparation_work_units = admission.charged_work() - preparation_work_before;
        if preparation_work_units > preparation_work_bound {
            return Err(restoration_resource_denial(
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    required: preparation_work_units,
                    maximum: preparation_work_bound,
                },
            ));
        }
        Ok(Some(ReadmittedOutput {
            computation_source,
            restored: crate::domain_computation::primary_graph::application_output_demand::WorthQueryRestoredAcceptedOutput {
                checkpoint: readmitted.checkpoint.clone(),
                correspondence: std::sync::Arc::clone(&readmitted.correspondence),
                observation: observation.clone(),
                source_scope,
                source_identity: source_epoch.checkpoint_identity(),
                observed_source_facts,
                native_output_witness: Some(witness),
            },
            settlement,
            verified_at,
            preparation_work_units,
            preparation_work_bound,
            charged_preparation_bytes: admission.charged_bytes() - preparation_bytes_before,
        }))
    }

    pub(super) fn record_restored_output(
        &self,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        readmitted: &ReadmittedOutput,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let restored = &readmitted.restored;
        let output_binding = restored.correspondence.binding_type().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                "checkpoint output has no installed output binding",
            )
        })?;
        let recorded = self
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .record_restoration(
                output_binding,
                self.runtime.authority_identity().as_u64(),
                self.installed_schema.binding_identity(),
                source_scope,
                &restored.observation,
                std::sync::Arc::clone(&restored.correspondence),
                crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(
                    restored.source_identity,
                ),
                restored.checkpoint.source_partition,
                restored.checkpoint.producer_dependency,
                restored.checkpoint.idempotency_key,
                readmitted.computation_source.retain_facts(std::sync::Arc::clone(&restored.observed_source_facts)),
                restored.checkpoint.resources,
                restored.native_output_witness.as_ref().map(std::sync::Arc::clone),

        );
        let witness = restored
            .native_output_witness
            .as_ref()
            .and_then(|witness| witness.get());
        if let (Some(recorded), Some(witness), Some(head)) =
            (recorded, witness, readmitted.verified_at.as_ref())
        {
            // Readmission is this output's one full comparison. Recording it
            // as marks lets the next clean demand read them instead; a stop
            // here leaves the output requiring verification.
            let source_owner = &self.primary_provider.graph.source_owner;
            if let Some(facts) = recorded.facts.for_comparison().filter(|_| {
                use crate::domain_computation::primary_graph::output_lineage::HeadCellRegistrationStop;
                use worth_relational::facade::mvcc::PublicationCompanionRegistrationStop as NativeStop;
                match source_owner.mint_mark_cell_at_head(head.branch_id(), admission) {
                    Ok(()) => true,
                    Err(HeadCellRegistrationStop::Admission(_) | HeadCellRegistrationStop::LookupChanged) => false,
                    Err(HeadCellRegistrationStop::Native(NativeStop::HeadCellPublicationContended)) => false,
                    Err(HeadCellRegistrationStop::Native(
                        NativeStop::OwnerUnavailable | NativeStop::PublicationPending
                        | NativeStop::RebindRequired | NativeStop::Superseded
                        | NativeStop::IdentityExhausted | NativeStop::ForeignRuntime
                        | NativeStop::HeadUnavailable)) => false,
                }
            }) {
                let _ = source_owner.invalidation_owner.establish_verified_root(
                    head,
                    &recorded.identity,
                    &facts,
                    witness,
                    admission,
                );
            }
        }
        Ok(())
    }
}

/// A checkpoint output whose facts and output were compared in full at
/// `verified_at`, the head of its branch.
pub(super) struct ReadmittedOutput {
    computation_source: crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence,
    pub(super) restored: crate::domain_computation::primary_graph::application_output_demand::WorthQueryRestoredAcceptedOutput,
    pub(super) settlement: std::sync::Arc<
        crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
    >,
    verified_at: Option<worth_relational::facade::runtime::PositionedRelationalSnapshot>,
    pub(super) preparation_work_units: u64,
    pub(super) preparation_work_bound: u64,
    pub(super) charged_preparation_bytes: u64,
}

fn restoration_resource_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    denial(
        match stop {
            Stop::PreparationMemoryExhausted { .. } | Stop::PreparationMemoryCounterOverflow => {
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
            }
            _ => WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        },
        "",
    )
}
