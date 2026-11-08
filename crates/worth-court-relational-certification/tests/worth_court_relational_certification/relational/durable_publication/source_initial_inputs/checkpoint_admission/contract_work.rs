//! A limited cold read preserves the wide schema and its retryable data owner.
use super::*;
use worth_execution::{
    CancellationSource, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
use worth_foundational::facade::*;
use worth_relational::facade::{
    config::CascadeDeletePolicy, identity::KindId, mvcc::RelationalTransactionReadDenial,
    schema::*, transactions::RelationalExecutionDenialCause,
};
use worth_testkit_relational::{
    aspect_storage::{definition::*, publications::begin_main_transaction, AspectSchemaFixture},
    durable_publication::durable_runtime,
};

#[test]
fn cold_head_work_exhaustion_keeps_a_wide_contract_region_retryable() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&setup);
    let declarations = wide_declarations();
    let (source, medium) = initialized_contracts(request, declarations.clone());
    let created = create_entity_outcome(request, &source, "wide-contract-region");
    let entity = created.semantic_result().created_entities()[0].assigned;
    release_test_commit_snapshot(&source, &created);
    drop(created);
    let image = capture_external(&source, &medium, request);
    medium.file_checkpoint(image.checkpoint().clone());
    let input = home_image_input(&medium, &image, request).unwrap();
    drop((image, source));
    let mut recovered = durable_runtime(&medium);
    recovered
        .prepare_initial_schema_installation()
        .unwrap()
        .install(declarations)
        .unwrap();
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(request, plan)
        .unwrap();
    drop(authority);

    let budget = SerialMemoryBudget::new(None);
    let caller =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let limited = &ExecutionRequest::serial(&caller);
    let mut transaction = begin_main_transaction(&recovered);
    let mut observed = None;
    let stopped = limited.run(ExecutionWorkCeiling::new(Some(16)), |_| {
        observed = Some(
            transaction
                .read_entity(limited, entity)
                .map(|value| value.base().is_some()),
        );
    });
    assert!(
        stopped.is_ok(),
        "the entered computation returns its typed read result"
    );
    assert!(
        matches!(
            observed,
            Some(Err(RelationalTransactionReadDenial::Request(
                RelationalExecutionDenialCause::WorkExhausted
            )))
        ),
        "{observed:?}"
    );
    drop(transaction);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let mut transaction = begin_main_transaction(&recovered);
    assert!(transaction
        .read_entity(request, entity)
        .unwrap()
        .base()
        .is_some());
}

pub(super) fn initialized_contracts(
    request: &ExecutionRequest<'_, '_>,
    declarations: RelationalSchemaRegistry,
) -> (RelationalRuntime, Arc<TestDurableMedium>) {
    let medium = TestDurableMedium::new();
    let mut runtime = durable_runtime(&medium);
    let permission = runtime.begin_fresh_home_initialization(request).unwrap();
    runtime
        .prepare_fresh_home_schema_installation(request, permission)
        .unwrap()
        .install_with_managed_sources(request, declarations, vec![], vec![])
        .unwrap();
    (runtime, medium)
}

fn wide_declarations() -> RelationalSchemaRegistry {
    let mut fields = aspects()
        .struct_fields()
        .optional("field-000", ScalarAspectType::String);
    for index in 1..128 {
        fields = fields.optional(format!("field-{index:03}"), ScalarAspectType::String);
    }
    let contract = aspects()
        .contract()
        .for_key(aspect_key("wide"))
        .identified_by(AspectIdentity(492))
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(fields.finish().unwrap());
    let mut fixture = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    fixture.entity_kind_id = KindId(902);
    fixture.entity_kind_name = "wide.contract.kind".into();
    fixture.entity_aspects.push(DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: field_key("wide"),
        },
        contract,
    });
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: fixture.entity_kind_id,
            kind_name: fixture.entity_kind_name,
            schema_id: fixture.schema_id,
            schema_version_id: fixture.schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::new(
                fixture.entity_aspects,
            ),
        })
        .unwrap()
}
