pub(crate) use std::collections::BTreeMap;
pub(crate) use {std::sync::Arc, std::sync::Mutex};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_custom_invariant, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::AuthoritativeRecordAspectState, worth_foundational::facade::ContractValidatedAspectValueView, worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::EntityId};
pub(crate) use {worth_relational::facade::runtime::CustomInvariantDescriptor, worth_relational::facade::runtime::CustomInvariantExecutionContext, worth_relational::facade::runtime::CustomInvariantExecutionError, worth_relational::facade::runtime::CustomInvariantOperationalMetadata, worth_relational::facade::runtime::CustomInvariantPreparationError, worth_relational::facade::runtime::CustomInvariantRefusal, worth_relational::facade::runtime::CustomInvariantRegistration, worth_relational::facade::runtime::CustomInvariantRule, worth_relational::facade::runtime::CustomInvariantScopePlanner, worth_relational::facade::runtime::CustomInvariantSemanticIdentity, worth_relational::facade::runtime::CustomInvariantSemanticVersion, worth_relational::facade::runtime::CustomInvariantVerdict, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantDecisionKind, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantFailureEffect, worth_relational::facade::runtime::InvariantGroupSet};
pub(crate) use {worth_relational::facade::transactions::planned_single_field_locator, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::EntityMutationIntent, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::UpdateEntityFieldsIntent, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) const RULE_ID: &str = "phase5.custom-proposed-aspect";

#[derive(Clone, Default)]
pub(crate) struct ProbeEvidence {
    pub(crate) target: Arc<Mutex<Option<EntityId>>>,
    pub(crate) prepared: Arc<Mutex<usize>>,
    pub(crate) evaluated: Arc<Mutex<usize>>,
}

impl ProbeEvidence {
    pub(crate) fn set_target(&self, target: EntityId) {
        *self.target.lock().expect("target lock") = Some(target);
    }

    pub(crate) fn target(&self) -> Option<EntityId> {
        *self.target.lock().expect("target lock")
    }

    pub(crate) fn prepared(&self) -> usize {
        *self.prepared.lock().expect("prepared lock")
    }

    pub(crate) fn evaluated(&self) -> usize {
        *self.evaluated.lock().expect("evaluated lock")
    }
}

#[derive(Clone)]
pub(crate) struct ProposedAspectStateProbe {
    pub(crate) evidence: ProbeEvidence,
    pub(crate) verdict: CustomInvariantVerdict,
}

impl CustomInvariantRule for ProposedAspectStateProbe {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: worth_relational::facade::runtime::CustomInvariantRuleId::new(RULE_ID),
                semantic_version: worth_relational::facade::runtime::CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: std::sync::Arc::from("Phase 5 proposed aspect state probe"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(std::num::NonZeroU64::new(1_000_000).unwrap()),
                access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                    read_entity_kinds: vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id(
                        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage,
                    )],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id(
                        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage,
                    )],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: true,
                }
                .canonicalize(),
                execution_point: worth_relational::facade::runtime::InvariantExecutionPoint::CommitBoundary,
                groups: worth_relational::facade::runtime::InvariantGroupSet::all(),
                cost_class: worth_relational::facade::runtime::InvariantCostClass::Touched,
                failure_effect: worth_relational::facade::runtime::InvariantFailureEffect::BlockCommit,
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
            return Ok(self.verdict.clone());
        }
        Ok(worth_relational::facade::runtime::CustomInvariantVerdict::Pass)
    }
}

pub(crate) fn assert_proposed_status(
    state: Result<
        &AuthoritativeRecordAspectState,
        worth_relational::facade::runtime::StructuralReadError,
    >,
) {
    assert_status(state, "Held");
}

pub(crate) fn assert_status(
    state: Result<
        &AuthoritativeRecordAspectState,
        worth_relational::facade::runtime::StructuralReadError,
    >,
    expected: &str,
) {
    let state = state.expect("custom view resolves the existing target");
    let aspect = worth_foundational::facade::AspectKey::new("status").expect("status aspect");
    let value = state.get(&aspect).expect("status aspect exists").view();
    let worth_foundational::facade::ContractValidatedAspectValueView::Scalar(
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw(observed),
        ),
    ) = value
    else {
        panic!("status aspect must be a scalar string");
    };
    assert_eq!(observed, expected);
}

pub(crate) fn assert_snapshot_status(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        .read_snapshot(&snapshot, execution)
        .expect("fixture read request remains live")
        .expect("branch snapshot is readable");
    let record = view
        .entities()
        .iter()
        .find(|record| record.entity_id == entity_id)
        .expect("target remains in the branch snapshot");
    assert_status(
        record
            .authoritative_aspect_state
            .as_ref()
            .ok_or(worth_relational::facade::runtime::StructuralReadError::RecordUnavailable),
        expected,
    );
}

pub(crate) fn validate_status_update(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        worth_foundational::facade::AspectKey::new("status").expect("status aspect"),
        worth_foundational::facade::FieldKey::new("status").expect("status field"),
    );
    let fields = worth_relational::facade::transactions::AspectFieldPatch::new(
        std::collections::BTreeMap::from([(
            locator,
            worth_foundational::facade::AspectValue::String(
                worth_foundational::facade::InternedString::Raw("Held".to_owned()),
            ),
        )]),
    );
    let mut transaction = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(
            worth_relational::facade::transactions::WorkerIntentBatch::new(
                "phase5-proposed-status",
            )
            .push(
                worth_relational::facade::transactions::MutationIntent::Entity(
                    worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(
                        UpdateEntityFieldsIntent { entity_id, fields },
                    ),
                ),
            ),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.validate(execution, runtime)
}
