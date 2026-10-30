#[allow(unused_imports)]
use crate::relational_certification::ordinary_context::invariant_proposed_state::*;

pub(crate) use worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_custom_invariant;
pub(crate) use {worth_relational::facade::history::BranchId};

#[test]
fn proposed_candidate_receipt_identifies_rule_that_read_proposed_and_committed_state() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court())
        .expect("Court Supply Chain definition is valid");
    let program = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(definition)
        .expect("Court Supply Chain program compiles");
    let evidence = crate::relational_certification::ordinary_context::invariant_proposed_state::ProbeEvidence::default();
    let registration = worth_relational::facade::runtime::CustomInvariantRegistration::new(
        ProposedAspectStateProbe {
            evidence: evidence.clone(),
            verdict: worth_relational::facade::runtime::CustomInvariantVerdict::Pass,
        },
    )
    .expect("proposed-state probe registers");
    let world =
        compile_supply_chain_baseline_with_custom_invariant(execution, program, registration)
            .expect("baseline commits with the inactive proposed-state probe");

    let target = world.handles.aurora_voyage().id;
    assert_snapshot_status(
        execution,
        &world.runtime,
        &BranchId("main".to_owned()),
        target,
        "Planned",
    );
    evidence.set_target(target);
    let branch_identity = world
        .runtime
        .branch_identity(&BranchId("main".to_owned()))
        .expect("the committed branch has owner-issued identity");
    let original_version = world
        .runtime
        .admit_branch_basis(&branch_identity)
        .expect("the original committed basis is admissible")
        .observation()
        .version_id();
    let proposal = validate_status_update(
        execution,
        &world.runtime,
        BranchId("main".to_owned()),
        target,
    )
    .expect("custom rule sees the proposed and committed status in both phases");

    assert_eq!(evidence.prepared(), 1);
    assert_eq!(evidence.evaluated(), 1);
    let receipt = proposal
        .custom_invariant_execution_receipts()
        .iter()
        .find(|receipt| receipt.rule_id().as_str() == RULE_ID)
        .expect("candidate retains the exact custom execution receipt");
    assert_eq!(
        receipt.semantic_version(),
        worth_relational::facade::runtime::CustomInvariantSemanticVersion::new(1, 0)
    );
    assert_eq!(
        receipt.execution_point(),
        worth_relational::facade::runtime::InvariantExecutionPoint::CommitBoundary
    );
    assert_eq!(
        receipt.verdict(),
        worth_relational::facade::runtime::InvariantDecisionKind::Passed
    );
    assert_eq!(
        receipt.provenance().version_id,
        proposal.invariant_evidence().proposed_version()
    );
    assert_eq!(receipt.provenance().current_version_id, original_version);
    assert_ne!(receipt.provenance().version_id, original_version);
    assert_eq!(
        receipt.provenance().proposal_identity.as_ref(),
        Some(proposal.proposal_identity())
    );
    assert!(receipt
        .provenance()
        .touched
        .visible_entity_ids
        .contains(&target));
    assert!(receipt.provenance().traversal.remaining_frontier > 0);
    assert!(receipt.provenance().traversal.remaining_steps > 0);
    assert_eq!(receipt.provenance().traversal.max_depth, 32);
}

#[test]
fn malformed_proposal_denial_preserves_custom_rule_identity() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let definition = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition::operating(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court())
        .expect("Court Supply Chain definition is valid");
    let program = worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram::compile(definition)
        .expect("Court Supply Chain program compiles");
    let evidence = crate::relational_certification::ordinary_context::invariant_proposed_state::ProbeEvidence::default();
    let registration = worth_relational::facade::runtime::CustomInvariantRegistration::new(
        ProposedAspectStateProbe {
            evidence: evidence.clone(),
            verdict: worth_relational::facade::runtime::CustomInvariantVerdict::Violation(
                worth_relational::facade::runtime::CustomInvariantRefusal::Empty,
            ),
        },
    )
    .expect("proposed-state denial rule registers");
    let world =
        compile_supply_chain_baseline_with_custom_invariant(execution, program, registration)
            .expect("baseline commits while the denial rule is inactive");

    let target = world.handles.aurora_voyage().id;
    evidence.set_target(target);
    let denial = validate_status_update(
        execution,
        &world.runtime,
        BranchId("main".to_owned()),
        target,
    )
    .expect_err("the malformed proposed status must be denied");

    let worth_relational::facade::transactions::TransactionCommitError::Conflict { error, .. } =
        denial
    else {
        panic!("custom violation must deny as a typed conflict");
    };
    let worth_relational::facade::transactions::ConflictClass::InvariantViolation {
        fields:
            worth_relational::facade::transactions::InvariantViolationFields::CustomInvariantViolation {
                identity,
                refusal,
            },
        ..
    } = error.class
    else {
        panic!("custom violation must preserve its semantic identity");
    };
    assert_eq!(identity.rule_id.as_str(), RULE_ID);
    assert_eq!(
        refusal,
        worth_relational::facade::runtime::CustomInvariantRefusal::Empty
    );
    assert_eq!(
        identity.semantic_version,
        worth_relational::facade::runtime::CustomInvariantSemanticVersion::new(1, 0)
    );
}
