use worth_query_installation::facade::ApplicationSchema;

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
    ) -> Result<
        Option<
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryRestoredAcceptedOutput,
        >,
        WorthQueryOutputDemandDenial,
    >{
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
        let Some(fact_bytes) = readmitted.checkpoint.producer_facts.as_deref() else {
            return Ok(None);
        };
        let Some(observed_source_facts) =
            crate::domain_computation::primary_graph::application_checkpoint::decode_producer_facts_for_wire_version(
                fact_bytes,
                readmitted.checkpoint.producer_fact_wire_version,
            )
            .map_err(|error| denial(WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage, error))?
        else {
            return Ok(None);
        };
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
        // variable branch names used by the full Product-read comparison.
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
        if !source_read.matches_observation(selected_observation) {
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
        let Some(witness) = crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness::from_checkpoint_facts(
            &readmitted.correspondence,
            graph.layout(),
            &observed_source_facts,
            owner,
            admission,
        ).map_err(restoration_resource_denial)? else {
            return Ok(None);
        };
        let current = graph
            .with_runtime(|relational| {
                witness
                    .get()
                    .expect("checkpoint constructor sealed its witness")
                    .checkpoint_facts_current_in(
                        relational,
                        selected.application_basis().snapshot_handle(),
                        &observed_source_facts,
                        admission,
                    )
            })
            .map_err(restoration_resource_denial)?;
        if !current {
            return Ok(None);
        }
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
        Ok(Some(
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryRestoredAcceptedOutput {
                checkpoint: readmitted.checkpoint.clone(),
                correspondence: std::sync::Arc::clone(&readmitted.correspondence),
                observation: observation.clone(),
                source_scope,
                source_identity: source_epoch.checkpoint_identity(),
                observed_source_facts,
                native_output_witness: Some(witness),
            },
        ))
    }

    pub(super) fn record_restored_output(
        &self,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        restored: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRestoredAcceptedOutput,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let output_binding = restored.correspondence.binding_type().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                "checkpoint output has no installed output binding",
            )
        })?;
        self.primary_provider
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
                std::sync::Arc::clone(&restored.observed_source_facts),
                restored.checkpoint.resources,
                restored.native_output_witness.as_ref().map(std::sync::Arc::clone),
            );
        Ok(())
    }
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
