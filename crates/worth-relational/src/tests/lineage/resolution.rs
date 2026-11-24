use crate::facade::diagnostics::{DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope};
use crate::facade::history::{AspectHistoryLineageEventSpan, BranchId, HistoryAspectQueryTarget};
use crate::facade::identity::{KindId, PartitionId};
use crate::facade::lineage::{
    HistoricalResolutionBoundednessBasis, HistoricalResolutionDigestMode,
    HistoricalResolutionRequest, LineageGraphRequest, LineageGraphTraversalBasis,
};
use crate::facade::transactions::{
    EntityMutationIntent, MutationIntent, ReplaceEntityIntent, WorkerIntentBatch,
};
use crate::tests::support::*;
use worth_relational_contracts::internal_port::lineage::data::graph::LineageNodeFieldsPort as _;
use worth_relational_lineage::internal_port::lineage::index::batch::PersistentNameCommittedBatchFixturePort as _;

// CONTRACT: historical_lineage_resolution
// LANES: success, adversarial, recovery

#[test]
fn historical_lineage_resolution_follows_replace_events() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    let start_lineage = *runtime
        .lineage_access()
        .for_record(execution, entity)
        .unwrap()
        .unwrap()
        .owner_lineage_id();

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        replace_batch(entity),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(execution, &runtime).unwrap();
    let resolution = runtime
        .lineage_access()
        .resolve_historical_lineage(execution, main_resolution_request(start_lineage))
        .unwrap();

    assert_eq!(resolution.start, start_lineage);
    assert_eq!(
        resolution.boundedness_basis,
        HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed
    );
    assert_eq!(resolution.traversed_event_ids.len(), 1);
    assert_eq!(resolution.resolved.len(), 1);
    assert_ne!(resolution.resolved[0], start_lineage);
    assert_eq!(resolution.metrics.traversed_event_count, 1);
    assert!(resolution.metrics.event_visit_count >= 1);
    assert_eq!(resolution.metrics.resolved_lineage_count, 1);
    assert_eq!(
        resolution.digest_basis().digest_mode(),
        HistoricalResolutionDigestMode::ExactDigestCanonicalOrder
    );
    assert_eq!(resolution.digest_basis().requested_start(), start_lineage);
    assert_eq!(
        resolution.digest_basis().canonical_traversed_event_ids(),
        resolution.traversed_event_ids
    );
    assert_eq!(resolution.trace.digest_basis(), resolution.digest_basis());
    assert_eq!(
        runtime
            .lineage_access()
            .graph(execution, main_graph_request())
            .unwrap()
            .events()
            .iter()
            .filter(|event| event.commit().commit_id == outcome.commit.commit_id)
            .count(),
        2
    );
}

#[test]
fn historical_lineage_resolution_does_not_scan_unrelated_branch_events() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    let start_lineage = *runtime
        .lineage_access()
        .for_record(execution, entity)
        .unwrap()
        .unwrap()
        .owner_lineage_id();

    for index in 0..6 {
        let label = format!("unrelated-{index}");
        let _ = create_entity_outcome(execution, &runtime, &label);
    }

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        replace_batch(entity),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let _ = txn.commit(execution, &runtime).unwrap();

    let total_branch_events = runtime
        .lineage_access()
        .graph(execution, main_graph_request())
        .unwrap()
        .metrics()
        .event_count;

    let resolution = runtime
        .lineage_access()
        .resolve_historical_lineage(execution, main_resolution_request(start_lineage))
        .unwrap();

    assert_eq!(resolution.metrics.traversed_event_count, 1);
    assert_eq!(resolution.metrics.event_visit_count, 1);
    assert!(total_branch_events > resolution.metrics.event_visit_count);
}

#[test]
fn lineage_aspect_history_keeps_origin_events_and_marks_resolution_context() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    let start_lineage = *runtime
        .lineage_access()
        .for_record(execution, entity)
        .unwrap()
        .unwrap()
        .owner_lineage_id();

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        replace_batch(entity),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let replacement = txn.commit(execution, &runtime).unwrap();
    let history = runtime
        .lineage_access()
        .entity_aspect_history(execution, main_resolution_request(start_lineage), None)
        .unwrap()
        .unwrap();
    let traced = runtime
        .lineage_access()
        .entity_aspect_history_with_trace(execution, main_resolution_request(start_lineage), None)
        .unwrap();
    let artifact = traced.trace.diagnostic_artifact();
    let digest = traced.lineage_aspect_resolution_digest();
    assert_eq!(history.start_lineage_id, start_lineage);
    assert_eq!(
        traced.trace.requested_target,
        HistoryAspectQueryTarget::Lineage(start_lineage)
    );
    assert_eq!(traced.trace.returned_entries, 3);
    assert_eq!(traced.trace.traversed_lineage_events, 1);
    assert_eq!(
        traced.trace.searched_lineage_event_span,
        Some(AspectHistoryLineageEventSpan {
            first_event_id: history.traversed_event_ids[0],
            last_event_id: history.traversed_event_ids[0],
        })
    );
    assert_eq!(artifact.scope, DiagnosticsScope::Lineage);
    assert_eq!(artifact.kind, DiagnosticsArtifactKind::DetailedTrace);
    assert_eq!(
        artifact.entries[0].code,
        DiagnosticCode::LineageAspectHistoryResolved
    );
    assert_eq!(
        digest.requested_target,
        HistoryAspectQueryTarget::Lineage(start_lineage)
    );
    assert_eq!(digest.entry_count, 3);
    assert_eq!(digest.traversed_lineage_events, 1);
    assert_eq!(
        digest.resolved_lineage_chain_len,
        history.resolved_lineage_chain.len() as u64
    );
    assert_eq!(traced.history.as_ref(), Some(&history));
    assert_eq!(history.traversed_event_ids.len(), 1);
    assert_eq!(history.entries.len(), 3);
    assert_lineage_history_origin_invariants(&history.entries, start_lineage);
    assert_eq!(
        history.entries[0].origin.commit_id,
        created.commit.commit_id
    );
    assert_eq!(
        history.entries[1].origin.commit_id,
        replacement.commit.commit_id
    );
    assert_eq!(
        history.entries[2].origin.commit_id,
        replacement.commit.commit_id
    );
}

#[test]
fn refused_durable_publication_leaves_no_replacement_lineage_behind() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let created = create_entity_outcome(execution, &runtime, "refused-lineage-source");
    let entity = changed_entities(&created)[0];
    let start_lineage = runtime
        .lineage_access()
        .for_record(execution, entity)
        .unwrap()
        .expect("source lineage")
        .lineage_id();
    let graph = |rt: &crate::runtime::RelationalRuntime| {
        rt.lineage_access()
            .graph(execution, main_graph_request())
            .unwrap()
    };
    let before = graph(&runtime);
    let head_before = runtime.history().branch_head(&BranchId("main".to_owned()));

    medium.script_next_publication(ScriptedPublication::ProvenNoEffect);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            replace_batch_with(entity, "refused-replacement", "refused-successor"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let refused = transaction
        .commit(execution, &runtime)
        .expect_err("a refused durable publication is a typed pre-effect denial");
    assert!(matches!(
        refused,
        crate::facade::transactions::TransactionCommitError::PublicationDenied {
            denial: crate::mvcc::RelationalPublicationDenial::DurablePublicationRefused(_),
            ..
        }
    ));
    let after = graph(&runtime);
    assert_eq!(after.nodes().len(), before.nodes().len());
    assert_eq!(after.events().len(), before.events().len());
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".to_owned())),
        head_before
    );
    let resolution = runtime
        .lineage_access()
        .resolve_historical_lineage(execution, main_resolution_request(start_lineage))
        .unwrap();
    assert_eq!(resolution.resolved, vec![start_lineage]);
    assert!(resolution.traversed_event_ids.is_empty());
}

#[test]
fn historical_lineage_resolution_traverses_restore() {
    use super::name_world_history::restore_traversal_cases;

    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    for case in restore_traversal_cases() {
        let runtime = runtime_with_test_schema();
        for batch in &case.log {
            runtime
                .lineage
                .install_recovered_event_batch(batch.lineage_events_for_test(), batch.commit())
                .expect("install finalized lineage batch");
        }
        let resolution = runtime
            .lineage_access()
            .resolve_historical_lineage(execution, main_resolution_request(case.start_lineage))
            .expect("historical lineage resolution succeeds across Restore");
        case.assert_agrees_with_resolution(&resolution);
    }
}

#[test]
fn commit_finalization_with_births_attaches_birth_and_refuses_with_zero_effects() {
    use super::birth_fixtures::{
        commit_receipt, create_postures_input, face_birth_request, face_scheme_registry,
        stage_two_entities, staged_entity_lineage_id,
    };
    use crate::facade::identity::EntityId;
    use crate::facade::lineage::{
        LineageEventKind, LineageNameDenial, LineagePostureKind, LineageSubject,
        PersistentNameHolder,
    };
    use crate::lineage::authority::PersistentNameMintingAuthority;
    use crate::transactions::data::RecordRef;
    use std::collections::{BTreeMap, BTreeSet};

    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = runtime_with_test_schema();
    let prep = runtime.preparation_runtime_snapshot();
    let authority = prep.lineage_preparation_authority();
    let registry = face_scheme_registry();
    let receipt = commit_receipt();
    let (empty_holders, empty_retired) = (BTreeMap::new(), BTreeSet::new());
    let before = prep.lineage_identity.frontiers();

    // 1. Create birth targeting uncreated entity refuses typed with zero allocator or staged effects.
    let (mut staged_missing, e0, _) = stage_two_entities();
    let phantom = EntityId::new(PartitionId::main(), 99, 1);
    let missing_input = create_postures_input(
        registry.clone(),
        vec![(
            phantom,
            face_birth_request("occ.phantom", "authored", "", phantom),
        )],
    );
    let missing_err = authority
        .ensure_lineage_for_commit_with_postures_for_test(
            execution,
            &mut staged_missing,
            &receipt,
            &[],
            &[RecordRef::Entity(e0)],
            &missing_input,
        )
        .err()
        .expect("birth targeting uncreated entity must refuse");
    assert!(matches!(
        missing_err.lineage_name_denial_ref(),
        Some(LineageNameDenial::LineagePostureMismatch {
            posture: LineagePostureKind::Create,
            subject: LineageSubject::Entity,
            holder,
            ..
        }) if holder == &PersistentNameHolder::Entity(phantom)
    ));
    assert_eq!(prep.lineage_identity.frontiers(), before);
    assert_eq!(staged_entity_lineage_id(&staged_missing, e0), None);

    // 2. DuplicateBirth collision refuses typed with both holders and zero effects.
    let (mut staged_dup, d0, d1) = stage_two_entities();
    let dup_input = create_postures_input(
        registry.clone(),
        vec![
            (d0, face_birth_request("occ.dup", "cut_piece", "same", d0)),
            (d1, face_birth_request("occ.dup", "cut_piece", "same", d1)),
        ],
    );
    let dup_err = authority
        .ensure_lineage_for_commit_with_postures_for_test(
            execution,
            &mut staged_dup,
            &receipt,
            &[],
            &[RecordRef::Entity(d0), RecordRef::Entity(d1)],
            &dup_input,
        )
        .err()
        .expect("colliding staged births must refuse");
    assert!(matches!(
        dup_err.lineage_name_denial_ref(),
        Some(LineageNameDenial::DuplicateBirth { first, second, .. })
            if first == &PersistentNameHolder::Entity(d0)
                && second == &PersistentNameHolder::Entity(d1)
    ));
    assert_eq!(prep.lineage_identity.frontiers(), before);
    assert_eq!(staged_entity_lineage_id(&staged_dup, d0), None);
    assert_eq!(staged_entity_lineage_id(&staged_dup, d1), None);

    // 3. Valid staged birth attaches the exact pure-minted birth record to the Create event.
    let (mut staged_ok, ok0, _) = stage_two_entities();
    let ok_req = face_birth_request("occ.face.1", "authored", "", ok0);
    let expected = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &empty_holders,
        &empty_retired,
        std::slice::from_ref(&ok_req),
    )
    .unwrap();
    let ok_input = create_postures_input(registry, vec![(ok0, ok_req)]);
    let prepared = authority
        .ensure_lineage_for_commit_with_postures_for_test(
            execution,
            &mut staged_ok,
            &receipt,
            &[],
            &[RecordRef::Entity(ok0)],
            &ok_input,
        )
        .expect("valid staged birth finalizes cleanly");
    let events = prepared.artifact().event_batch().events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind(), LineageEventKind::Create);
    assert_eq!(events[0].name_birth(), Some(&expected[0]));
}

mod request_fixture;
use request_fixture::{
    main_graph_request, main_resolution_request, replace_batch, replace_batch_with,
};
