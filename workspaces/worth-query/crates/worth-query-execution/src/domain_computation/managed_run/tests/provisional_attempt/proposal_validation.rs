use std::sync::Arc;

use super::super::provisional_attempt_fixture::*;
use super::state;
use crate::domain_computation::{
    WorthQueryProvisionalDenialKind, WorthQueryProvisionalEffectAction,
    WorthQueryProvisionalProposalBasisParts,
};

#[test]
fn proposal_dimensions_and_symbol_order_are_checked_before_provider_staging() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let state = state();
        let (mut running, graph) = provisional_run(Arc::clone(&state), resource_request);
        let (staged, fresh) = staged_with_fresh_read_set(execution, &mut running, &graph);
        let basis = proposal_parts(1);
        let effect_authority = staged.effect_authority();
        let first = effect_authority
            .admit_proposal_basis(&fresh, basis.clone())
            .unwrap();
        for changed in changed_proposal_dimensions(&basis) {
            match effect_authority.admit_proposal_basis(&fresh, changed) {
                Ok(changed) => assert_ne!(first, changed),
                Err(failure) => assert_eq!(
                    failure.kind(),
                    WorthQueryProvisionalDenialKind::ProposalBasisMismatch
                ),
            }
        }

        let invalid_symbol = effect_step(WorthQueryProvisionalEffectAction::DeriveView {
            view_identity: "view".into(),
        })
        .with_symbolic_dependencies(["not-created"])
        .unwrap();
        let failure = staged
            .effect_authority()
            .lower_provisional_program(&fresh, [invalid_symbol])
            .err()
            .expect("unknown symbolic reference must deny");
        assert_eq!(
            failure.kind(),
            WorthQueryProvisionalDenialKind::UnknownSymbolicReference
        );
        let undeclared_artifact = effect_step(WorthQueryProvisionalEffectAction::Replace {
            target_identity: "base".into(),
        })
        .with_artifact_dependencies(["caller-authored-artifact"])
        .unwrap();
        let failure = staged
            .effect_authority()
            .lower_provisional_program(&fresh, [undeclared_artifact])
            .err()
            .expect("undeclared artifact dependency must deny");
        assert_eq!(
            failure.kind(),
            WorthQueryProvisionalDenialKind::UndeclaredArtifactDependency
        );

        let wrong_generation = staged
            .effect_authority()
            .admit_proposal_basis(&fresh, proposal_parts(2))
            .unwrap();
        let program = staged
            .effect_authority()
            .lower_provisional_program(
                &fresh,
                [effect_step(WorthQueryProvisionalEffectAction::Replace {
                    target_identity: "base".into(),
                })
                .with_proposal_basis(wrong_generation)],
            )
            .unwrap();
        let failure = match staged.begin_provisional_attempt(fresh, program) {
            Ok(_) => panic!("wrong proposal generation must deny before overlay staging"),
            Err(failure) => failure,
        };
        assert_eq!(
            failure.kind(),
            WorthQueryProvisionalDenialKind::ProposalBasisMismatch
        );
        assert_eq!(state.lock().unwrap().stage_calls, 0);
        cleanup(running);
    });
}

#[test]
fn proposal_from_a_peer_session_cannot_be_repaired_by_equal_rendered_basis() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let first_state = state();
        let (mut first_run, first_graph) =
            provisional_run(Arc::clone(&first_state), resource_request);
        let (first_staged, first_fresh) =
            staged_with_fresh_read_set(execution, &mut first_run, &first_graph);
        let proposal = first_staged
            .effect_authority()
            .admit_proposal_basis(&first_fresh, proposal_parts(1))
            .expect("the originating session admits its proposal");

        let second_state = state();
        let (mut second_run, second_graph) =
            provisional_run(Arc::clone(&second_state), resource_request);
        let (second_staged, second_fresh) =
            staged_with_fresh_read_set(execution, &mut second_run, &second_graph);
        assert_eq!(
            first_staged.plan().basis_identity(),
            second_staged.plan().basis_identity()
        );
        let failure = second_staged
            .effect_authority()
            .lower_provisional_program(
                &second_fresh,
                [effect_step(WorthQueryProvisionalEffectAction::Replace {
                    target_identity: "base".into(),
                })
                .with_proposal_basis(proposal)],
            )
            .err()
            .expect("a peer terminal binding cannot adopt the proposal");
        assert_eq!(
            failure.kind(),
            WorthQueryProvisionalDenialKind::ProposalBasisMismatch
        );
        let _ = first_staged.abort();
        let _ = second_staged.abort();
        cleanup(first_run);
        cleanup(second_run);
    });
}

fn proposal_parts(target_generation: u64) -> WorthQueryProvisionalProposalBasisParts {
    WorthQueryProvisionalProposalBasisParts {
        source_occurrence: "source-1".to_owned(),
        search_occurrence: "search-1".to_owned(),
        candidate_identity: "candidate-a".to_owned(),
        transformation_evidence: "transform-1".to_owned(),
        target_generation,
        installed_policy_identity: "policy-1".to_owned(),
        correspondence_identity: "correspondence-1".to_owned(),
        identity_consequence_identity: "identity-map-1".to_owned(),
    }
}

fn changed_proposal_dimensions(
    basis: &WorthQueryProvisionalProposalBasisParts,
) -> Vec<WorthQueryProvisionalProposalBasisParts> {
    let mut changes = Vec::new();
    macro_rules! changed {
        ($field:ident, $value:expr) => {{
            let mut value = basis.clone();
            value.$field = $value;
            changes.push(value);
        }};
    }
    changed!(source_occurrence, "source-2".to_owned());
    changed!(search_occurrence, "search-2".to_owned());
    changed!(candidate_identity, "candidate-b".to_owned());
    changed!(transformation_evidence, "transform-2".to_owned());
    changed!(target_generation, 2);
    changed!(installed_policy_identity, "policy-2".to_owned());
    changed!(correspondence_identity, "correspondence-2".to_owned());
    changed!(identity_consequence_identity, "identity-map-2".to_owned());
    changes
}
