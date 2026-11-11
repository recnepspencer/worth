//! Genuine World/Native co-publication of correction DATA; no Query handler is fabricated.
use super::*;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_relational::facade::{
    history::RelationalCommitReceipt, identity::EntityId, mvcc::RelationalTransactionIntent,
    transactions::*,
};
use worth_runtime_world::facade::*;
mod cases;
mod rows;

#[derive(Clone)]
struct Published {
    world: CompositeCommitIdentity,
    native: RelationalCommitReceipt,
}
struct Source {
    query: super::super::bootstrap_publication::Query,
    home: DirectoryHome,
    expected: ProductBranchObservation,
    account: EntityId,
    count: usize,
}
impl Source {
    fn fresh(path: &std::path::Path, phase: &WorthQueryBootstrapAdvancementPhase<'_>) -> Self {
        let (mut query, mut home, _) = prepared(path, phase, true, &AtomicUsize::new(0));
        query.publish_home_query_seed(phase).unwrap();
        let expected = home.select_query_root(&phase.execution_request()).unwrap();
        let account = administration::rows(
            home.relational().runtime(),
            &phase.execution_request(),
            "Account",
        )[0]
        .entity_id;
        Self {
            query,
            home,
            expected,
            account,
            count: 0,
        }
    }
    fn update(&self, balance: u64) -> WorkerIntentBatch {
        WorkerIntentBatch::new("actual correction lineage source data").push(
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: self.account,
                    fields: AspectFieldPatch::from_locator(
                        planned_single_field_locator(
                            AspectKey::new("AccountIdentity").unwrap(),
                            FieldKey::new("Balance").unwrap(),
                        ),
                        AspectValue::UInt64(balance),
                    ),
                },
            )),
        )
    }
    fn publish(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        batch: WorkerIntentBatch,
    ) -> Published {
        let cancellation = RuntimeWorldCancellationSource::new();
        let native = self.home.relational().runtime();
        let change = RelationalTransactionIntent::ordinary();
        let mut open = self
            .home
            .world()
            .publication_port()
            .open_publication_without_signal(
                execution,
                self.expected.clone(),
                change.clone(),
                &cancellation.token(),
                None,
            );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let opened = loop {
            match open {
                WorldOpenAttemptOutcome::Opened(opened) => break opened,
                WorldOpenAttemptOutcome::Pending(pending) => {
                    assert!(std::time::Instant::now() < deadline);
                    open = pending.advance(execution);
                }
                other => panic!("actual World open: {other:?}"),
            }
        };
        let mut transaction = native
            .begin_branch_transaction(self.expected.basis().relational_basis(), change)
            .unwrap();
        transaction
            .push_batch(batch, execution.allocation_policy())
            .unwrap();
        let proposal = transaction.validate(execution, native).unwrap();
        let prepared = opened.prepare_native(native, execution, proposal).unwrap();
        let mut outcome = self
            .home
            .world()
            .publication_port()
            .execute_durable_without_signal(*execution, prepared, &cancellation.token());
        let performed = loop {
            match outcome {
                WorldDurablePublicationOutcome::Performed(performed) => break performed,
                WorldDurablePublicationOutcome::Pending(pending) => {
                    assert!(std::time::Instant::now() < deadline);
                    outcome = pending.advance(execution);
                }
                other => panic!("actual co-publication: {other:?}"),
            }
        };
        let result = Published {
            world: performed.commit().identity().clone(),
            native: performed
                .component_results()
                .relational_commit_result()
                .unwrap()
                .semantic_result()
                .receipt()
                .clone(),
        };
        self.expected = performed.consume().take_successor_observation().unwrap();
        self.count += 1;
        result
    }
    fn correction(&mut self, execution: &ExecutionRequest<'_, '_>, row: &rows::Row) -> Published {
        let batch = self.update(if row.selected.is_some() { 8 } else { 7 });
        let batch = rows::append(
            self.home.relational().runtime(),
            execution,
            batch,
            self.count,
            row,
        );
        self.publish(execution, batch)
    }
    fn first_fact(&self, execution: &ExecutionRequest<'_, '_>) -> EntityId {
        administration::rows(
            self.home.relational().runtime(),
            execution,
            "worth-query-aftermath-causality",
        )[0]
        .entity_id
    }
    fn close(self, execution: &ExecutionRequest<'_, '_>) {
        let Self {
            query,
            home,
            expected,
            ..
        } = self;
        drop((query, expected));
        close_world(home, execution);
    }
}
fn correction_denial(
    refused: &WorthQueryApplicationHomeOpenRefused<HomeIdentitySchema, HomeProgram>,
) -> &worth_query_execution::facade::primary_graph::QueryHomeCorrectionLineageDenial {
    use worth_query_host::facade::HomeCustodyReadmissionDenial;
    match refused {
        WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
            cause:
                HomeCustodyReadmissionDenial::Query(QueryHomeCustodyReadmissionDenial::Correction(
                    cause,
                )),
            ..
        } => cause,
        other => panic!("joined original gate must retain the actual Home: {other:?}"),
    }
}

/// Workflow readmission follows the selected World source after 6ea7a7126e.
/// Publish its malformed DATA there, rather than treating a private Native tail
/// as a published workflow. No workflow execution or admission is fabricated.
pub(super) fn co_published_required_row(
    path: &std::path::Path,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    name: &str,
) -> (worth_relational::facade::identity::KindId, EntityId) {
    let execution = phase.execution_request();
    let mut source = Source::fresh(path, phase);
    let (kind, batch) =
        administration::required_row_batch(source.home.relational().runtime(), &execution, name);
    source.publish(&execution, batch);
    let rows = administration::rows(source.home.relational().runtime(), &execution, name);
    let [row] = rows.as_slice() else {
        panic!("one actual World-co-published owner row");
    };
    let id = row.entity_id;
    source.close(&execution);
    (kind, id)
}
