//! The rules a republication and a restored row each keep.

use std::sync::Arc;

use super::{
    admission, checkpoint_identity,
    records::{assert_continues_the_performed_read, continues_input, input_key, performed, INPUT},
    selects, source_entity, staged, IDEMPOTENCY_KEY, PARTITION,
};
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryObservedSourceSelection,
    invariant_projection::ConsumedOutputEvidence,
    output_lineage::{
        invalidation::{register_republished, FullVerificationReason},
        retained_capacity::LineageRetentionLedger,
        RecordedOutput,
    },
    WorthQueryApplicationObservedFact as Fact,
};

const ALIAS_INPUT: [u8; 32] = [0x63; 32];

#[test]
fn a_republication_continues_the_suspended_performed_record() {
    staged(|mut stage| {
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
            .republish(
                suspended_generation,
                &equal,
                &restored_witness,
                &mut admission(4_096)
            )
            .is_none());

        let continued = stage
            .republish(
                suspended_generation,
                &facts,
                &restored_witness,
                &mut admission(4_096),
            )
            .expect("a performed record's proof continues")
            .expect("the restoration's address takes the republication");
        let suspended = suspended.get().unwrap();
        assert!(Arc::ptr_eq(
            &continued.predecessor,
            &suspended.settlement_identity
        ));
        assert!(Arc::ptr_eq(
            continued.facts.for_comparison().unwrap().facts(),
            &facts
        ));
        assert!(Arc::ptr_eq(&continued.consumed_outputs, &consumed));
        assert!(Arc::ptr_eq(&continued.witness, &restored_witness));

        let restored = stage.restored();
        assert!(Arc::ptr_eq(
            &restored.settlement_identity,
            &continued.identity
        ));
        assert!(Arc::ptr_eq(&restored.consumed_outputs, &consumed));
        assert!(Arc::ptr_eq(
            restored
                .observed_source_facts()
                .unwrap()
                .for_comparison()
                .unwrap()
                .facts(),
            &facts
        ));
        // The row is an origin of its own over the re-created entities.
        assert!(Arc::ptr_eq(
            restored.native_output_witness_cell().unwrap(),
            &restored_witness
        ));
        assert!(restored._retained_capacity.is_none());
        assert_continues_the_performed_read(restored);
        assert!(continues_input(restored, &selection, INPUT));
        assert!(!continues_input(restored, &selection, [0x62; 32]));

        // The same restoration recorded again keeps its first republication.
        assert!(stage
            .republish(
                suspended_generation,
                &facts,
                &performed_witness,
                &mut admission(4_096)
            )
            .expect("the suspended record still continues")
            .is_none());
        let restored = stage.restored();
        assert_eq!(restored.verification_requirement(), None);
        assert!(Arc::ptr_eq(
            restored.native_output_witness_cell().unwrap(),
            &restored_witness
        ));
    });
}

#[test]
fn a_restored_row_without_performed_proof_stays_fresh_until_verified() {
    staged(|mut stage| {
        let facts: Arc<[Fact]> = Arc::from([source_entity(1), source_entity(2)]);
        let suspended_generation = stage.restored_generation() - 1;
        let restored_witness = Arc::clone(&stage.restored_witness);
        let checkpoint_row = stage.retain(suspended_generation, |row| row, &facts);
        checkpoint_row
            .get()
            .unwrap()
            .require_verification(FullVerificationReason::CheckpointRestore);

        assert!(stage
            .republish(
                suspended_generation,
                &facts,
                &restored_witness,
                &mut admission(4_096)
            )
            .is_none());
        stage.lineage.record_restoration(
            stage.lineage.binding_type(&stage.source.output_binding),
            stage.source.runtime_authority,
            stage.source.schema.clone(),
            stage.source.scope,
            stage.observation,
            Arc::clone(&stage.correspondence),
            checkpoint_identity([0x31; 32]),
            PARTITION,
            None,
            IDEMPOTENCY_KEY,
            crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence::for_test(false).retain_facts(Arc::clone(&facts)),
            None,
            None,
        );
        let restored = stage.restored();
        assert_eq!(
            restored.verification_requirement(),
            Some(FullVerificationReason::CheckpointRestore)
        );
        assert!(restored.completed_handler_facts.is_none());
        assert!(restored.completed_decision_reuse.is_none());
        assert!(restored.prepared_input_reuse_key.is_none());
        assert!(restored.native_output_witness_cell().is_none());
    });
}

#[test]
fn a_republished_stable_alias_holds_the_performed_sequence() {
    staged(|mut stage| {
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
            .prepare_fact_projection(&mut admission(4_096))
            .unwrap()
            .unwrap();
        let mut combined = Vec::with_capacity(projection.count() + 2);
        combined.push(source_entity(1));
        projection.append_into(&mut combined);
        combined.push(source_entity(3));
        assert!(combined.len() > 2, "the installed output projects facts");
        let alias_facts: Arc<[Fact]> = Arc::from(combined);
        stage.retain(
            suspended_generation,
            |row| RecordedOutput {
                native_prior_checkpoint: None,
                computation_source: crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence::for_test(false),
                performed_origin: Some(Arc::clone(&origin)),
                consumed_outputs: Arc::clone(&consumed),
                prepared_input_reuse_key: Some(input_key(&selection, ALIAS_INPUT)),
                ..row
            },
            &alias_facts,
        );

        // The recomposed sequence is admitted and reserved before it exists.
        stage.lineage.retention = LineageRetentionLedger::new(0);
        assert!(stage
            .republish(
                suspended_generation,
                &alias_facts,
                &restored_witness,
                &mut admission(4_096)
            )
            .is_none());
        stage.lineage.retention = LineageRetentionLedger::default();
        assert!(stage
            .republish(
                suspended_generation,
                &alias_facts,
                &restored_witness,
                &mut admission(0)
            )
            .is_none());

        let continued = stage
            .republish(
                suspended_generation,
                &alias_facts,
                &restored_witness,
                &mut admission(4_096),
            )
            .expect("an alias continues its origin's proof")
            .expect("the restoration's address takes the republication");
        assert_eq!(continued.facts.len(), 2);
        assert!(selects(&continued.facts.for_comparison().unwrap()[0], 1));
        assert!(selects(&continued.facts.for_comparison().unwrap()[1], 3));
        assert!(Arc::ptr_eq(&continued.consumed_outputs, &consumed));

        let restored = stage.restored();
        assert!(Arc::ptr_eq(
            restored
                .observed_source_facts()
                .unwrap()
                .for_comparison()
                .unwrap()
                .facts(),
            continued.facts.for_comparison().unwrap().facts()
        ));
        assert!(Arc::ptr_eq(
            restored.native_output_witness_cell().unwrap(),
            &restored_witness
        ));
        assert!(restored._retained_capacity.is_some());
        assert_continues_the_performed_read(restored);
        // The alias's own prepared input continues, not its origin's.
        assert!(continues_input(restored, &selection, ALIAS_INPUT));
        assert!(!continues_input(restored, &selection, INPUT));
    });
}

#[test]
fn a_stable_alias_is_compared_by_the_witness_of_its_origin() {
    staged(|mut stage| {
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
            |row| RecordedOutput {
                native_prior_checkpoint: None,
                computation_source: crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence::for_test(false),
                performed_origin: Some(Arc::clone(&origin)),
                consumed_outputs: Arc::clone(&consumed),
                prepared_input_reuse_key: Some(input_key(&selection, ALIAS_INPUT)),
                ..row
            },
            &alias_facts,
        );

        let (candidates, _) = stage
            .lineage
            .retained_output_candidates(
                stage.source.runtime_authority,
                &stage.source.schema,
                stage.source.scope,
                stage.observation.lifecycle_incarnation(),
                alias_generation,
                &[stage.lineage.binding_type(&stage.source.output_binding)],
                PARTITION,
                4_096,
            )
            .expect("the candidate lookup is admitted");
        let [candidate] = candidates.as_slice() else {
            panic!("the partition retains one latest output");
        };
        assert!(Arc::ptr_eq(
            candidate.observed_source_facts.as_ref().unwrap().facts(),
            &alias_facts
        ));
        // Selection compares the output half of the alias's fact set, so a
        // moved output ends its exact reuse as it ends its origin's.
        assert!(candidate
            .native_output_witness
            .as_ref()
            .is_some_and(|witness| Arc::ptr_eq(witness, &performed_witness)));
    });
}

#[test]
fn a_republication_whose_predecessor_holds_no_mark_row_is_verified_in_full() {
    staged(|mut stage| {
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
            .republish(
                suspended_generation,
                &facts,
                &restored_witness,
                &mut admission(4_096),
            )
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
            &mut stage.owner.edit_admission(),
        );
        let reason = refused.expect_err("an unregistered predecessor refuses the registration");
        assert_eq!(reason, FullVerificationReason::MissingSettlement);
        stage
            .lineage
            .require_settlement_verification(&continued.identity, reason);
        assert_eq!(
            stage.restored().verification_requirement(),
            Some(FullVerificationReason::MissingSettlement)
        );
    });
}
