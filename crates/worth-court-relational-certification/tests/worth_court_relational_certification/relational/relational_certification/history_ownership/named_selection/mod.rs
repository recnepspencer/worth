//! Persistent names across Native owned-graph selection: restores at the
//! exact original identity survive checkpoint capture, validation and both
//! restore forms; selections fail closed on records this branch never named
//! and refuse any caller naming plan.
use super::*;
use std::collections::BTreeMap;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    CommitNamingPlan, InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    LineagePostureClaim, LineageRecordTarget, PendingBirthTarget, PendingCommitBirth,
    PersistentNameKey, PersistentNameOfRecord, PersistentNameSelection, PersistentNameSubjectKind,
};
use worth_relational::facade::mvcc::{
    RelationalOwnedGraphSelectionPreparationError, RelationalPublicationOutcome,
};
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;
mod checkpoint;
mod epochs;
mod refusals;

const MEMBER_SCHEME: &str = "worth.cert.member.v1";
const REFERENCE_SCHEME: &str = "worth.cert.reference.v1";
/// An unschemed owned entity kind, rematerialized beside named members.
const PART: KindId = KindId(7);

fn scheme(
    id: &str,
    subject: PersistentNameSubjectKind,
    name: &str,
) -> InstalledPersistentNameScheme {
    InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
        id,
        subject,
        name,
        ["authored"],
        false,
        false,
    )
    .unwrap()
}

/// `Member` and `Reference` are schemed; `Part` and ownership edges are not.
pub(super) fn named_registry() -> RelationalSchemaRegistry {
    let base = registry();
    let fixture = AspectSchemaFixture::default();
    let base = base
        .register_entity_kind(EntityKindRegistration {
            kind_id: PART,
            kind_name: "Part".into(),
            schema_id: fixture.schema_id.clone(),
            schema_version_id: fixture.schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    let mut schemes = InstalledPersistentNameSchemeRegistry::new();
    schemes
        .register(scheme(
            MEMBER_SCHEME,
            PersistentNameSubjectKind::Entity,
            "Member",
        ))
        .unwrap();
    schemes
        .register(scheme(
            REFERENCE_SCHEME,
            PersistentNameSubjectKind::Relation,
            "Reference",
        ))
        .unwrap();
    base.install_persistent_name_schemes(
        schemes,
        BTreeMap::from([
            (
                (PersistentNameSubjectKind::Entity, KindId(3)),
                MEMBER_SCHEME.to_owned(),
            ),
            (
                (PersistentNameSubjectKind::Relation, KindId(4)),
                REFERENCE_SCHEME.to_owned(),
            ),
        ]),
    )
    .unwrap()
}

pub(super) fn named_ownership() -> RelationalModelOwnership {
    RelationalModelOwnership::new(vec![RelationalModelOwnershipDomain {
        root: KindId(1),
        relations: vec![KindId(2)],
        entities: vec![KindId(3), PART],
    }])
}

pub(super) fn named_runtime() -> RelationalRuntime {
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(named_registry())
        .build()
        .unwrap();
    runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_custom_invariants(
            RelationalSchemaRegistry::new(),
            vec![named_ownership().invariant_registration().unwrap()],
        )
        .unwrap();
    runtime
}

pub(super) fn head(runtime: &RelationalRuntime) -> AdmittedRelationalBranchBasis {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
}

fn birth(target: PendingBirthTarget, key: &str) -> PendingCommitBirth {
    let scheme_id = match target_kind(&target) {
        PersistentNameSubjectKind::Relation => REFERENCE_SCHEME,
        _ => MEMBER_SCHEME,
    }
    .to_owned();
    PendingCommitBirth {
        target,
        scheme_id,
        occurrence_key: format!("cert.{key}"),
        role_path: "authored".to_owned(),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator: Vec::new(),
    }
}

fn target_kind(target: &PendingBirthTarget) -> PersistentNameSubjectKind {
    match target {
        PendingBirthTarget::CreatedRelation(_) => PersistentNameSubjectKind::Relation,
        PendingBirthTarget::CreatedEntity(_) => PersistentNameSubjectKind::Entity,
    }
}

/// Commits one intent on `basis`, born named when `named`.
pub(super) fn commit_on(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    intent: MutationIntent,
    named: Option<PendingBirthTarget>,
    key: &str,
) -> CommitResult {
    let mut transaction = runtime
        .begin_branch_transaction(basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(intent),
            execution.allocation_policy(),
        )
        .unwrap();
    if let Some(target) = named {
        transaction.attach_naming_plan(CommitNamingPlan {
            pending_births: vec![birth(target, key)],
            ..CommitNamingPlan::default()
        });
    }
    let committed = transaction.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
    committed
}

fn created_entity(committed: &CommitResult) -> EntityId {
    committed
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(id) => Some(*id),
            RecordRef::Relation(_) => None,
        })
        .unwrap()
}

/// A named `Member` (`named`) or an unnamed `Part` in the member partition.
pub(super) fn owned_entity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    key: &str,
    named: bool,
) -> EntityId {
    let kind_id = if named { KindId(3) } else { PART };
    let spec = EntitySpec {
        partition_id: PartitionId(3),
        kind_id,
        client_key: ClientKey::raw(key),
        fields: AspectFieldPatch::default(),
    };
    let target = CreatedEntityRef {
        partition_id: PartitionId(3),
        kind_id,
        client_key: ClientKey::raw(key),
    };
    let intent = MutationIntent::Create(CreateIntent::Entity(spec));
    let named = named.then_some(PendingBirthTarget::CreatedEntity(target));
    created_entity(&commit_on(execution, runtime, basis, intent, named, key))
}

pub(super) fn delete(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entities: &[EntityId],
) {
    let mut transaction = begin(runtime);
    let mut batch = WorkerIntentBatch::new("delete-owned");
    for &entity_id in entities {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id },
        )));
    }
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let committed = transaction.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}

/// Prepares and publishes the model's owned graph at `selected`.
pub(super) fn select(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    model: EntityId,
    selected: &AdmittedRelationalBranchBasis,
) -> Result<CommitResult, RelationalOwnedGraphSelectionPreparationError> {
    let graph = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &head(runtime),
            selected,
            &named_ownership(),
            &named_ownership(),
            model,
        )
        .unwrap();
    let port = runtime.preparation_port();
    let input = port.admit_owned_graph_selection(execution, begin(runtime), graph)?;
    let candidate = port.prepare_owned_graph_selection(execution, input, None)?;
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("a lawful selection publishes");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(runtime, &committed);
    Ok(committed)
}

pub(super) fn name_of(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    entity: EntityId,
) -> PersistentNameOfRecord {
    runtime
        .with_persistent_names_at_observation(execution, &basis.observation(), |view| {
            view.name_of_record(LineageRecordTarget::Entity(entity))
        })
        .unwrap()
}

pub(super) fn resolve(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    name: PersistentNameKey,
) -> String {
    runtime
        .with_persistent_names_at_observation(execution, &basis.observation(), |view| {
            format!(
                "{:?}",
                view.resolve(name, &PersistentNameSelection::Exact).result
            )
        })
        .unwrap()
}
