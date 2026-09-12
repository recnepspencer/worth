use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use super::world::supply_chain::{
    compile_supply_chain_baseline_with_custom_invariant, snapshot_for_supply_chain_identity,
    CompiledSupplyChainProgram, SupplyChainScale, SupplyChainWorldDefinition,
};
use worth_foundational::facade::{
    AspectKey, AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
    FieldKey, InternedString,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantScopePlanner,
    CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion, CustomInvariantVerdict,
    InvariantCostClass, InvariantDecisionKind, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantGroupSet,
};
use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, EntityMutationIntent, MutationIntent,
    TransactionCommitError, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

const RULE_ID: &str = "phase5.custom-proposed-aspect";

#[derive(Clone, Default)]
struct ProbeEvidence {
    target: Arc<Mutex<Option<EntityId>>>,
    prepared: Arc<Mutex<usize>>,
    evaluated: Arc<Mutex<usize>>,
}

impl ProbeEvidence {
    fn set_target(&self, target: EntityId) {
        *self.target.lock().expect("target lock") = Some(target);
    }

    fn target(&self) -> Option<EntityId> {
        *self.target.lock().expect("target lock")
    }

    fn prepared(&self) -> usize {
        *self.prepared.lock().expect("prepared lock")
    }

    fn evaluated(&self) -> usize {
        *self.evaluated.lock().expect("evaluated lock")
    }
}

#[test]
fn proposed_candidate_receipt_identifies_rule_that_read_proposed_and_committed_state() {
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court())
        .expect("Court Supply Chain definition is valid");
    let program = CompiledSupplyChainProgram::compile(definition)
        .expect("Court Supply Chain program compiles");
    let evidence = ProbeEvidence::default();
    let registration = CustomInvariantRegistration::new(ProposedAspectStateProbe {
        evidence: evidence.clone(),
        verdict: CustomInvariantVerdict::Pass,
    })
    .expect("proposed-state probe registers");
    let world = compile_supply_chain_baseline_with_custom_invariant(program, registration)
        .expect("baseline commits with the inactive proposed-state probe");

    let target = world.handles.aurora_voyage().id;
    assert_snapshot_status(
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
    let proposal = validate_status_update(&world.runtime, BranchId("main".to_owned()), target)
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
        CustomInvariantSemanticVersion::new(1, 0)
    );
    assert_eq!(
        receipt.execution_point(),
        InvariantExecutionPoint::CommitBoundary
    );
    assert_eq!(receipt.verdict(), InvariantDecisionKind::Passed);
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
    let definition = SupplyChainWorldDefinition::operating(SupplyChainScale::court())
        .expect("Court Supply Chain definition is valid");
    let program = CompiledSupplyChainProgram::compile(definition)
        .expect("Court Supply Chain program compiles");
    let evidence = ProbeEvidence::default();
    let registration = CustomInvariantRegistration::new(ProposedAspectStateProbe {
        evidence: evidence.clone(),
        verdict: CustomInvariantVerdict::Violation,
    })
    .expect("proposed-state denial rule registers");
    let world = compile_supply_chain_baseline_with_custom_invariant(program, registration)
        .expect("baseline commits while the denial rule is inactive");

    let target = world.handles.aurora_voyage().id;
    evidence.set_target(target);
    let denial = validate_status_update(&world.runtime, BranchId("main".to_owned()), target)
        .expect_err("the malformed proposed status must be denied");

    let TransactionCommitError::Conflict { error, .. } = denial else {
        panic!("custom violation must deny as a typed conflict");
    };
    let worth_relational::facade::transactions::ConflictClass::InvariantViolation {
        fields:
            worth_relational::facade::transactions::InvariantViolationFields::CustomInvariantViolation {
                identity,
            },
        ..
    } = error.class
    else {
        panic!("custom violation must preserve its semantic identity");
    };
    assert_eq!(identity.rule_id.as_str(), RULE_ID);
    assert_eq!(
        identity.semantic_version,
        CustomInvariantSemanticVersion::new(1, 0)
    );
}

#[derive(Clone)]
struct ProposedAspectStateProbe {
    evidence: ProbeEvidence,
    verdict: CustomInvariantVerdict,
}

impl CustomInvariantRule for ProposedAspectStateProbe {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: worth_relational::facade::runtime::CustomInvariantRuleId::new(RULE_ID),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Phase 5 proposed aspect state probe"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(1_000_000).unwrap(),
                access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                    read_entity_kinds: vec![super::world::supply_chain::entity_kind_id(
                        super::world::supply_chain::EntityKind::Voyage,
                    )],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![super::world::supply_chain::entity_kind_id(
                        super::world::supply_chain::EntityKind::Voyage,
                    )],
                    affected_relation_kinds: vec![],
                }
                .canonicalize(),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::all(),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        if let Some(target) = self.evidence.target() {
            assert_status(
                planner
                    .committed_aspect_states()
                    .entity_aspect_state(target),
                "Planned",
            );
            assert_proposed_status(planner.aspect_states().entity_aspect_state(target));
            *self.evidence.prepared.lock().expect("prepared lock") += 1;
        }
        Ok(())
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        if let Some(target) = self.evidence.target() {
            assert_status(
                context
                    .committed_aspect_states()
                    .entity_aspect_state(target),
                "Planned",
            );
            assert_proposed_status(context.aspect_states().entity_aspect_state(target));
            let identity = context
                .provenance()
                .proposal_identity
                .expect("custom execution carries the owner-issued proposal identity");
            assert_eq!(
                identity.proposed_version_id(),
                context.version_id(),
                "custom context must identify the candidate it evaluates"
            );
            assert!(
                identity.proposed_version_id().0 > context.current_version_id().0,
                "the candidate version must follow its committed basis"
            );
            *self.evidence.evaluated.lock().expect("evaluated lock") += 1;
            return Ok(self.verdict);
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}

fn assert_proposed_status(state: Option<&AuthoritativeRecordAspectState>) {
    assert_status(state, "Held");
}

fn assert_status(state: Option<&AuthoritativeRecordAspectState>, expected: &str) {
    let state = state.expect("custom view resolves the existing target");
    let aspect = AspectKey::new("status").expect("status aspect");
    let value = state.get(&aspect).expect("status aspect exists").view();
    let ContractValidatedAspectValueView::Scalar(AspectValue::String(InternedString::Raw(
        observed,
    ))) = value
    else {
        panic!("status aspect must be a scalar string");
    };
    assert_eq!(observed, expected);
}

fn assert_snapshot_status(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    branch: &BranchId,
    entity_id: EntityId,
    expected: &str,
) {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    let snapshot = snapshot_for_supply_chain_identity(runtime, &identity);
    let view = runtime
        .read_truth()
        .read_snapshot(&snapshot)
        .expect("branch snapshot is readable");
    let record = view
        .entities()
        .iter()
        .find(|record| record.entity_id == entity_id)
        .expect("target remains in the branch snapshot");
    assert_status(record.authoritative_aspect_state.as_ref(), expected);
}

fn validate_status_update(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    branch: BranchId,
    entity_id: EntityId,
) -> Result<worth_relational::facade::mvcc::ValidatedRelationalProposal, TransactionCommitError> {
    let identity = runtime
        .branch_identity(&branch)
        .expect("branch identity is owner-issued");
    let options = runtime
        .admit_branch_basis(&identity)
        .expect("transaction authority is owner-issued");
    let locator = planned_single_field_locator(
        AspectKey::new("status").expect("status aspect"),
        FieldKey::new("status").expect("status field"),
    );
    let fields = AspectFieldPatch::new(BTreeMap::from([(
        locator,
        AspectValue::String(InternedString::Raw("Held".to_owned())),
    )]));
    let mut transaction = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(
            WorkerIntentBatch::new("phase5-proposed-status").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
            )),
        )
        .unwrap();
    transaction.validate(runtime)
}
