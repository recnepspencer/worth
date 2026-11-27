//! End-to-end R2-B2 durable naming schema authority reopen courts.
//!
//! Proves that installed persistent naming schemes and kind bindings are bound
//! into Relational's `SchemaAuthoritySnapshot` and `DurableBranchRootSchemaImage`:
//! - Reopening a checkpoint with naming schemes omitted refuses typed with
//!   `RecoveryFailureClass::SchemaMismatch` and zero effects on the target runtime.
//! - Reopening a checkpoint with one role changed refuses typed with
//!   `RecoveryFailureClass::SchemaMismatch` and zero effects on the target runtime.

use std::collections::BTreeMap;

use worth_execution::ExecutionRequest;
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority, ExecutionPendingEntityBirth,
    ExecutionPendingRelationBirth,
};
use worth_relational::facade::config::{
    CascadeDeletePolicy, CrossContextPolicy, RelationalRuntimeProfile,
};
use worth_relational::facade::durability::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalCheckpointArtifactPolicy,
    RelationalDurableCheckpointSource, RelationalDurableRecoverySource, RelationalRecoveryStop,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry, PersistentNameSubjectKind,
};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use worth_relational::facade::transactions::EntityReference;

use crate::naming_cold_source_fixture::ExternalColdSource;
use crate::naming_execution_fixture::{
    begin_staged_tx, cad_schema_registry, create_face_intent, create_seam_intent,
    execution_lease_for_workers, face_ref, fresh_cad_runtime, lowered_cad_catalog,
    release_commit_snapshot, seam_ref, CadNamingSchema, Face, Seam, SeamSplitDisc,
    DUMMY_ENTITY_KIND_ID, DUMMY_RELATION_KIND_ID, EDGE_KIND_ID, EDGE_SCHEME_ID, FACE_KIND_ID,
    FACE_SCHEME_ID, SEAM_KIND_ID, SEAM_SCHEME_ID,
};

fn base_cad_registry_without_naming_schemes() -> RelationalSchemaRegistry {
    let schema_id = SchemaId("worth.tests.execution.naming.CadNaming".to_owned());
    let entity = |kind_id, kind_name: &str| EntityKindRegistration {
        kind_id,
        kind_name: kind_name.to_owned(),
        schema_id: schema_id.clone(),
        schema_version_id: SchemaVersionId(1),
        aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
    };
    let relation = |kind_id, kind_name: &str| RelationKindRegistration {
        kind_id,
        kind_name: kind_name.to_owned(),
        schema_id: schema_id.clone(),
        schema_version_id: SchemaVersionId(1),
        cross_context_policy: CrossContextPolicy::AllowExplicit,
        cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
        aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        relation_integrity: RelationIntegrityDeclarations::default(),
    };
    RelationalSchemaRegistry::new()
        .register_entity_kind(entity(FACE_KIND_ID, "Face"))
        .and_then(|reg| reg.register_entity_kind(entity(EDGE_KIND_ID, "Edge")))
        .and_then(|reg| reg.register_entity_kind(entity(DUMMY_ENTITY_KIND_ID, "DummyEntity")))
        .and_then(|reg| reg.register_relation_kind(relation(SEAM_KIND_ID, "Seam")))
        .and_then(|reg| reg.register_relation_kind(relation(DUMMY_RELATION_KIND_ID, "DummyRel")))
        .expect("base CAD registry without naming schemes builds")
}

fn cad_registry_with_changed_face_role() -> RelationalSchemaRegistry {
    let mut schemes = InstalledPersistentNameSchemeRegistry::new();
    schemes
        .register(
            InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
                FACE_SCHEME_ID,
                PersistentNameSubjectKind::Entity,
                "Face",
                ["authored", "renamed_cut_piece", "seam_patch"],
                true,
                true,
                false,
            )
            .expect("valid altered face scheme"),
        )
        .unwrap();
    schemes
        .register(
            InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
                EDGE_SCHEME_ID,
                PersistentNameSubjectKind::Entity,
                "Edge",
                ["authored", "cut_edge"],
                false,
                true,
                false,
            )
            .expect("valid edge scheme"),
        )
        .unwrap();
    schemes
        .register(
            InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
                SEAM_SCHEME_ID,
                PersistentNameSubjectKind::Relation,
                "Seam",
                ["seam", "seam_piece"],
                true,
                true,
                false,
            )
            .expect("valid seam scheme"),
        )
        .unwrap();
    base_cad_registry_without_naming_schemes()
        .install_persistent_name_schemes(
            schemes,
            BTreeMap::from([
                (
                    (PersistentNameSubjectKind::Entity, FACE_KIND_ID),
                    FACE_SCHEME_ID.to_owned(),
                ),
                (
                    (PersistentNameSubjectKind::Entity, EDGE_KIND_ID),
                    EDGE_SCHEME_ID.to_owned(),
                ),
                (
                    (PersistentNameSubjectKind::Relation, SEAM_KIND_ID),
                    SEAM_SCHEME_ID.to_owned(),
                ),
            ]),
        )
        .expect("altered CAD registry builds")
}

fn runtime_with(registry: RelationalSchemaRegistry) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .build()
        .expect("runtime builds")
}

#[test]
fn reopen_refuses_schema_mismatch_with_zero_effects_when_naming_schemes_omitted_or_role_changed() {
    let original_reg = cad_schema_registry();
    let omitted_reg = base_cad_registry_without_naming_schemes();
    let changed_role_reg = cad_registry_with_changed_face_role();

    assert_ne!(
        original_reg.authority_digest_bytes(),
        omitted_reg.authority_digest_bytes(),
        "omitting naming schemes must change SchemaAuthoritySnapshot digest"
    );
    assert_ne!(
        original_reg.authority_digest_bytes(),
        changed_role_reg.authority_digest_bytes(),
        "changing a role in a naming scheme must change SchemaAuthoritySnapshot digest"
    );

    let catalog = lowered_cad_catalog();
    let source = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let main = BranchId("main".to_owned());

    let f1_ref = face_ref("f1");
    let f2_ref = face_ref("f2");
    let s1_ref = seam_ref(
        "s1",
        EntityReference::Created(f1_ref.clone()),
        EntityReference::Created(f2_ref.clone()),
    );
    let tx = begin_staged_tx(
        &source,
        "seed-schema-auth",
        vec![
            create_face_intent("f1"),
            create_face_intent("f2"),
            create_seam_intent(
                "s1",
                EntityReference::Created(f1_ref.clone()),
                EntityReference::Created(f2_ref.clone()),
            ),
        ],
    );
    let mut b = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    b.add_entity_birth(ExecutionPendingEntityBirth::new(
        f1_ref.clone(),
        FACE_SCHEME_ID,
        "occ.sa.f1",
        "authored",
    ))
    .unwrap()
    .add_entity_birth(ExecutionPendingEntityBirth::new(
        f2_ref.clone(),
        FACE_SCHEME_ID,
        "occ.sa.f2",
        "authored",
    ))
    .unwrap()
    .add_relation_birth(
        ExecutionPendingRelationBirth::new(s1_ref.clone(), SEAM_SCHEME_ID, "occ.sa.s1", "seam")
            .with_discriminator(&SeamSplitDisc::Ordinal { index: 1 }),
    )
    .unwrap();
    let (res, names) =
        ExecutionNamingCommitAuthority::commit_with_naming_plan(&source, &execution, tx, b.build())
            .expect("seed commit succeeds")
            .into_parts();
    let _ = names
        .created_entity_name::<CadNamingSchema, Face>(&res, &f1_ref)
        .unwrap();
    let (s1_id, _) = names.created_relation_name::<Seam>(&res, &s1_ref).unwrap();
    release_commit_snapshot(&source, &res);

    source
        .durability_authority()
        .checkpoint(&execution)
        .expect("checkpoint succeeds");

    // Case (a): Reopen with persistent naming schemes omitted -> typed SchemaMismatch, zero effects.
    let plan_omitted = source
        .durability()
        .recovery_plan(
            &execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .expect("plan builds");
    let mut target_omitted = runtime_with(omitted_reg);
    let head_before_omitted = target_omitted
        .admit_branch_basis(&target_omitted.main_branch_identity())
        .unwrap()
        .observation()
        .commit_id();
    let err_omitted = target_omitted
        .durability_recovery()
        .recover(&execution, plan_omitted)
        .expect_err("reopen with schemes omitted must refuse");
    assert!(
        matches!(
            err_omitted,
            RelationalRecoveryStop::Durability(ref err) if err.class == RecoveryFailureClass::SchemaMismatch
        ),
        "expected SchemaMismatch when schemes omitted, got: {err_omitted:?}"
    );
    assert_eq!(
        target_omitted
            .admit_branch_basis(&target_omitted.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id(),
        head_before_omitted
    );
    assert!(target_omitted
        .branch_live_persistent_names(&main)
        .unwrap()
        .is_empty());
    assert_eq!(target_omitted.relation_lineage(s1_id), None);

    // Case (b): Reopen with one role changed -> typed SchemaMismatch, zero effects.
    let plan_changed = source
        .durability()
        .recovery_plan(
            &execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .expect("plan builds");
    let mut target_changed = runtime_with(changed_role_reg);
    let head_before_changed = target_changed
        .admit_branch_basis(&target_changed.main_branch_identity())
        .unwrap()
        .observation()
        .commit_id();
    let err_changed = target_changed
        .durability_recovery()
        .recover(&execution, plan_changed)
        .expect_err("reopen with altered scheme role must refuse");
    assert!(
        matches!(
            err_changed,
            RelationalRecoveryStop::Durability(ref err) if err.class == RecoveryFailureClass::SchemaMismatch
        ),
        "expected SchemaMismatch when role changed, got: {err_changed:?}"
    );
    assert_eq!(
        target_changed
            .admit_branch_basis(&target_changed.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id(),
        head_before_changed
    );
    assert!(target_changed
        .branch_live_persistent_names(&main)
        .unwrap()
        .is_empty());
    assert_eq!(target_changed.relation_lineage(s1_id), None);

    // Case (c): External-member cold-directory reopen also refuses both omitted and altered roles.
    let native = source
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .expect("external-member checkpoint captures");
    let covered_pos = native.covered_position().expect("covered position");
    let covered_trans = native
        .covered_branch_transitions()
        .expect("covered transitions");
    for (label, registry) in [
        ("cold-omitted", base_cad_registry_without_naming_schemes()),
        ("cold-role-changed", cad_registry_with_changed_face_role()),
    ] {
        let cold_source = ExternalColdSource::from_native_capture(&native, &source, &execution);
        let mut target = runtime_with(registry);
        let head_before = target
            .admit_branch_basis(&target.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id();
        let recovery = RelationalDurableRecoverySource::new(
            Some(RelationalDurableCheckpointSource::new(
                covered_pos,
                covered_trans,
                native.clone(),
            )),
            Vec::new(),
            Vec::new(),
            cold_source,
        );
        let plan = target
            .durability()
            .recovery_plan_from_source(
                &execution,
                recovery,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .expect("cold plan builds");
        let err = target
            .durability_recovery()
            .recover(&execution, plan)
            .expect_err("cold reopen with mismatched naming layout must refuse");
        assert!(
            matches!(
                err,
                RelationalRecoveryStop::Durability(ref d) if d.class == RecoveryFailureClass::SchemaMismatch
            ),
            "{label}: expected SchemaMismatch, got {err:?}"
        );
        assert_eq!(
            target
                .admit_branch_basis(&target.main_branch_identity())
                .unwrap()
                .observation()
                .commit_id(),
            head_before
        );
        assert!(target
            .branch_live_persistent_names(&main)
            .unwrap()
            .is_empty());
        assert_eq!(target.relation_lineage(s1_id), None);
    }
}
