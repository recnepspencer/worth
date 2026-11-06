//! Native opts into custody through the same public schema registrar as Query.
use super::*;
use worth_relational::facade::schema::register_current_publication_storage;

pub(super) fn runtime() -> RelationalRuntime {
    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        worth_relational::facade::config::CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let registry = register_current_publication_storage(
        fixture.build_registry(),
        &fixture.schema_id,
        fixture.schema_version_id,
    )
    .unwrap();
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap()
}

#[test]
fn owned_publication_admission_requires_registered_storage_before_any_effect() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let mut runtime = AspectSchemaFixture::with_default_declared_aspects(
        worth_relational::facade::config::CascadeDeletePolicy::RetainDanglingForAudit,
    )
    .build_runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let installed = installed.with_source_scope_owned_producers([("owned".to_owned(), KindId(1))]);
    let source = targets(&["authored-source"]);
    let mut tx = begin_main_transaction(&runtime);
    tx.push_batch(creates("author", &source), execution.allocation_policy())
        .unwrap();
    let born = tx.commit(execution, &runtime).unwrap();
    let source = born.created_entity(&source[0]).unwrap();
    release_test_commit_snapshot(&runtime, &born);
    let (_, before) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let meaning = RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap(),
        family: "owned".into(),
        source_scope: source,
        source_partition: [0; 32],
        source_epoch: [1; 32],
    };
    let deny = |error: TransactionCommitError| {
        let TransactionCommitError::Publication { error, .. } = error else {
            panic!("{error:?}")
        };
        assert_eq!(error.authored_retirement_refusal(), Some(worth_relational::facade::publication::AuthoredRetirementRefusal::CurrentOwnershipStorageMissing));
        assert_eq!(error.context.affected_records, [RecordRef::Entity(source)]);
    };
    deny(
        runtime
            .readmit_required_output_route_absence(execution, &installed, &before, &meaning)
            .unwrap_err(),
    );
    deny(
        runtime
            .readmit_required_output_publication(
                execution,
                &installed,
                &before,
                "owned",
                &[1; 32],
                &meaning,
                &[RequiredOutputRoleTarget {
                    role: "output".into(),
                    entity_name: "node".into(),
                    kind: KindId(1),
                    posture: RequiredOutputRolePosture::Create,
                    target: EntityReference::Existing(source),
                }],
            )
            .unwrap_err(),
    );
    let generated = targets(&["generated"]);
    let mut tx = begin_main_transaction(&runtime);
    tx.push_batch(creates("owned", &generated), execution.allocation_policy())
        .unwrap();
    let proposal = tx.validate(execution, &runtime).unwrap();
    let result = proposal.bind_required_output_publication(
        &installed,
        RequiredOutputPublication::Birth {
            admission: owner.admit_required_output("owned", [1; 32]),
            absence: None,
            meaning,
            roles: vec![RequiredOutputRoleTarget {
                role: "output".into(),
                entity_name: "node".into(),
                kind: KindId(1),
                posture: RequiredOutputRolePosture::Create,
                target: EntityReference::Created(generated[0].clone()),
            }],
        },
    );
    deny(match result {
        Err(error) => error,
        Ok(_) => panic!("unregistered owned publication acquired admission"),
    });
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
}
