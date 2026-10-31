use super::*;
use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryMutationDelta, WorthQuerySnapshotIdentity,
};

pub(in crate::runtime::tests) struct TestIntentAuthority {
    next_version: u64,
}

#[allow(non_upper_case_globals)]
pub(in crate::runtime::tests) const TestIntentAuthority: TestIntentAuthority =
    TestIntentAuthority { next_version: 2 };

impl WorthQueryIntentAuthorityAdapter for TestIntentAuthority {
    fn execute_intent(
        &mut self,
        bridge: &RuntimeBridge,
        _relational_runtime: Option<&RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError> {
        let version = self.next_version;
        self.next_version = self.next_version.saturating_add(1);
        let mutation_receipt = bridge_authoritative_intent_receipt(bridge, version, true)?;
        Ok(WorthQueryIntentExecution::admitted(
            declaration.strategy_name(),
            declaration.strategy_version(),
            "test-strategy-descriptor-digest",
            declaration.input_digest(),
            crate::identity::hash_parts(&[
                "test-intent-produced-mutation".to_string(),
                mutation_receipt
                    .commit_identity
                    .evidence_identity()
                    .as_str()
                    .to_string(),
                mutation_receipt
                    .snapshot_identity
                    .evidence_identity()
                    .as_str()
                    .to_string(),
            ]),
            ["test-invariant-authority"],
            mutation_receipt,
        ))
    }
}

pub(in crate::runtime::tests) struct CountingIntentAuthority {
    pub(in crate::runtime::tests) attempted: std::rc::Rc<std::cell::Cell<usize>>,
}

impl WorthQueryIntentAuthorityAdapter for CountingIntentAuthority {
    fn execute_intent(
        &mut self,
        _bridge: &RuntimeBridge,
        _relational_runtime: Option<&RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError> {
        self.attempted.set(self.attempted.get().saturating_add(1));
        let mut authority = TestIntentAuthority;
        authority.execute_intent(_bridge, _relational_runtime, declaration)
    }
}

pub(in crate::runtime::tests) struct NoopIntentAuthority;

impl WorthQueryIntentAuthorityAdapter for NoopIntentAuthority {
    fn execute_intent(
        &mut self,
        _bridge: &RuntimeBridge,
        _relational_runtime: Option<&RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError> {
        Ok(WorthQueryIntentExecution::idempotent_noop(
            declaration.strategy_name(),
            declaration.strategy_version(),
            "test-noop-strategy-descriptor-digest",
            declaration.input_digest(),
            crate::identity::hash_parts(&[
                "test-intent-idempotent-noop".to_string(),
                declaration.input_digest().to_string(),
            ]),
            ["test-invariant-authority", "idempotent-noop"],
            crate::memory_workspace::admit_external_commit_label("external-intent-noop-commit-1"),
            crate::memory_workspace::admit_external_snapshot_label(
                "external-intent-noop-snapshot-1",
            ),
        ))
    }
}

pub(in crate::runtime::tests) struct EmptyMutatingIntentAuthority;

impl WorthQueryIntentAuthorityAdapter for EmptyMutatingIntentAuthority {
    fn execute_intent(
        &mut self,
        bridge: &RuntimeBridge,
        _relational_runtime: Option<&RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError> {
        let mutation_receipt = bridge_authoritative_intent_receipt(bridge, 2, false)?;
        Ok(WorthQueryIntentExecution::admitted(
            declaration.strategy_name(),
            declaration.strategy_version(),
            "test-empty-mutating-strategy-descriptor-digest",
            declaration.input_digest(),
            crate::identity::hash_parts(&[
                "test-intent-empty-mutating".to_string(),
                mutation_receipt
                    .commit_identity
                    .evidence_identity()
                    .as_str()
                    .to_string(),
                mutation_receipt
                    .snapshot_identity
                    .evidence_identity()
                    .as_str()
                    .to_string(),
            ]),
            ["test-invariant-authority"],
            mutation_receipt,
        ))
    }
}

pub(in crate::runtime::tests) struct InvariantViolationIntentAuthority;

impl WorthQueryIntentAuthorityAdapter for InvariantViolationIntentAuthority {
    fn execute_intent(
        &mut self,
        _bridge: &RuntimeBridge,
        _relational_runtime: Option<&RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError> {
        Ok(WorthQueryIntentExecution::invariant_violation(
            declaration.strategy_name(),
            declaration.strategy_version(),
            "test-invariant-violation-strategy-descriptor-digest",
            declaration.input_digest(),
            crate::identity::hash_parts(&[
                "test-intent-invariant-violation".to_string(),
                declaration.input_digest().to_string(),
            ]),
            [
                "relational-invariant:constraint-a:false",
                "relational-invariant:constraint-b:false",
            ],
            crate::memory_workspace::admit_external_snapshot_label(
                "external-intent-invariant-denial-snapshot-1",
            ),
        ))
    }
}

pub(in crate::runtime::tests) struct DriftingIntentAuthority;

impl WorthQueryIntentAuthorityAdapter for DriftingIntentAuthority {
    fn execute_intent(
        &mut self,
        bridge: &RuntimeBridge,
        _relational_runtime: Option<&RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError> {
        let mutation_receipt = bridge_authoritative_intent_receipt(bridge, 2, true)?;
        Ok(WorthQueryIntentExecution::admitted(
            "strategy.intent.other",
            declaration.strategy_version(),
            "test-strategy-descriptor-digest",
            declaration.input_digest(),
            crate::identity::hash_parts(&[
                "test-intent-produced-mutation".to_string(),
                mutation_receipt
                    .commit_identity
                    .evidence_identity()
                    .as_str()
                    .to_string(),
                mutation_receipt
                    .snapshot_identity
                    .evidence_identity()
                    .as_str()
                    .to_string(),
            ]),
            ["test-invariant-authority"],
            mutation_receipt,
        ))
    }
}

pub(in crate::runtime::tests) struct AuthoritylessIntentAuthority;

impl WorthQueryIntentAuthorityAdapter for AuthoritylessIntentAuthority {
    fn execute_intent(
        &mut self,
        _bridge: &RuntimeBridge,
        _relational_runtime: Option<&RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError> {
        let mutation_receipt = test_mutation_receipt(
            WorthQueryCommitIdentity::from_relational_commit_id(1),
            WorthQuerySnapshotIdentity::from_relational_snapshot(
                worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts::new(1, 1),
            ),
            "Task",
            WorthQueryEntityIdentity::from_relational_record(
                RelationalBridgeRecordIdentityParts::entity(1, 1, 0),
            ),
            WorthQueryMutationKind::Updated,
            test_aspect_touches(["title.value"]).to_vec(),
        );
        Ok(WorthQueryIntentExecution::admitted(
            declaration.strategy_name(),
            declaration.strategy_version(),
            "authorityless-strategy-descriptor-digest",
            declaration.input_digest(),
            "authorityless-intent-outcome-digest",
            ["claimed-invariant-authority"],
            mutation_receipt,
        ))
    }
}

fn bridge_authoritative_intent_receipt(
    bridge: &RuntimeBridge,
    version: u64,
    include_delta: bool,
) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
    let entity_identity = WorthQueryEntityIdentity::from_relational_record(
        RelationalBridgeRecordIdentityParts::entity(1, 1, 0),
    );
    let snapshot_identity = WorthQuerySnapshotIdentity::from_relational_snapshot(
        worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts::new(1, version),
    );
    let touch = test_aspect_touch("title.value");
    let command = WorthQueryWriteCommand::UpdateAspects {
        entity_identity: entity_identity.clone(),
        aspects: vec![WorthQueryAuthoredAspectMutation::new_set(
            touch.clone(),
            WorthQueryAuthoredAspectMutation::native_string_value("Intent committed title"),
        )
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?],
        metadata: Default::default(),
        naming_intent: None,
        continuity_intent: None,
    };
    let contracts = crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry::from_contracts(
        stateful_bridge_aspect_contracts(),
    )
    .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
    let mutation = WorthQueryBackendAdmissibleMutation::from_authored_command(command, &contracts)
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
    let bridge_authority = crate::runtime::backend::build_bridge_authority_bundle(
        bridge,
        &snapshot_identity,
        &mutation,
        crate::runtime::WorthQueryBridgeMutationTarget::new(
            "Task",
            &entity_identity,
            WorthQueryMutationKind::Updated,
        ),
    )?;
    let deltas = include_delta
        .then(|| {
            WorthQueryMutationDelta::from_touched_aspects(
                "Task",
                entity_identity,
                WorthQueryMutationKind::Updated,
                vec![touch],
            )
        })
        .into_iter()
        .collect();
    Ok(WorthQueryMutationReceipt::from_bridge_authoritative_parts(
        WorthQueryCommitIdentity::from_relational_commit_id(version),
        snapshot_identity,
        deltas,
        bridge_authority,
    ))
}
