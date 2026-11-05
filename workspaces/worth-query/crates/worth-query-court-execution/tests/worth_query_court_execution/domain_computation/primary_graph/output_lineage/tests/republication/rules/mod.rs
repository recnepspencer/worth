//! The rules a republication and a restored row each keep.

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::tests::TestsWorthQueryObservedSourceSelectionForTest;

use std::sync::Arc;

use crate::domain_computation::primary_graph::output_lineage::tests::checkpoint_identity;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::records::assert_continues_the_performed_read;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::records::continues_input;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::records::input_key;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::records::performed;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::records::INPUT;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::selects;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::source_entity;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::staged;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::IDEMPOTENCY_KEY;
use crate::domain_computation::primary_graph::output_lineage::tests::republication::PARTITION;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::publication::register_republished;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutput;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;

pub const ALIAS_INPUT: [u8; 32] = [0x63; 32];

#[test]
fn a_republication_continues_the_suspended_performed_record() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    staged(request_execution, |mut stage| {
        let selection = WorthQueryObservedSourceSelection::selecting_for_test(2);
        let facts: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(2)]);
        let consumed: Arc<[ConsumedOutputEvidence]> = Arc::from([]);
        let suspended_generation = stage.restored_generation() - 1;
        let performed_witness = Arc::clone(&stage.performed_witness);
        let restored_witness = Arc::clone(&stage.restored_witness);
        let suspended = stage.retain(
            suspended_generation,
            |row| performed(row, &selection, &consumed, &performed_witness),
            &facts,
        );

        // Another retained sequence is not the record this suspension
        // qualified, however equal its content.
        let equal: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(2)]);
        assert!(stage
            .republish(suspended_generation, &equal, &restored_witness)
            .is_none());

        let continued = stage
            .republish(suspended_generation, &facts, &restored_witness)
            .expect("a performed record's proof continues")
            .expect("the restoration's address takes the republication");
        let suspended = suspended.get().unwrap();
        assert!(Arc::ptr_eq(
            &continued.predecessor,
            suspended.settlement_identity_for_test()
        ));
        assert!(continued
            .facts
            .for_comparison()
            .unwrap()
            .facts_for_test()
            .same_backing_for_test(&Arc::clone(&facts).into()));
        assert!(Arc::ptr_eq(&continued.consumed_outputs, &consumed));
        assert!(Arc::ptr_eq(&continued.witness, &restored_witness));

        let restored = stage.restored();
        assert!(Arc::ptr_eq(
            restored.settlement_identity_for_test(),
            &continued.identity
        ));
        assert!(Arc::ptr_eq(restored.consumed_outputs_for_test(), &consumed));
        assert!(restored
            .observed_source_facts_for_test()
            .unwrap()
            .for_comparison()
            .unwrap()
            .facts_for_test()
            .same_backing_for_test(&Arc::clone(&facts).into()));
        // The row is an origin of its own over the re-created entities.
        assert!(Arc::ptr_eq(
            restored.native_output_witness_cell_for_test().unwrap(),
            &restored_witness
        ));

        assert_continues_the_performed_read(restored);
        assert!(continues_input(restored, &selection, INPUT));
        assert!(!continues_input(restored, &selection, [0x62; 32]));

        // The same restoration recorded again keeps its first republication.
        assert!(stage
            .republish(suspended_generation, &facts, &performed_witness)
            .expect("the suspended record still continues")
            .is_none());
        let restored = stage.restored();
        assert_eq!(restored.verification_requirement_for_test(), None);
        assert!(Arc::ptr_eq(
            restored.native_output_witness_cell_for_test().unwrap(),
            &restored_witness
        ));
    });
}

#[test]
fn a_restored_row_without_performed_proof_stays_fresh_until_verified() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    staged(request_execution, |mut stage| {
        let facts: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(2)]);
        let suspended_generation = stage.restored_generation() - 1;
        let restored_witness = Arc::clone(&stage.restored_witness);
        let checkpoint_row = stage.retain(suspended_generation, |row| row, &facts);
        checkpoint_row
            .get()
            .unwrap()
            .require_verification_for_test(FullVerificationReason::CheckpointRestore);

        assert!(stage
            .republish(suspended_generation, &facts, &restored_witness)
            .is_none());
        stage.lineage.record_restoration_for_test(
            stage.lineage.binding_type_for_test(stage.source.output_binding_for_test()),
            (*stage.source.runtime_authority_for_test()),
            (*stage.source.schema_for_test()).clone(),
            stage.source.scope,
            stage.observation,
            Arc::clone(&stage.correspondence),
            checkpoint_identity([0x31; 32]),
            PARTITION,
            None,
            IDEMPOTENCY_KEY,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(Arc::clone(&facts)),
            None,
            None,
            None,
        );
        let restored = stage.restored();
        assert_eq!(
            restored.verification_requirement_for_test(),
            Some(FullVerificationReason::CheckpointRestore)
        );
        assert!((*restored.completed_handler_facts_for_test()).is_none());
        assert!((*restored.completed_decision_reuse_for_test()).is_none());
        assert!((*restored.prepared_input_reuse_key_for_test()).is_none());
        assert!(restored.native_output_witness_cell_for_test().is_none());
    });
}

#[test]
fn a_republished_stable_alias_holds_the_performed_sequence() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    staged(request_execution, |mut stage| {
        let selection = WorthQueryObservedSourceSelection::selecting_for_test(2);
        let consumed: Arc<[ConsumedOutputEvidence]> = Arc::from([]);
        let suspended_generation = stage.restored_generation() - 1;
        let origin_generation = suspended_generation
            .checked_sub(1)
            .expect("the fixture occurrence has two earlier generations");
        let performed_witness = Arc::clone(&stage.performed_witness);
        let restored_witness = Arc::clone(&stage.restored_witness);
        let origin_facts: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(2)]);
        let origin = stage.retain(
            origin_generation,
            |row| performed(row, &selection, &consumed, &performed_witness),
            &origin_facts,
        );
        // A stable alias: the origin's handler prefix, the origin's performed
        // output projected as facts, then the alias's own source suffix.
        let projection = performed_witness
            .get()
            .unwrap()
            .prepare_fact_projection_for_test()
            .unwrap()
            .unwrap();
        let mut combined = Vec::with_capacity(projection.count_for_test() + 2);
        combined.push(source_entity(1));
        projection.append_into_for_test(&mut combined);
        combined.push(source_entity(3));
        assert!(combined.len() > 2, "the installed output projects facts");
        let alias_facts: Arc<[Fact]> = Arc::from(combined);
        stage.retain(
            suspended_generation,
            |row| {
                let mut row = { let __coupled_completion = None; let __producing_implementation = None; let __required_publication = None; let __native_prior_checkpoint = None; let __computation_source = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false); let __performed_origin = Some(Arc::clone(&origin)); let __consumed_outputs = Arc::clone(&consumed); let __prepared_input_reuse_key = Some(input_key(&selection, ALIAS_INPUT)); let mut __base = row; *__base.coupled_completion_mut_for_test() = __coupled_completion; *__base.producing_implementation_mut_for_test() = __producing_implementation; *__base.required_publication_mut_for_test() = __required_publication; *__base.native_prior_checkpoint_mut_for_test() = __native_prior_checkpoint; *__base.computation_source_mut_for_test() = __computation_source; *__base.performed_origin_mut_for_test() = __performed_origin; *__base.consumed_outputs_mut_for_test() = __consumed_outputs; *__base.prepared_input_reuse_key_mut_for_test() = __prepared_input_reuse_key; __base };
                let spans = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_cutoff::stable_publication::StableFactSpans::new_for_test(
                    alias_facts.clone().into(), 1, alias_facts.len() - 2).unwrap();
                let mutable = (*row.mutable_mut_for_test()).get_mut().unwrap();
                // Local representation proof: real witness projection above,
                // no source/currentness authority is manufactured by spans.
                mutable.set_stable_fact_spans_for_test(spans);
                row
            },
            &alias_facts,
        );

        let continued = stage
            .republish(suspended_generation, &alias_facts, &restored_witness)
            .expect("an alias continues its origin's proof")
            .expect("the restoration's address takes the republication");
        assert_eq!(continued.facts.len_for_test(), 2);
        assert!(selects(&continued.facts.for_comparison().unwrap()[0], 1));
        assert!(selects(&continued.facts.for_comparison().unwrap()[1], 3));
        assert!(Arc::ptr_eq(&continued.consumed_outputs, &consumed));

        let restored = stage.restored();
        assert!(restored
            .observed_source_facts_for_test()
            .unwrap()
            .for_comparison()
            .unwrap()
            .facts_for_test()
            .same_backing_for_test(continued.facts.for_comparison().unwrap().facts_for_test()));
        assert!(Arc::ptr_eq(
            restored.native_output_witness_cell_for_test().unwrap(),
            &restored_witness
        ));

        assert_continues_the_performed_read(restored);
        // The alias's own prepared input continues, not its origin's.
        assert!(continues_input(restored, &selection, ALIAS_INPUT));
        assert!(!continues_input(restored, &selection, INPUT));
    });
}

#[test]
fn a_stable_alias_is_compared_by_the_witness_of_its_origin() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    staged(request_execution, |mut stage| {
        let selection = WorthQueryObservedSourceSelection::selecting_for_test(2);
        let consumed: Arc<[ConsumedOutputEvidence]> = Arc::from([]);
        let alias_generation = stage.restored_generation() - 1;
        let performed_witness = Arc::clone(&stage.performed_witness);
        let origin_facts: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(2)]);
        let origin = stage.retain(
            alias_generation - 1,
            |row| performed(row, &selection, &consumed, &performed_witness),
            &origin_facts,
        );
        // The alias seals no witness of its own: its output is its origin's.
        let alias_facts: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(3)]);
        stage.retain(
            alias_generation,
            |row| { let __coupled_completion = None; let __producing_implementation = None; let __required_publication = None; let __native_prior_checkpoint = None; let __computation_source = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false); let __performed_origin = Some(Arc::clone(&origin)); let __consumed_outputs = Arc::clone(&consumed); let __prepared_input_reuse_key = Some(input_key(&selection, ALIAS_INPUT)); let mut __base = row; *__base.coupled_completion_mut_for_test() = __coupled_completion; *__base.producing_implementation_mut_for_test() = __producing_implementation; *__base.required_publication_mut_for_test() = __required_publication; *__base.native_prior_checkpoint_mut_for_test() = __native_prior_checkpoint; *__base.computation_source_mut_for_test() = __computation_source; *__base.performed_origin_mut_for_test() = __performed_origin; *__base.consumed_outputs_mut_for_test() = __consumed_outputs; *__base.prepared_input_reuse_key_mut_for_test() = __prepared_input_reuse_key; __base },
            &alias_facts,
        );

        let candidates = stage
            .lineage
            .retained_output_candidates_metered_for_test(
                (*stage.source.runtime_authority_for_test()),
                stage.source.schema_for_test(),
                stage.source.scope,
                stage.observation.lifecycle_incarnation(),
                alias_generation,
                &[stage
                    .lineage
                    .binding_type_for_test(stage.source.output_binding_for_test())],
                PARTITION,
                request_execution,
            )
            .expect("the candidate lookup is admitted");
        let [candidate] = candidates.as_slice() else {
            panic!("the partition retains one latest output");
        };
        assert!((*candidate.observed_source_facts_for_test())
            .as_ref()
            .unwrap()
            .facts_for_test()
            .same_backing_for_test(&Arc::clone(&alias_facts).into()));
        // Selection compares the output half of the alias's fact set, so a
        // moved output ends its exact reuse as it ends its origin's.
        assert!((*candidate.native_output_witness_for_test())
            .as_ref()
            .is_some_and(|witness| Arc::ptr_eq(witness, &performed_witness)));
    });
}

#[test]
fn a_republication_whose_predecessor_holds_no_mark_row_is_verified_in_full() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    staged(request_execution, |mut stage| {
        let selection = WorthQueryObservedSourceSelection::selecting_for_test(2);
        let facts: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(2)]);
        let consumed: Arc<[ConsumedOutputEvidence]> = Arc::from([]);
        let suspended_generation = stage.restored_generation() - 1;
        let performed_witness = Arc::clone(&stage.performed_witness);
        let restored_witness = Arc::clone(&stage.restored_witness);
        stage.retain(
            suspended_generation,
            |row| performed(row, &selection, &consumed, &performed_witness),
            &facts,
        );
        let continued = stage
            .republish(suspended_generation, &facts, &restored_witness)
            .unwrap()
            .unwrap();

        // This suspended record never registered with the mark owner, so the
        // republication has no currentness to continue there.

        let refused = register_republished(
            stage.owner,
            &continued.predecessor,
            Arc::clone(&continued.identity),
            continued.facts,
            &continued.consumed_outputs,
            continued.witness.get().unwrap(),
            stage.read_basis.take().unwrap(),
        );
        let reason = refused.expect_err("an unregistered predecessor refuses the registration");
        assert_eq!(reason, FullVerificationReason::MissingSettlement);
        stage
            .lineage
            .require_settlement_verification_for_test(&continued.identity, reason);
        assert_eq!(
            stage.restored().verification_requirement_for_test(),
            Some(FullVerificationReason::MissingSettlement)
        );
    });
}
