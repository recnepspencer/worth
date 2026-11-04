//! Actual Empty producers, original DATA lifetime, and concrete roster admission.

use super::*;
use crate::schema::data::RelationalSchemaRegistry;
use crate::tests::support::runtime_with_test_schema;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

fn controller_memory() -> u64 {
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut observed = None;
    request
        .run_native_with_checkpoint(|_| observed = Some(budget.charged_memory_bytes()))
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    observed.unwrap()
}

#[test]
fn original_empty_schemas_survive_archived_installation_and_stream_replacement() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let mut runtime = runtime_with_test_schema();
    let birth = runtime
        .history
        .capture_original_branch_schema_inputs(&request)
        .unwrap();
    let original = Arc::clone(&birth.birth.as_ref().unwrap().schema);
    let expected = original.registry().authority_snapshot();
    assert!(!original.registry().entity_kinds.is_empty());
    let identity = runtime.main_branch_identity();
    runtime.archive_branch(&request, &identity).unwrap();
    runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();

    let routes = runtime.history.canonical_checkpoint_gate();
    let head = routes.branch_transition_head();
    assert_eq!(head.0, 1);
    runtime.history.install_commit_catalog(Default::default());
    routes.compact_branch_transitions_through(head);
    routes.restore_branch_transition_coverage(head);
    assert!(routes.retained_branch_transitions().1.is_empty());
    let continuing = routes.continuing_branch_transitions();
    let detached = routes.detached_branch_transitions();
    drop(routes);
    drop(runtime);
    drop(birth);
    for source in [continuing, detached] {
        let capture = source
            .capture_original_branch_schema_inputs(&request)
            .unwrap();
        assert!(Arc::ptr_eq(
            &capture.birth.as_ref().unwrap().schema,
            &original
        ));
        let installation = capture.installation.as_ref().unwrap();
        assert!(Arc::ptr_eq(&installation.before.schema, &original));
        assert!(!Arc::ptr_eq(&installation.after.schema, &original));
        assert_eq!(installation.before.generation.get(), 1);
        assert_eq!(installation.after.generation.get(), 2);
        assert_eq!(
            installation.before.lifecycle,
            RelationalBranchLifecyclePosture::Archived
        );
        assert_eq!(
            installation.after.lifecycle,
            RelationalBranchLifecyclePosture::Archived
        );
        assert_eq!(
            installation.before.truth_version,
            installation.after.truth_version
        );
        assert_eq!(installation.follows_transition, head);
        assert_eq!(
            installation.after.schema.registry().authority_snapshot(),
            expected
        );
        assert_eq!(capture.transitions().len(), 1);
        assert!(Arc::ptr_eq(&capture.transitions()[0].schema, &original));
        assert_eq!(capture.transitions()[0].positioned.sequence(), head);
        assert_eq!(capture.transitions()[0].positioned.follows().0, 0);
        let original_transition = &capture.transitions()[0];
        source.restore_branch_transition_coverage(RelationalBranchTransitionSequence(0));
        assert!(source.retained_branch_transitions().1.is_empty());
        assert_eq!(
            source
                .branch_transitions
                .prepare_append(&request, &original_transition.positioned)
                .unwrap(),
            Some(0)
        );
        let different = PositionedBranchTransition::admit(
            head,
            original_transition.positioned.follows(),
            crate::branch::RelationalBranchTransition::Archive {
                branch: crate::history::data::BranchId("different-original-prefix".to_owned()),
                prior_generation: FoundationalBranchReferenceGeneration::initial(),
                prior_truth_version: crate::branch::RelationalBranchVersion::initial(),
            },
        );
        assert!(matches!(
            source
                .branch_transitions
                .prepare_append(&request, &different),
            Err(OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch)
        ));
        assert_eq!(source.branch_transitions.state().log.len(), 1);
        assert_eq!(source.branch_transition_head().0, 0);
    }
}

#[test]
fn fresh_recovery_receiver_discards_only_its_unobserved_construction_seed() {
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let birth = runtime
        .history
        .capture_original_branch_schema_inputs(&request)
        .unwrap();
    let original = &birth.birth.as_ref().unwrap().schema;
    runtime
        .history
        .discard_unobserved_construction_schema_inputs_for_recovery();
    let missing = runtime
        .history
        .capture_original_branch_schema_inputs(&request)
        .unwrap();
    assert!(missing.birth.is_none());
    assert!(missing.installation.is_none());
    assert!(missing.transitions().is_empty());
    // This explicit discard precedes foreign source installation. The actual
    // receiver's original root still owns its construction schema.
    let root = runtime
        .history
        .branch_cell(runtime.main_branch_identity().branch_id())
        .unwrap()
        .root()
        .unwrap();
    assert!(Arc::ptr_eq(&root.retained_schema_authority(), original));
    assert!(runtime
        .observe_branch(&runtime.main_branch_identity())
        .is_ok());
}

#[test]
fn original_schema_capture_retains_exact_roster_and_refuses_before_copy() {
    let ordinary = crate::test_execution_policy::serial_request();
    let ordinary_request = ExecutionRequest::serial(&ordinary);
    let mut runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    runtime
        .archive_branch(&ordinary_request, &identity)
        .unwrap();
    let bill = u64::try_from(
        std::alloc::Layout::array::<RetainedBranchTransition>(1)
            .unwrap()
            .size(),
    )
    .unwrap();
    let controller = controller_memory();
    let denied_host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(controller + bill - 1)),
        CancellationSource::new().token(),
        None,
    );
    let denied = runtime
        .history
        .capture_original_branch_schema_inputs(&ExecutionRequest::serial(&denied_host))
        .unwrap_err();
    let OriginalBranchSchemaInputsDenial::Request(cause) = denied else {
        panic!("real roster admission must refuse");
    };
    let cause: crate::execution::RelationalExecutionDenialCause = cause.into();
    match cause {
        crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
            requested,
            admitted,
            ancestor: 0,
        } => {
            assert_eq!(requested, bill);
            assert_eq!(admitted, bill - 1);
        }
        other => panic!("actual entered roster allocation required: {other:?}"),
    }
    assert_eq!(denied_host.memory().unwrap().charged_memory_bytes(), 0);
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(64 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let capture = runtime
        .history
        .capture_original_branch_schema_inputs(&ExecutionRequest::serial(&host))
        .unwrap();
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), bill);
    drop(runtime);
    assert!(!capture
        .birth
        .as_ref()
        .unwrap()
        .schema
        .registry()
        .entity_kinds
        .is_empty());
    assert_eq!(capture.transitions().len(), 1);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), bill);
    drop(capture);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);

    let cancel = CancellationSource::new();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(64 * 1024)),
        cancel.token(),
        None,
    );
    let runtime = runtime_with_test_schema();
    cancel.cancel();
    assert!(matches!(
        runtime
            .history
            .capture_original_branch_schema_inputs(&ExecutionRequest::serial(&host)),
        Err(OriginalBranchSchemaInputsDenial::Request(_))
    ));
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    assert!(roster_bytes(usize::MAX).is_none());
}

#[test]
fn transition_backing_refusal_precedes_archive_and_retains_its_own_grant() {
    let mut runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    // Isolate the stream backing bill from the producer's independently owned
    // original schema carrier, now required before every transition durable step.
    let setup = crate::test_execution_policy::serial_request();
    runtime
        .history
        .branch_cell(identity.branch_id())
        .unwrap()
        .root()
        .unwrap()
        .retained_schema_authority()
        .recorded_image(&ExecutionRequest::serial(&setup))
        .unwrap();
    assert!(setup.memory().unwrap().charged_memory_bytes() > 0);
    let bill = u64::try_from(
        std::alloc::Layout::array::<RetainedBranchTransition>(4)
            .unwrap()
            .size(),
    )
    .unwrap();
    let controller = controller_memory();
    let denied_host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(controller + bill - 1)),
        CancellationSource::new().token(),
        None,
    );
    let Err(crate::branch::RelationalBranchTransitionStop::RequestStopped(cause)) =
        runtime.archive_branch(&ExecutionRequest::serial(&denied_host), &identity)
    else {
        panic!("actual transition backing must refuse before archive");
    };
    let cause: crate::execution::RelationalExecutionDenialCause = cause.into();
    match cause {
        crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
            requested,
            admitted,
            ancestor: 0,
        } => {
            assert_eq!(requested, bill);
            assert_eq!(admitted, bill - 1);
        }
        other => panic!("actual entered transition allocation required: {other:?}"),
    }
    assert!(runtime.observe_branch(&identity).is_ok());
    assert_eq!(
        runtime
            .history
            .canonical_checkpoint_gate()
            .branch_transition_head()
            .0,
        0
    );
    assert_eq!(denied_host.memory().unwrap().charged_memory_bytes(), 0);
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(64 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    runtime
        .archive_branch(&ExecutionRequest::serial(&host), &identity)
        .unwrap();
    assert_eq!(
        runtime
            .history
            .canonical_checkpoint_gate()
            .branch_transition_head()
            .0,
        1
    );
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), bill);
    drop(runtime);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    assert_eq!(setup.memory().unwrap().charged_memory_bytes(), 0);
}
