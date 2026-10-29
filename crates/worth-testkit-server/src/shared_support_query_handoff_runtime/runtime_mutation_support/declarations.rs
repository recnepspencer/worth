pub use {worth_query::facade::foundation::WorthQueryCommitIdentity, worth_query::facade::foundation::WorthQueryEntityIdentity, worth_query::facade::foundation::WorthQueryMutationDelta, worth_query::facade::foundation::WorthQueryMutationKind, worth_query::facade::foundation::WorthQueryMutationReceipt, worth_query::facade::foundation::WorthQuerySnapshotIdentity, worth_query::facade::foundation::WorthQueryWorkspaceError};
pub use {worth_query::facade::runtime::SubscriptionActivationInput, worth_query::facade::runtime::WorthQueryBackendAdmissibleMutation, worth_query::facade::runtime::WorthQueryEvidenceIdentity, worth_query::facade::runtime::WorthQueryRuntimeSubscriptionActivationAdapter};
pub use {worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts, worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts};

pub fn test_mutation_receipt(
    command: &WorthQueryBackendAdmissibleMutation,
    ordinal: usize,
) -> WorthQueryMutationReceipt {
    worth_query::facade::foundation::WorthQueryMutationReceipt::from_authoritative_parts(
        worth_query::facade::foundation::WorthQueryCommitIdentity::from_bridge_commit_projection(
            worth_runtime_bridge::facade::TruthCommitIdentity::from_relational_commit_id(
                ordinal as u64,
            ),
        ),
        worth_query::facade::foundation::WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(
            worth_runtime_bridge::facade::TruthSnapshotIdentity::from_relational_snapshot(
                worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts::new(ordinal as u64, 1),
            ),
        )
        .expect("relational snapshot projection must retain its typed payload"),
        vec![worth_query::facade::foundation::WorthQueryMutationDelta::from_touched_aspects(
            mutation_collection(command),
            mutation_entity_identity(command, ordinal),
            mutation_kind(command),
            command.declared_aspect_touches(),
        )],
    )
}

pub struct TestSubscriptionActivation;

impl WorthQueryRuntimeSubscriptionActivationAdapter for TestSubscriptionActivation {
    fn support_evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        worth_query::facade::runtime::runtime_subscription_support_evidence_identity(
            "worth-server-query-handoff-test-support",
        )
    }

    fn admit_activation(
        &mut self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<
        worth_query::facade::runtime::SubscriptionActivationBoundaryReceipt,
        WorthQueryWorkspaceError,
    > {
        let receipt = self.build_subscription_activation_receipt(view_name, activation);
        Ok(self.build_subscription_activation_boundary_receipt(view_name, activation, receipt))
    }
}

pub fn mutation_collection(command: &WorthQueryBackendAdmissibleMutation) -> String {
    command
        .declared_collection_identity()
        .map(|collection| collection.as_str().to_string())
        .unwrap_or_else(|| "Task".to_string())
}

pub fn mutation_entity_identity(
    command: &WorthQueryBackendAdmissibleMutation,
    ordinal: usize,
) -> WorthQueryEntityIdentity {
    command.declared_entity_identity().unwrap_or_else(|| {
        worth_query::facade::foundation::WorthQueryEntityIdentity::from_bridge_record_projection(
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, ordinal as u64, 0),
        )
    })
}

pub fn mutation_kind(command: &WorthQueryBackendAdmissibleMutation) -> WorthQueryMutationKind {
    match command.mutation_family() {
        worth_query::facade::runtime::WorthQueryMutationFamily::Insert => {
            worth_query::facade::foundation::WorthQueryMutationKind::Created
        }
        worth_query::facade::runtime::WorthQueryMutationFamily::Delete => {
            worth_query::facade::foundation::WorthQueryMutationKind::Deleted
        }
        _ => worth_query::facade::foundation::WorthQueryMutationKind::Updated,
    }
}