//! Prepare the existing canonical transaction kernel without moving this cell.
use super::SignalBranchCellState;
use crate::branch::owner_services::durable_publication::prepared::SignalPreparedDurableAdvance;
use crate::branch::owner_services::durable_publication::SignalDurableAdvanceDenial as D;
use crate::data::error::SignalError;
use crate::logic::transaction::{SignalCanonicalCallerUnwind, SignalTransaction};
use worth_execution::ExecutionRequest;

impl<Dm, I, T> SignalBranchCellState<Dm, I, T>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn prepare_durable_advance<E, Ctx, F>(
        &mut self,
        execution: ExecutionRequest<'_, '_>,
        runtime_ctx: &mut Ctx,
        scope: super::super::conditional_execution::SignalConditionalOperationScopeBinding,
        recorded_nodes: worth_execution::ExecutionOrderedMap<u32, u32>,
        apply: F,
    ) -> Result<Result<SignalPreparedDurableAdvance<Dm, I, T>, D>, SignalCanonicalCallerUnwind>
    where
        F: FnOnce(&mut SignalTransaction<'_, Dm, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        let predecessor = match self.durable_descriptor() {
            Ok(Some(descriptor)) => descriptor,
            Ok(None) => return Ok(Err(D::Source(
                super::super::durable_source_port::SignalDurableSourceDenial::SourceNotInstalled,
            ))),
            Err(denial) => return Ok(Err(D::Frame(denial))),
        };
        let mut candidate = self.state.stage_durable_successor();
        candidate
            .graph_mut()
            .install_recorded_node_issuance(recorded_nodes);
        let mut generation = self.head_generation;
        let mut restore_snapshot = self.restore_snapshot_id;
        let transaction = match candidate.execute_canonical_transaction(
            execution,
            &mut generation,
            &mut restore_snapshot,
            Some(scope),
            runtime_ctx,
            apply,
        )? {
            Ok(transaction) => transaction,
            Err(error) => {
                return Ok(Err(D::Mutation(
                    crate::branch::SignalBranchAdvanceDenial::MutationFailedNoMovement { error },
                )))
            }
        };
        let source_changed = !self.state.shares_authored_storage_with(&candidate);
        if source_changed
            && !candidate
                .graph()
                .preserves_conditional_definitions_of(self.state.graph())
        {
            return Ok(Err(D::ConditionalDefinitionPublicationRequired));
        }
        let observation = match self.observation_for(
            generation,
            self.handle.head_snapshot_id,
            restore_snapshot,
        ) {
            Ok(observation) => observation,
            Err(error) => {
                return Ok(Err(D::Mutation(
                    crate::branch::SignalBranchAdvanceDenial::MutationFailedNoMovement { error },
                )))
            }
        };
        Ok(Ok(SignalPreparedDurableAdvance {
            state: candidate,
            predecessor,
            observation,
            generation,
            restore_snapshot,
            source_changed,
            transaction,
        }))
    }
}

impl<Dm, I, T> SignalBranchCellState<Dm, I, T>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(in crate::branch::owner_services) fn commit_durable_advance(
        &mut self,
        proof: worth_proof::DurablyCommitted<
            super::super::durable_publication::medium_binding::SignalCommittedBranchAdvance<
                Dm,
                I,
                T,
            >,
        >,
    ) -> (
        Box<crate::logic::transaction::BranchState<Dm, I, T>>,
        Box<crate::logic::transaction::TransactionResult>,
        crate::branch::SignalBranchObservation,
        crate::durability::publication_binding::SignalCommittedAdvanceRecord,
    ) {
        proof.consume(|subject| {
            let (mut prepared, record) = subject.into_original();
            let artifact = record.artifact();
            assert_eq!(
                self.durable_descriptor()
                    .expect("canonical descriptor remains valid")
                    .as_ref(),
                Some(&prepared.predecessor),
                "the original per-branch reservation excludes a second movement"
            );
            assert_eq!(
                artifact.descriptor().generation().ordinal(),
                prepared.generation
            );
            assert_eq!(
                artifact.descriptor().branch(),
                prepared.predecessor.branch()
            );
            std::mem::swap(&mut self.state, &mut prepared.state);
            self.head_generation = prepared.generation;
            self.restore_snapshot_id = prepared.restore_snapshot;
            self.conditional_execution_basis = None;
            self.install_durable_coordinates(artifact.descriptor().clone());
            (
                prepared.state,
                prepared.transaction,
                prepared.observation,
                record,
            )
        })
    }
}
