//! Actual field custody on root/nested Source loans and genuine materialization.
use super::*;
use worth_foundational::facade::{
    AspectValue, CanonicalDigestId, ContentRefId, ContractValidationInput,
    PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
    StructAspectValue,
};
mod lifecycle;
pub(super) fn observe(
    source: &ManagedIndexSourcePreparation<'_>,
    court: &owner::Court,
) -> Result<(), ManagedIndexSourcePreparationDenial> {
    for scope in source.scopes() {
        if scope.scope_exists()? {
            observe_rows(&scope.proposed(), court, true)?;
        }
        match scope.committed_scope_exists() {
            Ok(true) => observe_rows(&scope.committed()?, court, false)?,
            Ok(false) => {}
            Err(cause) if cause.source_unavailable() => {}
            Err(cause) => return Err(cause),
        }
    }
    Ok(())
}
pub(super) fn observe_rows(
    rows: &ManagedIndexSourceRows<'_>,
    court: &owner::Court,
    proposed: bool,
) -> Result<(), ManagedIndexSourcePreparationDenial> {
    let read = rows.acquire_all()?;
    for root in rows.occurrences(&read)? {
        let prepared = rows.prepare_row_from_read(&read, &root)?;
        prepared.with_field_content(0, |value, frame| {
            check(value.value(), frame, court, proposed)
        })?;
        let owners = rows.prepare_projection(&root, 0)?;
        let foreign = rows.prepare_projection(&root, 1)?;
        for owner in rows.projection_occurrences(&owners)? {
            rows.with_projected_field_content(&owners, &owner, 0, |value, frame| {
                check(value.value(), frame, court, proposed)
            })?;
            let entered = std::cell::Cell::new(false);
            assert!(matches!(
                rows.with_projected_field_content(&foreign, &owner, 0, |_, _| entered.set(true)),
                Err(ManagedIndexSourcePreparationDenial::Source(
                    DerivedRelationReadDenial::Definition(
                        DerivedRelationDenial::ForeignObservation
                    )
                ))
            ));
            assert!(
                !entered.get(),
                "a foreign route cannot lend even the same content identity"
            );
            court.content_foreign_checks.fetch_add(1, Ordering::SeqCst);
        }
    }
    Ok(())
}
fn check(
    value: Option<&AspectValue>,
    frame: Option<worth_relational::facade::content::RelationalContentFrame>,
    court: &owner::Court,
    proposed: bool,
) {
    let frame = frame.expect("the actual declared content field is present");
    assert_eq!(value, Some(&AspectValue::ContentRef(frame.identity())));
    // This frozen frame is a separate literal numerical/digest oracle.
    assert_eq!(frame.bytes(), crate::content::expected::FRAME);
    court.content_checks.fetch_add(1, Ordering::SeqCst);
    if proposed {
        court.content_proposed_checks.fetch_add(1, Ordering::SeqCst);
    }
}
fn fixture() -> (
    RelationalRuntime,
    ManagedIndexSourceProgram,
    Arc<owner::Court>,
    worth_relational::facade::mvcc::PublicationCompanionRegistration,
) {
    let (registry, program) = registry::profile(registry::Profile::Content);
    let court = Arc::new(owner::Court::default());
    court.probe_content.store(true, Ordering::SeqCst);
    let rule = CustomInvariantRegistration::new(owner::Reader {
        maximum_work: None,
        program: program.clone(),
        court: court.clone(),
    })
    .unwrap()
    .with_managed_index_sources(vec![program.clone()])
    .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .custom_invariant(rule)
        .build()
        .unwrap();
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court.clone())))
        .unwrap();
    (runtime, program, court, registration)
}
fn birth(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) -> (EntityId, EntityId, RelationId, ContentRefId) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut txn = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    let id = txn
        .admit_content(
            execution,
            CanonicalDigestId::new([11; 32]),
            &crate::content::fixture::grid(),
        )
        .unwrap();
    let mut batch = WorkerIntentBatch::new("content-source");
    let contract = content_contract();
    for (kind, name) in [(1, "A"), (4, "x")] {
        let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&contract),
            value: ContractValidationInput::Struct(
                StructAspectValue::new([(registry::field("value"), AspectValue::ContentRef(id))])
                    .unwrap(),
            ),
        }]);
        batch = batch.push(MutationIntent::Create(CreateIntent::EntityAspects(
            EntityAspectCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(kind),
                client_key: ClientKey::raw(name),
                aspect_patch: patch,
            },
        )));
    }
    batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw("A-x"),
            source: EntityReference::Created(created(1, "A")),
            target: EntityReference::Created(created(4, "x")),
            fields: Default::default(),
        },
    )));
    txn.push_batch(batch, execution.allocation_policy())
        .unwrap();
    let born = txn.commit(execution, runtime).unwrap();
    let a = born.created_entity(&created(1, "A")).unwrap();
    let x = born.created_entity(&created(4, "x")).unwrap();
    let relation = born
        .changed_records
        .iter()
        .find_map(|r| match r {
            RecordRef::Relation(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    (a, x, relation, id)
}
fn content_contract() -> worth_foundational::facade::AspectContract {
    registry::content_declaration().contract
}
#[test]
fn content_native_source_root_and_nested_endpoint_keep_actual_route_authority() {
    let (runtime, _program, court, _registration) = fixture();
    let cancel = CancellationSource::new();
    let host = crate::content::original(&cancel);
    let execution = ExecutionRequest::serial(&host);
    birth(&runtime, &execution);
    assert!(court.content_checks.load(Ordering::SeqCst) >= 4);
    assert!(court.content_foreign_checks.load(Ordering::SeqCst) >= 2);
}
