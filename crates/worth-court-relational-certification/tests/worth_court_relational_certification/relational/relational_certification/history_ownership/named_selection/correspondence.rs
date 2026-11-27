//! A schema-declared correspondence never reconciles a source-named record
//! onto a different target record silently (Phase 7, Arch 5, 10).
//!
//! Both branches create a named record with the same declared key, so the
//! merge plan reconciles the source record onto the target's distinct
//! record. Carrying the source's name there needs the producer that named
//! it, so preparation refuses typed, naming the source record.
use super::*;
use std::sync::Arc;
use worth_foundational::facade::AspectKey;
use worth_relational::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
    IdentityBasisKind, IdentityBasisScope, MergeExecutionPreparationError, MergeExecutionRequest,
    MergeIntent,
};
use worth_testkit_relational::aspect_storage::{aspect_key, definition, field_key, publications};

const SCHEME: &str = "worth.cert.corresponded.v1";
const KIND: KindId = KindId(9);

fn corresponded_registry() -> RelationalSchemaRegistry {
    let aspect = |name| {
        let (binding, contract) =
            definition::entity_field_aspect(aspect_key(name), field_key(name));
        DeclaredAspectContractBinding { binding, contract }
    };
    let declarations = KindAspectContractDeclarations::new(vec![aspect("name"), aspect("status")])
        .with_identity_declarations(vec![IdentityBasisDeclaration {
            scope: IdentityBasisScope::AspectKey(aspect_key("name")),
            basis: IdentityBasisKind::DeclaredKeySet(
                Arc::<[AspectKey]>::from([aspect_key("name")]).into(),
            ),
        }])
        .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
            aspect_key: aspect_key("status"),
            policy: AspectMergePolicyKind::PreferRicher,
        }]);
    let fixture = worth_testkit_relational::aspect_storage::AspectSchemaFixture::default();
    let base = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KIND,
            kind_name: "Corresponded".into(),
            schema_id: fixture.schema_id.clone(),
            schema_version_id: fixture.schema_version_id,
            aspect_contract_declarations: declarations,
        })
        .unwrap();
    let mut schemes = InstalledPersistentNameSchemeRegistry::new();
    schemes
        .register(scheme(
            SCHEME,
            PersistentNameSubjectKind::Entity,
            "Corresponded",
        ))
        .unwrap();
    let binding = ((PersistentNameSubjectKind::Entity, KIND), SCHEME.to_owned());
    base.install_persistent_name_schemes(schemes, std::collections::BTreeMap::from([binding]))
        .unwrap()
}

/// Creates a record with the shared declared key on `basis`, born named.
fn named(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    key: &str,
    status: Option<&str>,
) -> EntityId {
    let mut fields = vec![(aspect_key("name"), field_key("name"), "shared-name")];
    fields.extend(status.map(|status| (aspect_key("status"), field_key("status"), status)));
    let spec = EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KIND,
        client_key: ClientKey::raw(key),
        fields: publications::string_aspect_field_patch(fields),
    };
    let mut transaction = runtime
        .begin_branch_transaction(basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let batch =
        WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(spec)));
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let target = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KIND,
        client_key: ClientKey::raw(key),
    };
    transaction.attach_naming_plan(CommitNamingPlan {
        pending_births: vec![PendingCommitBirth {
            target: PendingBirthTarget::CreatedEntity(target),
            scheme_id: SCHEME.to_owned(),
            occurrence_key: format!("cert.corresponded.{key}"),
            role_path: "authored".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: Vec::new(),
        }],
        ..CommitNamingPlan::default()
    });
    let committed = transaction.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
    created_entity(&committed)
}

#[test]
fn a_corresponded_source_named_record_refuses_instead_of_reconciling_unnamed() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(corresponded_registry())
        .build()
        .unwrap();
    named(execution, &runtime, &head(&runtime), "main-shared", None);
    let (_, source) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    let fork = runtime
        .fork_branch(execution, BranchId("feature".into()), source)
        .unwrap();
    let feature = runtime.observe_branch(fork.target_identity()).unwrap().1;
    let source = named(
        execution,
        &runtime,
        &feature,
        "feature-shared",
        Some("active"),
    );
    let bound = runtime
        .bind_merge_execution_request(MergeExecutionRequest::new(
            BranchId("main".into()),
            BranchId("feature".into()),
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    let refusal = runtime
        .prepare_merge_execution(execution, bound)
        .expect_err("a corresponded named record never reconciles silently");
    let MergeExecutionPreparationError::NamedRecord(branch, record, _) = refusal else {
        panic!("the refusal names the source record: {refusal:?}");
    };
    assert_eq!(branch, BranchId("feature".into()));
    assert_eq!(record, RecordRef::Entity(source));
}
