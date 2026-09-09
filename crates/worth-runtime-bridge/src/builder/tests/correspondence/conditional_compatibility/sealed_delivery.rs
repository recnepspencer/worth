use worth_proof::TransitionOutcome;
use worth_signal::facade::SignalConditionalDecisionClass;

use super::*;

fn execute_session(
    owner: &crate::facade::BridgeSealedRuntimeAssembly,
    lowering: &Arc<BridgeInstalledConditionalLowering>,
    session: &crate::facade::BridgeConditionalEvaluationSession,
    attempt: u64,
) -> crate::facade::BridgeConditionalDecisionEvidence {
    owner
        .execute_admitted_conditional(
            session,
            crate::facade::BridgeConditionalExecutionRequest {
                lowering,
                query_binding_identity: "sealed-delivery-binding",
                query_capability_identity: 1,
                snapshot_identity: "sealed-delivery-snapshot",
                truth_branch_identity: None,
                bridge_snapshot_identity: Some(&crate::truth_identity_fixtures::truth_snapshot(
                    1, 1,
                )),
                execution_identity: "sealed-delivery-execution",
                attempt,
            },
            &mut (),
        )
        .unwrap()
}

fn admit_source_session(
    owner: &crate::facade::BridgeSealedRuntimeAssembly,
    lowering: &Arc<BridgeInstalledConditionalLowering>,
    source: &crate::snapshot::TruthSnapshotIdentity,
) -> crate::facade::BridgeConditionalEvaluationSession {
    let signal_basis = owner
        .admit_conditional_signal_basis(lowering, owner.admitted_signal_basis())
        .unwrap();
    owner
        .admit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                &signal_basis,
                source,
            ),
        )
        .unwrap()
}

#[test]
fn owned_correspondence_delivery_and_execution_share_the_sealed_signal_root() {
    let (owner, lowering) = install(always_eligible_contract("query:one"), "bridge-main");
    let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let pinned = admit_source_session(&owner, &lowering, &source);
    let first = execute_session(&owner, &lowering, &pinned, 1);
    assert_eq!(
        first.signal().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    let signal_basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let TransitionOutcome::Success(receipt) = owner
        .deliver_owned_authoritative_change(&signal_basis, 0)
        .expect("the retained owned dependency admits publication")
    else {
        panic!("owned correspondence delivery must complete through Signal owner services")
    };
    let prepared = receipt
        .prepared_signal_invalidation()
        .expect("an admitted semantic change prepares exact Signal invalidation");
    assert_eq!(
        prepared.graph_instance_id(),
        lowering.signal_graph_instance_id()
    );
    assert_eq!(prepared.target_count(), 1);
    assert_eq!(receipt.signal_capability_admissions(), 1);
    assert_eq!(receipt.signal_seeds_emitted(), 1);
    assert_eq!(receipt.node_fan_out(), 1);
    assert_eq!(receipt.slots_touched(), 1);

    let old_basis = execute_session(&owner, &lowering, &pinned, 2);
    assert_eq!(
        old_basis.signal().class(),
        SignalConditionalDecisionClass::DependencyUnchanged,
        "the retained session remains pinned to its admitted execution basis"
    );
    let fresh = owner
        .readmit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationReadmissionRequest {
                predecessor: &pinned,
                transitions: &[&receipt],
            },
        )
        .unwrap();
    let reuse = fresh.readmission_counters().unwrap();
    assert_eq!(reuse.transitions_checked(), 1);
    assert_eq!(reuse.targets_applied(), 1);
    assert_eq!(reuse.persistent_roots_forked(), 1);
    assert_eq!(reuse.source_opens(), 0);
    assert_eq!(reuse.unrelated_lowering_scans(), 0);
    let second = execute_session(&owner, &lowering, &fresh, 3);
    assert_ne!(
        second.signal().class(),
        SignalConditionalDecisionClass::DependencyUnchanged
    );
    assert_eq!(second.signal().counters().compute_contacts, 1);
}

#[test]
fn empty_conditional_root_seals_without_inventing_a_lowering() {
    let builder = super::owned_runtime_builder(runtime(exact_mapping(), Vec::new())).unwrap();
    let owner = builder.seal().unwrap();
    assert_eq!(owner.installed_conditional_definition_count(), 0);
    assert_eq!(owner.owned_signal_active_node_count().unwrap(), 0);
}

#[test]
fn successor_admission_recomputes_affected_lowering_and_reuses_unaffected_lowering() {
    let mut owner = super::owned_runtime_builder(runtime(exact_mapping(), Vec::new())).unwrap();
    let install = |owner: &mut BridgeConditionalRuntimeBuilder, identity: &'static str, output| {
        owner
            .install_owned_conditional(crate::facade::BridgeOwnedConditionalInstallationRequest {
                contract: always_eligible_contract(identity),
                location: BridgeConditionalLocation::operation(identity),
                dependencies: vec![freshly_installed_dependency(identity)],
                providers: BridgeConditionalProviderSet::new().compute(Compute(output)),
            })
            .unwrap()
    };
    let affected = install(&mut owner, "query:affected", 7);
    let unaffected = install(&mut owner, "query:unaffected", 11);
    let owner = owner.seal().unwrap();
    let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let admit = |lowering: &Arc<BridgeInstalledConditionalLowering>| {
        admit_source_session(&owner, lowering, &source)
    };
    let affected_initial = admit(&affected);
    let unaffected_initial = admit(&unaffected);
    execute_session(&owner, &affected, &affected_initial, 1);
    execute_session(&owner, &unaffected, &unaffected_initial, 1);
    let signal_basis = owner
        .admit_conditional_signal_basis(&affected, owner.admitted_signal_basis())
        .unwrap();
    let TransitionOutcome::Success(receipt) = owner
        .deliver_owned_authoritative_change(&signal_basis, 0)
        .unwrap()
    else {
        panic!("affected change publishes a successor basis")
    };
    let affected_successor = owner
        .readmit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationReadmissionRequest {
                predecessor: &affected_initial,
                transitions: &[&receipt],
            },
        )
        .unwrap();
    let unaffected_successor = owner
        .readmit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationReadmissionRequest {
                predecessor: &unaffected_initial,
                transitions: &[&receipt],
            },
        )
        .unwrap();
    let recomputed = execute_session(&owner, &affected, &affected_successor, 2);
    let reused = execute_session(&owner, &unaffected, &unaffected_successor, 2);

    assert_eq!(recomputed.signal().counters().compute_contacts, 1);
    assert_eq!(
        reused.signal().class(),
        SignalConditionalDecisionClass::DependencyUnchanged
    );
    assert_eq!(reused.signal().counters().compute_contacts, 0);
}

#[test]
fn successor_readmission_requires_the_exact_ordered_chain_and_allows_fresh_fallback() {
    let (owner, lowering) = install(always_eligible_contract("query:one"), "bridge-main");
    let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let a = admit_source_session(&owner, &lowering, &source);
    execute_session(&owner, &lowering, &a, 1);
    let signal_basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let TransitionOutcome::Success(ab) = owner
        .deliver_owned_authoritative_change(&signal_basis, 0)
        .unwrap()
    else {
        panic!("A to B delivery must publish")
    };
    let TransitionOutcome::Success(bc) = owner
        .deliver_owned_authoritative_change(&signal_basis, 0)
        .unwrap()
    else {
        panic!("B to C delivery must publish")
    };

    let incomplete = owner
        .readmit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationReadmissionRequest {
                predecessor: &a,
                transitions: &[&ab],
            },
        )
        .unwrap_err();
    assert_eq!(
        incomplete.kind(),
        crate::facade::BridgeConditionalDenialKind::ConditionalTransitionChainIncomplete
    );
    for invalid in [&[&bc, &ab][..], &[&ab, &ab][..], &[&bc][..]] {
        let denial = owner
            .readmit_conditional_evaluation(
                crate::facade::BridgeConditionalEvaluationReadmissionRequest {
                    predecessor: &a,
                    transitions: invalid,
                },
            )
            .unwrap_err();
        assert_eq!(
            denial.kind(),
            crate::facade::BridgeConditionalDenialKind::ConditionalTransitionChainMismatch
        );
    }

    let c = owner
        .readmit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationReadmissionRequest {
                predecessor: &a,
                transitions: &[&ab, &bc],
            },
        )
        .unwrap();
    let counters = c.readmission_counters().unwrap();
    assert_eq!(counters.transitions_checked(), 2);
    assert_eq!(counters.targets_applied(), 2);
    assert_eq!(counters.source_opens(), 0);
    assert_eq!(counters.unrelated_lowering_scans(), 0);
    assert_eq!(
        execute_session(&owner, &lowering, &c, 2)
            .signal()
            .counters()
            .compute_contacts,
        1
    );
    assert_eq!(
        execute_session(&owner, &lowering, &a, 3).signal().class(),
        SignalConditionalDecisionClass::DependencyUnchanged,
        "the old A admission remains executable after later deliveries"
    );

    let fresh = admit_source_session(&owner, &lowering, &source);
    assert_eq!(
        execute_session(&owner, &lowering, &fresh, 4)
            .bridge_execution_counters()
            .snapshot_admission_attempts,
        1,
        "explicit fresh fallback reopens its source exactly once"
    );

    let (foreign, foreign_lowering) = install(always_eligible_contract("query:one"), "foreign");
    let foreign_basis = foreign
        .admit_conditional_signal_basis(&foreign_lowering, foreign.admitted_signal_basis())
        .unwrap();
    let TransitionOutcome::Success(foreign_receipt) = foreign
        .deliver_owned_authoritative_change(&foreign_basis, 0)
        .unwrap()
    else {
        panic!("foreign fixture delivery must publish")
    };
    let foreign_denial = owner
        .readmit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationReadmissionRequest {
                predecessor: &a,
                transitions: &[&foreign_receipt],
            },
        )
        .unwrap_err();
    assert_eq!(
        foreign_denial.kind(),
        crate::facade::BridgeConditionalDenialKind::ConditionalTransitionChainMismatch
    );
}
