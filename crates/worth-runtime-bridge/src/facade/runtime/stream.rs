use super::*;

impl RuntimeBridge {
    /// Specialist validation entrypoint for change stream declarations.
    pub fn validate_change_stream_declaration(
        &self,
        declaration: ChangeStreamDeclaration,
    ) -> Result<ValidatedStreamProtocol, BridgeStreamError> {
        crate::stream::validate_change_stream_declaration(declaration)
    }

    /// Admits a validated stream protocol for one consumer shape.
    pub fn resolve_change_stream_consumer_contract(
        &self,
        protocol: &ValidatedStreamProtocol,
    ) -> Result<AdmittedConsumerContract, BridgeStreamError> {
        crate::stream::resolve_consumer_contract(protocol)
    }

    /// Plans and lowers one change stream window for a consumer contract.
    ///
    /// This is the main advanced stream-planning door; lower-level validation
    /// and replay helpers remain specialist.
    ///
    /// ```no_run
    /// use worth_runtime_bridge::facade::{
    ///     AdmittedConsumerContract, BridgeCommittedPatchEnvelope, RuntimeBridge,
    /// };
    ///
    /// fn plan_stream_window(
    ///     bridge: &RuntimeBridge,
    ///     contract: &AdmittedConsumerContract,
    ///     envelopes: Vec<BridgeCommittedPatchEnvelope>,
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     let window = bridge.plan_change_stream_window(contract, envelopes)?;
    ///     let _checkpoint = bridge.publish_consumer_checkpoint(
    ///         contract,
    ///         &window,
    ///         worth_runtime_bridge::facade::StreamCheckpointFrontierKind::ContiguousFrontier,
    ///     );
    ///     Ok(())
    /// }
    /// ```
    pub fn plan_change_stream_window(
        &self,
        contract: &AdmittedConsumerContract,
        envelopes: Vec<BridgeCommittedPatchEnvelope>,
    ) -> Result<PlannedChangeStreamWindow, BridgeStreamError> {
        let window = crate::stream::plan_change_stream_window(contract, envelopes, 0)?;
        self.lower_change_stream_window(contract, window)
    }

    /// Publishes a checkpoint token from a planned stream window.
    pub fn publish_consumer_checkpoint(
        &self,
        contract: &AdmittedConsumerContract,
        window: &PlannedChangeStreamWindow,
        checkpoint_frontier_kind: StreamCheckpointFrontierKind,
    ) -> ConsumerCheckpointToken {
        let checkpoint = crate::stream::ConsumerCheckpointToken::from_window(
            contract,
            window,
            checkpoint_frontier_kind,
        );
        self.diagnostics
            .record_stream_checkpoint(checkpoint.clone());
        checkpoint
    }

    /// Validates that a checkpoint still matches a contract and window pair.
    pub fn validate_consumer_checkpoint(
        &self,
        contract: &AdmittedConsumerContract,
        window: &PlannedChangeStreamWindow,
        checkpoint: &ConsumerCheckpointToken,
    ) -> Result<(), BridgeStreamError> {
        crate::stream::validate_checkpoint_for_window(contract, window, checkpoint)
    }

    /// Canonicalizes the replay record for a stream checkpoint.
    pub fn canonicalize_stream_replay_record(
        &self,
        contract: &AdmittedConsumerContract,
        window: &PlannedChangeStreamWindow,
        checkpoint: &ConsumerCheckpointToken,
    ) -> Result<CanonicalStreamReplayRecord, BridgeStreamError> {
        let record =
            crate::stream::canonicalize_stream_replay_record(contract, window, checkpoint)?;
        self.diagnostics.record_stream_replay_record(record.clone());
        Ok(record)
    }

    /// Resumes stream planning from a retained checkpoint identity.
    ///
    /// ```no_run
    /// use worth_runtime_bridge::facade::{
    ///     AdmittedConsumerContract, BridgeCommittedPatchEnvelope, RuntimeBridge,
    /// };
    ///
    /// fn resume_stream_window(
    ///     bridge: &RuntimeBridge,
    ///     contract: &AdmittedConsumerContract,
    ///     envelopes: Vec<BridgeCommittedPatchEnvelope>,
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     let window = bridge.plan_change_stream_window(contract, envelopes.clone())?;
    ///     let checkpoint = bridge.publish_consumer_checkpoint(
    ///         contract,
    ///         &window,
    ///         worth_runtime_bridge::facade::StreamCheckpointFrontierKind::ContiguousFrontier,
    ///     );
    ///     let _resumed = bridge.resume_stream_window_from_checkpoint(
    ///         contract,
    ///         envelopes,
    ///         checkpoint.checkpoint_token_identity(),
    ///     )?;
    ///     Ok(())
    /// }
    /// ```
    pub fn resume_stream_window_from_checkpoint(
        &self,
        contract: &AdmittedConsumerContract,
        envelopes: Vec<BridgeCommittedPatchEnvelope>,
        checkpoint_identity: &CheckpointTokenIdentity,
    ) -> Result<BridgeStreamResumeSummary, BridgeStreamError> {
        let checkpoint_identity_label = checkpoint_identity.as_str();
        let checkpoint = self
            .diagnostics
            .stream_checkpoint_for_identity(checkpoint_identity)
            .ok_or_else(|| {
                BridgeStreamError::new(
                    BridgeStreamErrorKind::CheckpointTruncated,
                    format!(
                        "The checkpoint `{checkpoint_identity_label}` is no longer retained by the bridge diagnostics store."
                    ),
                )
            })?;
        let replay_record = match self
            .diagnostics
            .stream_replay_record_for_checkpoint_identity(checkpoint_identity)
        {
            Some(record) => record,
            None => return Err(BridgeStreamError::new(
                BridgeStreamErrorKind::CheckpointTruncated,
                format!(
                    "The checkpoint `{checkpoint_identity_label}` no longer retains its canonical replay record."
                ),
            )),
        };
        if checkpoint.consumer_contract_identity() != contract.consumer_contract_identity() {
            return Err(BridgeStreamError::new(
                BridgeStreamErrorKind::CheckpointContractMismatch,
                "The checkpoint token was issued for a different consumer contract identity.",
            ));
        }
        if checkpoint.stream_protocol_identity() != contract.stream_protocol_identity() {
            return Err(BridgeStreamError::new(
                BridgeStreamErrorKind::ProtocolVersionMismatch,
                "The checkpoint token was issued under a different stream protocol identity.",
            ));
        }

        let resumed_window = self.lower_change_stream_window(
            contract,
            crate::stream::plan_change_stream_window(
                contract,
                envelopes,
                checkpoint.checkpoint_member_count(),
            )?,
        )?;
        if resumed_window.members().iter().any(|member| {
            member.stream_member_identity()
                == checkpoint.contiguous_acknowledged_through_member_identity()
        }) {
            return Err(BridgeStreamError::new(
                BridgeStreamErrorKind::CheckpointStreamMismatch,
                "Resume material overlapped the acknowledged checkpoint frontier instead of beginning after it.",
            ));
        }

        Ok(BridgeStreamResumeSummary::new(
            checkpoint,
            replay_record,
            resumed_window,
        ))
    }

    /// Validates a replay record against a contract, window, and checkpoint.
    pub fn validate_stream_replay_record(
        &self,
        contract: &AdmittedConsumerContract,
        window: &PlannedChangeStreamWindow,
        checkpoint: &ConsumerCheckpointToken,
        record: &CanonicalStreamReplayRecord,
    ) -> Result<(), BridgeStreamError> {
        crate::stream::validate_stream_replay_record(contract, window, checkpoint, record)
    }

    /// Classifies backpressure for a planned stream window without changing its semantics.
    pub fn classify_stream_backpressure(
        &self,
        window: &PlannedChangeStreamWindow,
    ) -> BackpressureDecisionRecord {
        crate::stream::BackpressureDecisionRecord::classify(window)
    }

    /// Delivers one lowered change stream window to the configured consumer.
    pub fn deliver_change_stream_window(
        &self,
        contract: &AdmittedConsumerContract,
        window: &PlannedChangeStreamWindow,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<StreamWindowDeliveryResult, BridgeStreamError> {
        crate::stream::deliver_change_stream_window(self, contract, window, execution)
    }

    /// Delivers a replay-audit stream window and retains its proof artifacts.
    pub fn deliver_replay_audit_stream_window(
        &self,
        contract: &AdmittedConsumerContract,
        window: &PlannedChangeStreamWindow,
    ) -> Result<StreamReplayAuditResult, BridgeStreamError> {
        let result = crate::stream::audit_change_stream_window(contract, window)?;
        self.diagnostics
            .record_stream_checkpoint(result.checkpoint().clone());
        self.diagnostics
            .record_stream_replay_record(result.replay_record().clone());
        Ok(result)
    }

    fn lower_change_stream_window(
        &self,
        contract: &AdmittedConsumerContract,
        window: PlannedChangeStreamWindow,
    ) -> Result<PlannedChangeStreamWindow, BridgeStreamError> {
        let lowered_change_set = match contract.consumer_shape() {
            StreamConsumerShape::RoutingConsumer => {
                let planned_routes = window
                    .members()
                    .iter()
                    .map(|member| {
                        self.plan_envelope(member.committed_envelope().clone()).map_err(|error| {
                            BridgeStreamError::new(
                                BridgeStreamErrorKind::StreamDeliveryRejected,
                                format!(
                                    "Failed to lower canonical stream member `{}` into a planned route: {error}",
                                    member.stream_member_identity()
                                ),
                            )
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                LoweredConsumedChangeSet::Routing {
                    planned_routes: planned_routes.into(),
                }
            }
            StreamConsumerShape::ReplayAuditConsumer => LoweredConsumedChangeSet::ReplayAudit {
                canonical_member_identities: window
                    .members()
                    .iter()
                    .map(|member| std::sync::Arc::from(member.stream_member_identity()))
                    .collect::<Vec<_>>()
                    .into(),
            },
        };
        Ok(window.with_lowered_change_set(lowered_change_set))
    }
}
