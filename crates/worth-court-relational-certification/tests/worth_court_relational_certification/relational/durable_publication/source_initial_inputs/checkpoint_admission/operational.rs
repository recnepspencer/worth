//! Operational Native image admission keeps the original-frame reader separate.
use super::*;
use worth_relational::facade::mvcc::RelationalCallerAssociation;
use worth_testkit_relational::durable_publication::create_entity;

#[test]
fn checkpoint_image_restores_publication_bases_and_creations_without_canonical_body_reads() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let mut source = runtime_with_creation_aspect(&medium);
    let initialization = source.begin_fresh_home_initialization(request).unwrap();
    source
        .prepare_fresh_home_schema_installation(request, initialization)
        .unwrap()
        .install_with_managed_sources(request, additions(), vec![], vec![])
        .unwrap();
    create_entity(request, &source, "image-predecessor");
    let before = current(request, &source);
    let (_, before_basis) = source
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let before_lease = source.retain_component_basis(&before_basis).unwrap();
    let association = RelationalCallerAssociation::from_bytes(b"image-original-effect").unwrap();
    let performed = perform_created_struct(request, &source, &association);
    let receipt = performed.canonical_commit().commit.clone();
    let after = current(request, &source);
    let after_lease = source
        .retain_component_basis(performed.next_basis())
        .unwrap();
    let result = source.settle_performed_publication(performed).unwrap();
    let entity = result.semantic_result().created_entities()[0].assigned;
    release_test_commit_snapshot(&source, &result);
    create_entity(request, &source, "image-later-current");
    let current_source = current(request, &source);
    let capture = capture_external(&source, &medium, request);
    medium.file_checkpoint(capture.checkpoint().clone());
    let reads = medium.canonical_member_reads();
    let input = home_image_input(&medium, &capture, request).unwrap();
    source.release_component_basis(before_lease).unwrap();
    source.release_component_basis(after_lease).unwrap();
    drop((capture, before_basis, source));
    let mut receiver = runtime_with_creation_aspect(&medium);
    receiver
        .prepare_initial_schema_installation()
        .unwrap()
        .install(additions())
        .unwrap();
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = receiver
        .durability_recovery()
        .recover_with_authority(request, plan)
        .unwrap();
    assert_eq!(current(request, &receiver), current_source);
    let subject = receiver
        .durability_authority()
        .discover_associated_effects(request, &association)
        .unwrap()
        .pop()
        .unwrap();
    let effect = receiver
        .durability_authority()
        .readmit_associated_effect(request, &subject)
        .unwrap();
    assert_eq!(effect.semantic_result().receipt(), &receipt);
    assert_eq!(
        effect
            .created_entity(request, entity)
            .unwrap()
            .unwrap()
            .entity_id(),
        entity
    );
    assert_eq!(
        effect
            .created_entity(request, entity)
            .unwrap()
            .unwrap()
            .field(
                &worth_foundational::facade::AspectKey::new("summary").unwrap(),
                &worth_foundational::facade::FieldKey::new("title").unwrap(),
            ),
        Some(&worth_foundational::facade::AspectValue::String(
            "image-created".into()
        ))
    );
    let recovered_after = authority
        .recorded_publication_successor(request, &receiver, &effect)
        .unwrap();
    assert_eq!(recovered_after, after);
    let custody = authority
        .admit_recorded_publication(request, &receiver, effect, &before, &after)
        .unwrap();
    assert!(receiver
        .owner_component_services()
        .basis_port()
        .compare_current_exact(custody.custody().successor())
        .is_err());
    let fork = receiver.fork().unwrap();
    let fork_recorded = current(request, &fork);
    assert_eq!(fork_recorded.target(), current_source.target());
    let (_, fork_basis) = fork.observe_branch(&fork.main_branch_identity()).unwrap();
    assert!(receiver
        .owner_component_services()
        .basis_port()
        .compare_current_exact(&fork_basis)
        .is_err());
    let fork_subject = fork
        .durability_authority()
        .discover_associated_effects(request, &association)
        .unwrap()
        .pop()
        .unwrap();
    let fork_effect = fork
        .durability_authority()
        .readmit_associated_effect(request, &fork_subject)
        .unwrap();
    assert_eq!(fork_effect.semantic_result().receipt(), &receipt);
    drop((fork_effect, fork_subject, fork_basis, fork));
    assert_eq!(
        medium.canonical_member_reads(),
        reads,
        "covered image admission opens no canonical bodies"
    );
    let next = capture_external(&receiver, &medium, request);
    medium.file_checkpoint(next.checkpoint().clone());
    request
        .run_native_with_checkpoint(|context| {
            next.checkpoint().acknowledge_selected(request, context)
        })
        .unwrap()
        .0
        .unwrap();
    let port = receiver.owner_component_services().basis_port();
    assert_eq!(
        port.export_recorded_retained_component(request, custody.custody().predecessor())
            .unwrap(),
        before
    );
    assert_eq!(
        port.export_recorded_retained_component(request, custody.custody().successor())
            .unwrap(),
        after
    );
    assert!(authority
        .admit_recorded_component(request, &receiver, &before)
        .is_ok());
    let subject = receiver
        .durability_authority()
        .discover_associated_effects(request, &association)
        .unwrap()
        .pop()
        .unwrap();
    let effect = receiver
        .durability_authority()
        .readmit_associated_effect(request, &subject)
        .unwrap();
    assert_eq!(effect.semantic_result().receipt(), &receipt);
    create_entity(request, &receiver, "image-after-replacement");
    let before_export = medium.canonical_member_reads();
    let original = receiver
        .durability()
        .canonical_commit_artifacts(request)
        .next()
        .unwrap()
        .unwrap();
    assert!(original.as_bytes().starts_with(b"WRTHRCCA"));
    assert!(
        medium.canonical_member_reads() > before_export,
        "original export still demands the selected frame"
    );
}

fn runtime_with_creation_aspect(medium: &Arc<TestDurableMedium>) -> RelationalRuntime {
    use worth_relational::facade::{config::*, runtime::RelationalRuntimeApi, schema::*};
    use worth_testkit_relational::aspect_storage::{definition::*, AspectSchemaFixture};
    let mut schema = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let (binding, contract) =
        entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    schema
        .entity_aspects
        .push(DeclaredAspectContractBinding { binding, contract });
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema.build_registry())
        .build_durable(medium.attachment())
        .unwrap()
}

fn perform_created_struct(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    association: &RelationalCallerAssociation,
) -> worth_relational::facade::mvcc::PerformedRelationalCommit {
    use worth_foundational::facade::*;
    use worth_relational::facade::{
        identity::{KindId, PartitionId},
        mvcc::RelationalPublicationOutcome,
        transactions::*,
    };
    use worth_testkit_relational::aspect_storage::{
        definition::*, publications::begin_main_transaction,
    };
    let (_, contract) = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
        value: ContractValidationInput::Struct(
            StructAspectValue::new([(
                field_key("title"),
                AspectValue::String("image-created".into()),
            )])
            .unwrap(),
        ),
    }]);
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("whole-creation").push(MutationIntent::Create(
                CreateIntent::EntityAspects(EntityAspectCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: "image-created".into(),
                    aspect_patch: patch,
                }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction
        .validate(request, runtime)
        .unwrap()
        .bind_caller_association(association.clone())
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(request, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(request, candidate)
    else {
        panic!("actual struct creation performs")
    };
    performed
}
