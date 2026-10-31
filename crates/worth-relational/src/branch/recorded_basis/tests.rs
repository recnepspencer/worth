use super::*;
use crate::facade::branch::RecordedRelationalComponentTarget as Target;
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

#[test]
fn recorded_current_component_distinguishes_empty_from_actual_commit() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let port = runtime.owner_component_services().basis_port();
    let (_, empty) = port
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let recorded = port
        .export_recorded_current_component(execution, &empty)
        .unwrap();
    assert!(matches!(recorded.target(), Target::Empty));
    assert!(matches!(
        recorded.descriptor().reference().target(),
        worth_foundational::FoundationalBranchTarget::Empty
    ));
    assert_eq!(
        recorded.descriptor().schema_commitment(),
        empty.descriptor().schema_commitment()
    );
    assert_eq!(
        recorded.descriptor().visibility_commitment(),
        empty.descriptor().visibility_commitment()
    );
    assert_ne!(
        recorded.descriptor().runtime_instance_id(),
        empty.descriptor().runtime_instance_id()
    );
    let wire = rmp_serde::to_vec_named(&recorded).unwrap();
    assert_eq!(
        rmp_serde::from_slice::<RecordedRelationalComponentReference>(&wire).unwrap(),
        recorded
    );

    let committed = create_entity_outcome(execution, &runtime, "recorded-current");
    let (_, current) = port
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let recorded = port
        .export_recorded_current_component(execution, &current)
        .unwrap();
    let Target::Canonical(actual) = recorded.target() else {
        panic!("actual commit must be canonical")
    };
    let original = attached_test_medium(&runtime).offered().pop().unwrap();
    assert_eq!(actual.receipt(), &committed.commit);
    assert_eq!(actual.position(), committed.patch_position());
    assert_eq!(
        actual.artifact_digest(),
        original.payload_digest().as_bytes()
    );
    let worth_foundational::FoundationalBranchTarget::Basis(target) = current.reference().target()
    else {
        panic!("owner basis selects the same actual commit")
    };
    assert_eq!(actual.roots(), target.roots());
    assert_eq!(
        recorded.descriptor().reference().generation(),
        current.reference().generation()
    );
    assert_eq!(
        recorded.descriptor().truth_version(),
        current.truth_version()
    );
    let wire = rmp_serde::to_vec_named(&recorded).unwrap();
    assert_eq!(
        rmp_serde::from_slice::<RecordedRelationalComponentReference>(&wire).unwrap(),
        recorded
    );
    assert!(
        matches!(
            port.readmit_branch_basis(recorded.descriptor()),
            Err(crate::branch::RelationalBranchBasisDenial::ForeignRuntime { .. })
        ),
        "canonical descriptions are not rewritten into operational bases"
    );
    release_test_commit_snapshot(&runtime, &committed);
}

#[test]
fn recorded_current_component_refuses_foreign_and_stale_admitted_bases() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let foreign = runtime_with_test_schema();
    let port = runtime.owner_component_services().basis_port();
    let (_, before) = port
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(port
        .export_recorded_current_component(execution, &before)
        .is_ok());
    assert!(matches!(
        foreign
            .owner_component_services()
            .basis_port()
            .export_recorded_current_component(execution, &before),
        Err(RelationalRecordedComponentRefusal::Basis(
            crate::branch::RelationalBranchBasisDenial::ForeignRuntime { .. }
        ))
    ));
    create_entity(execution, &runtime, "advances-recorded-basis");
    assert!(matches!(
        port.export_recorded_current_component(execution, &before),
        Err(RelationalRecordedComponentRefusal::Basis(_))
    ));
    let (_, after) = port
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(port
        .export_recorded_current_component(execution, &after)
        .is_ok());
}
