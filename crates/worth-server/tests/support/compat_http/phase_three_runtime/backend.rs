use super::*;

impl worth_query::facade::runtime::WorthQuerySettlementRecoveryBackend
    for StatefulCountingMutationRuntimeBackend
{
}

impl worth_query::facade::runtime::WorthQueryMergeSnapshotOwner
    for StatefulCountingMutationRuntimeBackend
{
}

impl WorthQueryRuntimeBackend for StatefulCountingMutationRuntimeBackend {
    fn prepare_product_source(
        &self,
    ) -> Result<
        worth_query::facade::runtime::WorthQueryProductRelationalInstallation,
        worth_query::facade::runtime::WorthQueryProductSourceDenial,
    > {
        let branch = self
            .product_source
            .with_runtime(|runtime| runtime.main_branch_identity());
        self.product_source
            .prepare_product_source(&branch)
            .map_err(worth_query::facade::runtime::WorthQueryProductSourceDenial::Basis)
    }

    fn support_profile(&self) -> WorthQueryRuntimeSupportProfile {
        self.support_profile.clone()
    }

    fn admit_live_view_declaration(
        &self,
        _name: &str,
        _request: &worth_query::facade::foundation::DeclarativeLiveQueryRequest,
        _schema_view: &worth_query::facade::runtime::QuerySchemaView,
    ) -> Result<
        worth_query::facade::runtime::LiveViewDeclarationAdmissionBoundaryReceipt,
        WorthQueryWorkspaceError,
    > {
        panic!("phase three mutation runtime does not declare live views")
    }

    fn declare_live_view(
        &mut self,
        _name: String,
        _request: worth_query::facade::foundation::DeclarativeLiveQueryRequest,
        _schema_view: worth_query::facade::runtime::QuerySchemaView,
    ) -> Result<worth_query::facade::foundation::WorthQueryLiveViewHandle, WorthQueryWorkspaceError>
    {
        panic!("phase three mutation runtime does not declare live views")
    }

    fn close_live_view(&mut self, _name: &str) -> Result<(), WorthQueryWorkspaceError> {
        panic!("phase three mutation runtime does not close live views")
    }

    fn write(
        &mut self,
        command: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        self.attempted_writes.fetch_add(1, Ordering::Relaxed);
        Ok(test_mutation_receipt(
            &command,
            self.next_snapshot_ordinal(),
            WorthQueryMutationKind::Created,
        ))
    }

    fn write_batch(
        &mut self,
        commands: Vec<WorthQueryBackendAdmissibleMutation>,
    ) -> Result<Vec<WorthQueryMutationReceipt>, WorthQueryWorkspaceError> {
        self.attempted_writes
            .fetch_add(commands.len(), Ordering::Relaxed);
        Ok(commands
            .iter()
            .enumerate()
            .map(|(index, command)| {
                test_mutation_receipt(
                    command,
                    self.next_snapshot_ordinal() + index,
                    mutation_kind(command),
                )
            })
            .collect())
    }

    fn execute_intent(
        &mut self,
        _declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryRuntimeError> {
        panic!("phase three mutation runtime does not execute generic intents")
    }

    fn live_entities_for_target(
        &self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryEntity> {
        Vec::new()
    }

    fn drain_live_patches_for_target(
        &mut self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch> {
        Vec::new()
    }

    fn affected_live_view_targets(
        &self,
        _receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget> {
        Vec::new()
    }

    fn current_snapshot_identity(&self) -> WorthQuerySnapshotIdentity {
        WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(
            worth_runtime_bridge::facade::TruthSnapshotIdentity::from_relational_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(
                    self.snapshot_version.load(Ordering::Relaxed) as u64,
                    1,
                ),
            ),
        )
        .expect("relational snapshot projection must retain its typed payload")
    }

    fn install_live_subscription(
        &mut self,
        _view_name: &str,
        _activation: &SubscriptionActivationInput,
    ) -> Result<SubscriptionActivationReceipt, WorthQueryWorkspaceError> {
        panic!("phase three mutation runtime does not install subscriptions")
    }

    fn admit_preview_basis(
        &self,
        _label: &WorthQuerySessionLabel,
        _effect_policy: worth_query::facade::runtime::WorthQueryEffectPolicy,
        _authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryPreviewBasisAdmission, WorthQueryWorkspaceError> {
        panic!("phase three mutation runtime does not admit preview basis")
    }

    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError> {
        Ok(WorthQueryRuntimeInspectionEvidence::new(
            authority,
            "phase-three-mutation-inspection",
            receipt.authority_lane(),
            ["phase-three-mutation-runtime"],
        ))
    }
}

fn test_mutation_receipt(
    command: &WorthQueryBackendAdmissibleMutation,
    ordinal: usize,
    kind: WorthQueryMutationKind,
) -> WorthQueryMutationReceipt {
    WorthQueryMutationReceipt::from_authoritative_parts(
        WorthQueryCommitIdentity::from_bridge_commit_projection(
            worth_runtime_bridge::facade::TruthCommitIdentity::from_relational_commit_id(
                ordinal as u64,
            ),
        ),
        WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(
            worth_runtime_bridge::facade::TruthSnapshotIdentity::from_relational_snapshot(
                RelationalBridgeSnapshotIdentityParts::new(ordinal as u64, 1),
            ),
        )
        .expect("relational snapshot projection must retain its typed payload"),
        vec![WorthQueryMutationDelta::from_touched_aspects(
            command
                .declared_collection_identity()
                .map(|collection| collection.as_str().to_string())
                .unwrap_or_else(|| "Task".to_string()),
            command.declared_entity_identity().unwrap_or_else(|| {
                WorthQueryEntityIdentity::from_bridge_record_projection(
                    RelationalBridgeRecordIdentityParts::entity(1, ordinal as u64, 0),
                )
            }),
            kind,
            command.declared_aspect_touches(),
        )],
    )
}

fn mutation_kind(command: &WorthQueryBackendAdmissibleMutation) -> WorthQueryMutationKind {
    match command.mutation_family() {
        worth_query::facade::runtime::WorthQueryMutationFamily::Insert => {
            WorthQueryMutationKind::Created
        }
        worth_query::facade::runtime::WorthQueryMutationFamily::Delete => {
            WorthQueryMutationKind::Deleted
        }
        _ => WorthQueryMutationKind::Updated,
    }
}
