//! Redo admission unit evidence for Relational-owned lineage (Gate 8.5).

use worth_relational::facade::{
    history::{BranchId, CommitId, RelationalCommitReceipt},
    identity::VersionId,
};

use super::recovery_handle::{WorthQueryRecoveryHandle, WorthQueryRecoveryHandleBindingAxisProbe};
use super::recovery_progression::WorthQueryRecoveryEffectAuthority;
use super::redo_admission::{
    admit_redo_against_relational, map_recovery_denial, WorthQueryPriorRedoObservation,
};
use super::redo_denial::WorthQueryRedoDenialKind;
use super::redo_intent::{
    WorthQueryProvedUndo, WorthQueryProvedUndoAxisProbe, WorthQueryRedoIntent,
};
use super::redo_recovery::WorthQueryRedoRecovery;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;

fn relational_commit(id: u64) -> RelationalCommitReceipt {
    RelationalCommitReceipt {
        commit_id: CommitId(id),
        version_id: VersionId(id),
        branch_id: BranchId("main".to_owned()),
        parents: id.checked_sub(1).map(CommitId).into_iter().collect(),
    }
}

fn probe_handle(governed_input_identity: Option<[u8; 32]>) -> WorthQueryRecoveryHandle {
    let binding = WorthQueryRecoveryHandleBindingAxisProbe::real()
        .runtime_instance_id(7)
        .branch(BranchId("main".to_owned()))
        .installed_operation([0x44; 32])
        .attempt_commit_id(10)
        .retained_governed_input_identity(governed_input_identity)
        .idempotency(WorthQueryApplicationIdempotencyBinding::new(
            [0x55; 32], [0x56; 32],
        ))
        .installed_aftermath(super::aftermath_schema_fixture::transfer())
        .expires_at_unix_ms(Some(u64::MAX))
        .finish();
    WorthQueryRecoveryHandle::axis_probe(binding)
}

fn proved_and_intent(
    compatibility_generation: u64,
) -> (WorthQueryProvedUndo, WorthQueryRedoIntent) {
    let proved = WorthQueryProvedUndo::axis_probe(WorthQueryProvedUndoAxisProbe {
        original_operation: [0x44; 32],
        undo_commit_id: 20,
        principal_scope_digest: bound_scope_digest(),
        compatibility_generation,
        runtime_instance: 7,
    });
    let intent = WorthQueryRedoIntent::derive(&proved, relational_commit(20)).expect("derive");
    (proved, intent)
}

fn bound_scope_digest() -> [u8; 32] {
    let binding = WorthQueryRecoveryHandleBindingAxisProbe::real().finish();
    let scope = binding.principal_scope();
    let principal = scope.principal();
    let mut bytes = [0u8; 32];
    bytes[0..4].copy_from_slice(&principal.partition_id().to_le_bytes());
    bytes[4..12].copy_from_slice(&principal.local_slot().to_le_bytes());
    bytes[12..16].copy_from_slice(&principal.generation().to_le_bytes());
    bytes[16..24].copy_from_slice(&scope.runtime_authority().to_le_bytes());
    bytes
}

fn admit(
    proved: WorthQueryProvedUndo,
    intent: &WorthQueryRedoIntent,
    current_head: RelationalCommitReceipt,
    prior_redo: WorthQueryPriorRedoObservation,
    governed_input: Option<[u8; 32]>,
) -> Result<super::WorthQueryRedoAdmission, super::WorthQueryRedoDenial> {
    let handle = probe_handle(governed_input);
    let authority = WorthQueryRecoveryEffectAuthority::mint(
        handle.runtime_authority(),
        handle.authority_identity(),
    );
    admit_redo_against_relational(
        WorthQueryRedoRecovery::axis_probe(proved, handle),
        &authority,
        intent,
        &current_head,
        prior_redo,
    )
}

#[test]
fn lawful_redo_admits_against_exact_relational_head() {
    let (proved, intent) = proved_and_intent(1);
    let admission = admit(
        proved,
        &intent,
        relational_commit(20),
        WorthQueryPriorRedoObservation::Absent,
        Some([0x66; 32]),
    )
    .expect("lawful redo");
    assert_eq!(admission.redo_admission_work().basis_preparations(), 1);
    assert_eq!(admission.redo_admission_work().digest_derivations(), 1);
    assert_eq!(admission.original_input::<()>(), Some(&()));
    assert_eq!(
        admission.idempotency_binding().key_identity(),
        intent.identity().digest().bytes()
    );
    assert_eq!(
        admission.idempotency_binding().intent_identity(),
        &[0x66; 32]
    );
}

#[test]
fn missing_original_governed_input_fails_closed() {
    let (proved, intent) = proved_and_intent(1);
    let denied = admit(
        proved,
        &intent,
        relational_commit(20),
        WorthQueryPriorRedoObservation::Absent,
        None,
    )
    .expect_err("missing input");
    assert_eq!(
        denied.kind(),
        WorthQueryRedoDenialKind::ChangedOperationMeaning
    );
}

#[test]
fn exact_relational_head_drift_invalidates_without_mutating_intent() {
    let (proved, intent) = proved_and_intent(1);
    let bound = intent.bound_relational_head().clone();
    let denied = admit(
        proved,
        &intent,
        relational_commit(21),
        WorthQueryPriorRedoObservation::Absent,
        Some([0x66; 32]),
    )
    .expect_err("diverged");
    assert_eq!(
        denied.kind(),
        WorthQueryRedoDenialKind::DivergenceInvalidation
    );
    assert_eq!(intent.bound_relational_head(), &bound);
}

#[test]
fn copied_intent_and_duplicate_fact_are_distinct_denials() {
    let (proved, _) = proved_and_intent(1);
    let foreign = WorthQueryProvedUndo::axis_probe(WorthQueryProvedUndoAxisProbe {
        original_operation: [0x99; 32],
        undo_commit_id: 20,
        principal_scope_digest: [0x70; 32],
        compatibility_generation: 1,
        runtime_instance: 7,
    });
    let foreign_intent =
        WorthQueryRedoIntent::derive(&foreign, relational_commit(20)).expect("foreign intent");
    let copied = admit(
        proved,
        &foreign_intent,
        relational_commit(20),
        WorthQueryPriorRedoObservation::Absent,
        Some([0x66; 32]),
    )
    .expect_err("copied");
    assert_eq!(copied.kind(), WorthQueryRedoDenialKind::CopiedIntent);

    let (proved, intent) = proved_and_intent(1);
    let duplicate = admit(
        proved,
        &intent,
        relational_commit(20),
        WorthQueryPriorRedoObservation::Committed,
        Some([0x66; 32]),
    )
    .expect_err("duplicate");
    assert_eq!(duplicate.kind(), WorthQueryRedoDenialKind::DuplicateRedo);
}

#[test]
fn current_generation_and_principal_binding_remain_required() {
    let (proved, intent) = proved_and_intent(9);
    let generation = admit(
        proved,
        &intent,
        relational_commit(20),
        WorthQueryPriorRedoObservation::Absent,
        Some([0x66; 32]),
    )
    .expect_err("generation drift");
    assert_eq!(
        generation.kind(),
        WorthQueryRedoDenialKind::ChangedOperationMeaning
    );

    let proved = WorthQueryProvedUndo::axis_probe(WorthQueryProvedUndoAxisProbe {
        original_operation: [0x44; 32],
        undo_commit_id: 20,
        principal_scope_digest: [0xAB; 32],
        compatibility_generation: 1,
        runtime_instance: 7,
    });
    let intent = WorthQueryRedoIntent::derive(&proved, relational_commit(20)).expect("derive");
    let principal = admit(
        proved,
        &intent,
        relational_commit(20),
        WorthQueryPriorRedoObservation::Absent,
        Some([0x66; 32]),
    )
    .expect_err("foreign principal");
    assert_eq!(principal.kind(), WorthQueryRedoDenialKind::ForeignPrincipal);
}

#[test]
fn recovery_denials_keep_typed_mapping() {
    use super::recovery_handle::WorthQueryRecoveryHandleDenialKind as K;
    assert_eq!(
        map_recovery_denial(K::Expired).kind(),
        WorthQueryRedoDenialKind::Stale
    );
    assert_eq!(
        map_recovery_denial(K::CurrentPolicyDenied).kind(),
        WorthQueryRedoDenialKind::NewlyUnauthorized
    );
}

#[test]
fn query_has_no_parallel_lineage_or_replay_residue() {
    for source in [
        include_str!("redo_intent.rs"),
        include_str!("redo_admission.rs"),
        include_str!("redo_progression.rs"),
        include_str!("../primary_graph/application_runtime.rs"),
    ] {
        assert!(!source.contains("WorthQueryLinearLineageChain"));
        assert!(!source.contains("append_linear_lineage"));
        assert!(!source.contains("worth_query_replay"));
        assert!(!source.contains("worth-query-replay"));
    }
    for source in [
        include_str!("causal_fact.rs"),
        include_str!("causal_commit.rs"),
        include_str!("redo_admission.rs"),
    ] {
        assert!(!source.contains("worth_runtime_bridge"));
        assert!(!source.contains("BridgeHistoricalLineage"));
        assert!(!source.contains("BridgeLineageContext"));
    }
}
