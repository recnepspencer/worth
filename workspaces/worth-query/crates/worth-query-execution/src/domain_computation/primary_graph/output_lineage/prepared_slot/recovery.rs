#[cfg(test)]
use super::PreparedComputationCustody;
use crate::domain_computation::primary_graph::application_contribution::SealedComputationRetention;
use crate::equality_type_id::EqualityTypeId as TypeId;

use worth_runtime_world::facade::PlannedProductReferenceSuccessor;

use super::PreparedOutputLineageSlot;
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;

use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

/// Completed read proofs and the sealed computation move across a World recovery plan. The cancelled address
/// and its retained capacity stay with the original slot's Drop path.
pub(in crate::domain_computation::primary_graph) struct PreparedLineageRecoveryMetadata {
    computation: SealedComputationRetention,
    prior_computation:
        Option<crate::domain_computation::primary_graph::output_lineage::PriorComputationRecord>,
    handler: Option<
        crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary,
    >,
    decision_reuse: Option<
        crate::domain_computation::primary_graph::application_attempt::CompletedDecisionReuseProof,
    >,
    reuse: Option<crate::domain_computation::primary_graph::output_lineage::PreparedInputReuseKey>,
}

impl PreparedOutputLineageSlot {
    /// Reuse is allowed only for this attempt's exact still-vacant address.
    pub(in crate::domain_computation::primary_graph) fn matches_planned_successor(
        &self,
        scope: &WorthQueryOperationScopeBinding,
        output_binding: TypeId,
        partition: Option<[u8; 32]>,
        planned: &PlannedProductReferenceSuccessor,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        Ok(!self.filled
            && self.record_cell.get().is_none()
            && self.partition_cell.get().is_none()
            && self.source.runtime_authority == scope.runtime_authority()
            && &self.source.schema == scope.binding_identity()
            && self.source.scope == scope.scope()
            && self.output_binding_type == output_binding
            && self.coordinate.occurrence == planned.occurrence()
            && self.coordinate.generation == planned.generation().get()
            && self.partition == partition
            && self.identity.source() == &self.source
            && self.identity.address()
                == (
                    self.coordinate.occurrence,
                    self.coordinate.generation,
                    self.identity.slot(),
                ))
    }

    pub(in crate::domain_computation::primary_graph) fn into_recovery_metadata(
        mut self,
    ) -> PreparedLineageRecoveryMetadata {
        let metadata = PreparedLineageRecoveryMetadata {
            computation: self.computation.take(),
            prior_computation: self.prior_computation.take(),
            handler: self.completed_handler_facts.take(),
            decision_reuse: self.completed_decision_reuse.take(),
            reuse: self.prepared_input_reuse_key.take(),
        };
        drop(self);
        metadata
    }

    pub(in crate::domain_computation::primary_graph) fn retain_recovery_metadata(
        &mut self,
        mut metadata: PreparedLineageRecoveryMetadata,
    ) {
        assert!(self.completed_handler_facts.is_none());
        assert!(self.completed_decision_reuse.is_none());
        assert!(self.prepared_input_reuse_key.is_none());

        self.computation.assign(metadata.computation);
        self.prior_computation = metadata.prior_computation.take();
        self.completed_handler_facts = metadata.handler.take();
        self.completed_decision_reuse = metadata.decision_reuse.take();
        self.prepared_input_reuse_key = metadata.reuse.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::application_contribution::sealed_run_for_lineage_test;
    use crate::domain_computation::primary_graph::application_contribution::PriorAbsence;
    use crate::domain_computation::primary_graph::application_contribution::SealedComputationRetention;
    use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
    use crate::domain_computation::primary_graph::output_lineage::ProductCoordinate;
    use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
    use crate::domain_computation::primary_graph::output_lineage::SemanticSource;
    use crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
    use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    #[test]
    fn world_recovery_metadata_carries_the_original_computation_and_absence() {
        let _observation =
            crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        let world = installed_authorization_world(true);
        let product = world
            .application
            .product_runtime()
            .admit_product_branch(world.application.product_runtime().default_branch())
            .unwrap();
        let source = SemanticSource {
            runtime_authority: world.application.runtime.authority_identity().as_u64(),
            schema: world.application.installed_schema.binding_identity().clone(),
            scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
                worth_relational::facade::identity::EntityId::new(worth_relational::facade::identity::PartitionId::main(), 1, 1)),
            output_binding: OutputBindingIdentity::declared("()"),
        };
        let coordinate = ProductCoordinate {
            occurrence: product.observation().lifecycle_incarnation(),
            generation: product.observation().reference_generation().get() + 1,
        };
        let owner = Arc::new(Mutex::new(WorthQueryApplicationOutputLineage::default()));
        // These are private vacant addresses, never publication authority.
        // Reserve their ledger custody before constructing their test slots.
        let generation = std::cell::Cell::new(coordinate.generation);

        let slot = || {
            let coordinate = ProductCoordinate {
                generation: generation.get(),
                ..coordinate
            };
            generation.set(generation.get() + 1);
            let mut lineage = owner.lock().unwrap();
            let identity = RecordedSettlementIdentity::retain(&source, coordinate, 0);
            let record_cell = Arc::new(OnceLock::new());
            lineage
                .by_source
                .entry(source.clone())
                .or_default()
                .entry(coordinate.occurrence)
                .or_default()
                .insert(coordinate.generation, vec![Arc::clone(&record_cell)]);
            let partition_cell =
                lineage
                    .partition_index
                    .insert_vacancy(source.clone(), coordinate, None);
            PreparedOutputLineageSlot {
                owner: Arc::clone(&owner),
                output_binding_type: TypeId::of::<()>(),
                source: source.clone(),
                coordinate,
                partition: None,
                identity: Arc::clone(&identity),
                record_cell,
                partition_cell,
                cancellation: Some(Box::new(super::super::CancelledLineageSlot {
                    identity,
                    partition: None,
                    next: None,
                })),
                completed_handler_facts: None,
                completed_decision_reuse: None,
                prepared_input_reuse_key: None,
                native_output_witness: None,
                required_publication: None,
                native_prior_checkpoint: None,
                prior_computation: None,
                filled: false,
                computation: PreparedComputationCustody::Unassigned,
            }
        };
        for computation in [
            SealedComputationRetention::Produced(sealed_run_for_lineage_test()),
            SealedComputationRetention::Absent(PriorAbsence::Stopped),
        ] {
            let produced = matches!(&computation, SealedComputationRetention::Produced(_));
            let mut original = slot();
            original.retain_computation(computation, None);
            let metadata = original.into_recovery_metadata();
            let mut replacement = slot();
            replacement.retain_recovery_metadata(metadata);
            assert_eq!(
                matches!(
                    &replacement.computation,
                    PreparedComputationCustody::Assigned(SealedComputationRetention::Produced(_))
                ),
                produced
            );
            if !produced {
                assert!(matches!(
                    replacement.computation,
                    PreparedComputationCustody::Assigned(SealedComputationRetention::Absent(
                        PriorAbsence::Stopped
                    ))
                ));
            }
        }
    }
}
