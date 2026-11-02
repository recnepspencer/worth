use std::{path::Path, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::{
    identity::*, mvcc::*, runtime::*, schema::*, symbols::ClientKey, transactions::*,
};
use worth_store_relational::{
    RelationalStoreHome, RelationalStoreHomeOpenRefusal, RelationalStoreHomeRequest,
};
use worth_testkit_relational::content_grid::{ExactSampleGrid, ExactSampleGridCodec};

pub(super) const FORMAT: CanonicalDigestId = CanonicalDigestId::new([11; 32]);
const FIELD: CanonicalDigestId = CanonicalDigestId::new([12; 32]);

pub(super) fn open(
    path: &Path,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<RelationalStoreHome, RelationalStoreHomeOpenRefusal> {
    RelationalStoreHome::open(request(path, 1), execution)
}

pub(super) fn request(path: &Path, version: u32) -> RelationalStoreHomeRequest {
    let contract = contract();
    let registry = worth_testkit_relational::aspect_storage::AspectSchemaFixture {
        entity_aspects: vec![DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: FieldKey::new("samples").unwrap(),
            },
            contract,
        }],
        ..Default::default()
    }
    .build_registry()
    .register_content_format(
        FORMAT,
        Arc::new(ExactSampleGridCodec {
            descriptor: ContentFormat::new("sample-grid", version).unwrap(),
            alter: false,
        }),
    )
    .unwrap()
    .bind_content_field(FIELD, FORMAT)
    .unwrap();
    RelationalStoreHomeRequest::new(
        path,
        RelationalRuntimeApi::builder().schema_registry(registry),
    )
}

fn contract() -> AspectContract {
    AspectContract::struct_aspect(
        AspectKey::new("samples").unwrap(),
        AspectIdentity(741),
        AspectContractRevision(1),
        StructAspectShape::new([FieldDeclaration::new(
            FieldKey::new("values").unwrap(),
            ScalarAspectType::ContentRef,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()
        .with_codec(FIELD)])
        .unwrap(),
    )
}

pub(super) fn publish(
    home: &RelationalStoreHome,
    execution: &ExecutionRequest<'_, '_>,
    first: i64,
    entity: Option<EntityId>,
) -> (ContentRefId, EntityId) {
    publish_grid(
        home,
        execution,
        ExactSampleGrid {
            rows: 2,
            columns: 2,
            samples: vec![first, -1, 0, i64::MAX],
        },
        entity,
    )
}

pub(super) fn publish_grid(
    home: &RelationalStoreHome,
    execution: &ExecutionRequest<'_, '_>,
    grid: ExactSampleGrid,
    entity: Option<EntityId>,
) -> (ContentRefId, EntityId) {
    let (identity, committed) = try_publish_grid(home, execution, grid, entity);
    let committed = committed.unwrap();
    let entity = committed
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    (identity, entity)
}

pub(super) fn try_publish_grid(
    home: &RelationalStoreHome,
    execution: &ExecutionRequest<'_, '_>,
    grid: ExactSampleGrid,
    entity: Option<EntityId>,
) -> (ContentRefId, Result<CommitResult, TransactionCommitError>) {
    let runtime = home.runtime();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let identity = transaction.admit_content(execution, FORMAT, &grid).unwrap();
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract()),
        value: ContractValidationInput::Struct(
            StructAspectValue::new([(
                FieldKey::new("values").unwrap(),
                AspectValue::ContentRef(identity),
            )])
            .unwrap(),
        ),
    }]);
    let intent = match entity {
        Some(entity_id) => MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
            ApplyEntityAspectPatchIntent {
                entity_id,
                aspect_patch: patch,
            },
        )),
        None => MutationIntent::Create(CreateIntent::EntityAspects(EntityAspectCreateIntent {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw("grid"),
            aspect_patch: patch,
        })),
    };
    transaction
        .push_batch(
            WorkerIntentBatch::new("content-grid").push(intent),
            execution.allocation_policy(),
        )
        .unwrap();
    (identity, transaction.commit(execution, runtime))
}

pub(super) fn assert_value(
    home: &RelationalStoreHome,
    execution: &ExecutionRequest<'_, '_>,
    entity: EntityId,
    id: ContentRefId,
    first: i64,
) {
    let (_, basis) = home
        .runtime()
        .observe_branch(&home.runtime().main_branch_identity())
        .unwrap();
    let read = home
        .runtime()
        .read_truth()
        .read_observation(execution, &basis.observation())
        .unwrap()
        .unwrap();
    let record = read.get_entity(entity).unwrap();
    let state = record.authoritative_aspect_state.as_ref().unwrap();
    let ContractValidatedAspectValueView::Struct(fields) = state
        .get(&AspectKey::new("samples").unwrap())
        .unwrap()
        .view()
    else {
        panic!("the selected content aspect must be a struct");
    };
    assert_eq!(
        fields.get(&FieldKey::new("values").unwrap()),
        Some(&AspectValue::ContentRef(id))
    );
    let frame = basis.read_content(execution, id, FORMAT).unwrap();
    // Independent numeric oracle: literal format/header and integer positions;
    // no codec encoder, verifier or Native header parser supplies expectations.
    assert_eq!(&frame.bytes()[..12], b"WRTHCONT\x01\0\x0b\0");
    assert_eq!(&frame.bytes()[12..23], b"sample-grid");
    assert_eq!(
        &frame.bytes()[23..35],
        &[1, 0, 0, 0, 40, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(frame.bytes().len(), 75);
    assert_eq!(&frame.bytes()[35..43], &[2, 0, 0, 0, 2, 0, 0, 0]);
    for (word, expected) in
        frame.bytes()[43..]
            .as_chunks::<8>()
            .0
            .iter()
            .zip([first, -1, 0, i64::MAX])
    {
        assert_eq!(i64::from_le_bytes(*word), expected);
    }
}
