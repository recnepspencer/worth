//! Genuine Section Complete capture, actual selection and weak owner lifetime.
use super::*;
use crate::{RelationalSectionCheckpointDenial as Denial, RelationalSectionCloseDenial};
use worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial as Ack;
use worth_relational::facade::runtime::RelationalRuntimeCloseDenial;

fn section(
    execution: &ExecutionRequest<'_, '_>,
) -> (
    tempfile::TempDir,
    crate::RelationalStoreSection,
    Arc<Selected>,
) {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(test_store::initialize(directory.path()));
    let source = StoreColdSource::new(serving.clone());
    let selected = Arc::new(Selected::default());
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "complete.section".into(),
            schema_id: SchemaId("complete.section".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let producer = runtime(&serving, &source, &selected, &schema);
    let (_, basis) = producer
        .observe_branch(&producer.main_branch_identity())
        .unwrap();
    let mut transaction = producer
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("complete-original").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: "complete-original".into(),
                    fields: Default::default(),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &producer).unwrap();
    producer
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    drop(committed);
    drop(basis);
    let section = crate::RelationalStoreSection {
        source: None,
        runtime: Arc::new(producer),
        cold_source: source,
        serving,
        checkpoint_selection: selected.clone(),
    };
    (directory, section, selected)
}

#[test]
fn genuine_complete_section_retains_capture_and_resumes_same_selection_ack() {
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = &ExecutionRequest::serial(&serial);
    let (_directory, section, selected) = section(execution);
    let prepared = section.prepare_complete_checkpoint(execution).unwrap();
    assert_eq!(prepared.covered().0, 1);
    prepared.validate_publication().unwrap();
    let unknown = execution
        .run_native_with_checkpoint(|context| prepared.acknowledge_selected(execution, context))
        .unwrap()
        .0
        .unwrap_err();
    assert!(matches!(
        unknown,
        Denial::Acknowledgement(Ack::SelectionNotKnown)
    ));
    *selected.subject.lock().unwrap() = prepared.publication_subject().cloned();
    let (members, member_memory) = prepared.prepare_members(&[], execution).unwrap();
    assert!(
        !members.is_empty(),
        "the actual original canonical frame is a selected member"
    );
    let published = section
        .serving
        .anchored_records()
        .unwrap()
        .publish(
            AnchoredRecordPublication::new(
                AnchoredRecordName::new("home.checkpoint").unwrap(),
                0,
                prepared.bytes().to_vec(),
                worth_store_testkit_store::fixture::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_members(members),
        )
        .unwrap();
    *selected.receipt.lock().unwrap() = Some(match published {
        AnchoredRecordPublished::Published(receipt)
        | AnchoredRecordPublished::AlreadyPublished(receipt) => receipt,
    });
    execution
        .run_native_with_checkpoint(|context| prepared.acknowledge_selected(execution, context))
        .unwrap()
        .0
        .unwrap();
    execution
        .run_native_with_checkpoint(|context| prepared.acknowledge_selected(execution, context))
        .unwrap()
        .0
        .unwrap();
    assert_eq!(
        prepared.covered().0,
        1,
        "ACK retries keep the same captured result"
    );
    drop(member_memory);
    drop(prepared);
}

#[test]
fn weak_complete_preparation_preserves_close_stop_resume_and_owner_loss() {
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = &ExecutionRequest::serial(&serial);
    let (_directory, section, selected) = section(execution);
    let preparation = section.complete_checkpoint_preparation();
    let idle = preparation.clone();
    let stopped = CancellationSource::new();
    let stopped_serial =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), stopped.token(), None);
    stopped.cancel();
    assert!(matches!(
        preparation.prepare(&ExecutionRequest::serial(&stopped_serial)),
        Err(Denial::Execution(
            worth_execution::WorkCeilingDenial::Stopped(_)
        ))
    ));
    let prepared = preparation.prepare(execution).unwrap();
    let retirement = crate::PreparedRelationalSectionClose::prepare(execution).unwrap();
    let (section, retirement, cause) = match section.prepare_close(retirement, execution) {
        Err(refused) => refused,
        Ok(_) => panic!("the actual retained Complete capture must veto Native close"),
    };
    assert!(matches!(
        cause,
        RelationalSectionCloseDenial::Native(RelationalRuntimeCloseDenial::ReadCustodyRetained)
    ));
    drop(prepared);
    let closing = section
        .prepare_close(retirement, execution)
        .unwrap_or_else(|(_, _, cause)| {
            panic!("idle weak preparation cannot retain close custody: {cause:?}")
        });
    assert!(
        matches!(preparation.prepare(execution), Err(Denial::Capture(_))),
        "stopped Native owner cannot prepare through a weak port"
    );
    let (section, _retirement) = closing
        .resume()
        .unwrap_or_else(|_| panic!("reversible preparation retains the same serving owner"));
    let prepared = idle.prepare(execution).unwrap();
    assert_eq!(prepared.covered().0, 1);
    drop(prepared);
    drop(selected);
    drop(section);
    assert!(matches!(
        preparation.prepare(execution),
        Err(Denial::OwnerUnavailable)
    ));
}

#[test]
fn owned_visibility_cache_retires_before_external_capture_recheck_and_source_seal() {
    use worth_relational::facade::runtime::RelationalSourceCloseProgress as Progress;
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = &ExecutionRequest::serial(&serial);
    let (_directory, section, _selected) = section(execution);
    let native = section.runtime();
    let (_, basis) = native
        .observe_branch(&native.main_branch_identity())
        .unwrap();
    let snapshot = native
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    drop(basis);
    drop(
        native
            .read_truth()
            .read_snapshot(&snapshot, execution)
            .unwrap()
            .unwrap(),
    );
    native.snapshots().release_snapshot(&snapshot).unwrap();
    drop(snapshot);
    let preparation = section.complete_checkpoint_preparation();
    let capture = preparation.prepare(execution).unwrap();
    let serving = section.serving.clone();
    let retirement = crate::PreparedRelationalSectionClose::prepare(execution).unwrap();
    let mut closing = section
        .prepare_close(retirement, execution)
        .unwrap_or_else(|(_, _, cause)| panic!("the actual cache owns its read pins: {cause:?}"));
    let mut refused = false;
    for _ in 0..1000 {
        match closing.advance(execution, &serving) {
            Ok(Progress::Advanced) => {}
            Err(RelationalSectionCloseDenial::Native(
                RelationalRuntimeCloseDenial::ReadCustodyRetained,
            )) => {
                refused = true;
                break;
            }
            result => panic!("the original external capture must retain close: {result:?}"),
        }
    }
    assert!(
        refused,
        "the real capture still owns its original Native custody"
    );
    let mut closing = match closing.resume() {
        Err(closing) => closing,
        Ok(_) => panic!("retiring the actual cache forbids service restoration"),
    };
    assert!(matches!(
        preparation.prepare(execution),
        Err(Denial::Capture(_))
    ));
    assert_eq!(capture.covered().0, 1);
    drop(capture);
    for _ in 0..1000 {
        match closing.advance(execution, &serving) {
            Ok(Progress::Advanced) => {}
            Ok(Progress::Complete) => return,
            result => panic!("same original owner must finish after capture release: {result:?}"),
        }
    }
    panic!("the finite original source graph must finish retirement");
}
