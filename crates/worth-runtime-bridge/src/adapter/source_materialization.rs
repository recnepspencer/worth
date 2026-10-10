use super::*;

pub trait BridgeSourceAdapter: Send + Sync + 'static {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet;

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError>;

    fn materialize_packet(
        &self,
        planned: crate::snapshot::PlannedTruthViewPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<MaterializedTruthViewObservation, BridgeDeliveryError> {
        let snapshot_identity = planned
            .authority_basis()
            .snapshot_identity()
            .cloned()
            .ok_or_else(|| {
                BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::HistoricalPolicyRejected,
                    format!(
                        "Planned source packet `{}` has no resolved snapshot authority.",
                        planned.digest()
                    ),
                )
            })?;

        crate::execution_contact::admit(execution).map_err(|cause| {
            BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::ExecutionDenied(cause),
                cause.label(),
            )
        })?;
        let snapshot_reader =
            self.open_snapshot(&snapshot_identity, execution)
                .map_err(|error| {
                    BridgeDeliveryError::new(
                        error.delivery_kind(),
                        format!(
                            "Bridge source adapter failed to open snapshot `{}`: {error}",
                            snapshot_identity.as_str()
                        ),
                    )
                    .with_context(BridgeErrorContext::snapshot(snapshot_identity.clone()))
                })?;
        let snapshot = BridgeSnapshotContext::bind(snapshot_reader);
        let admitted = AdmittedSnapshotContext::admit_for(snapshot, &snapshot_identity).map_err(
            |bound_snapshot_identity| {
                BridgeDeliveryError::new(
                    BridgeDeliveryErrorKind::SnapshotIdentityMismatch,
                    format!(
                        "Source adapter bound snapshot `{}` but planned source packet required `{}`.",
                        bound_snapshot_identity.as_str(),
                        snapshot_identity.as_str()
                    ),
                )
                .with_context(BridgeErrorContext::snapshot(snapshot_identity.clone()))
            },
        )?;
        let snapshot_token = BridgeSnapshotToken::issued(
            snapshot_identity.clone(),
            format!(
                "source-truth-view-observation|planned={}|snapshot={}",
                planned.digest(),
                snapshot_identity.as_str()
            ),
        );

        Ok(MaterializedTruthViewObservation::new(
            planned.clone(),
            snapshot_token,
            source_materialization_path_for(&planned),
            admitted,
        ))
    }

    fn materialize_packets(
        &self,
        planned_packet_set: &PlannedSourceReadPacketSet,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<MaterializedTruthViewPacketSet, BridgeDeliveryError> {
        let observations = planned_packet_set
            .packets()
            .iter()
            .cloned()
            .map(|planned| self.materialize_packet(planned, execution))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(MaterializedTruthViewPacketSet::new(
            planned_packet_set.clone(),
            observations,
        ))
    }
}

fn source_materialization_path_for(
    planned: &crate::snapshot::PlannedTruthViewPacket,
) -> crate::diagnostics::BridgeHistoricalMaterializationPath {
    match planned.declaration().selector().view_kind() {
        BridgeTruthViewKind::CommittedSnapshot | BridgeTruthViewKind::BranchSnapshot => {
            crate::diagnostics::BridgeHistoricalMaterializationPath::DirectSnapshotRead
        }
        BridgeTruthViewKind::HistoricalCommit | BridgeTruthViewKind::BranchCommit => {
            crate::diagnostics::BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot
        }
        BridgeTruthViewKind::BranchHead => {
            crate::diagnostics::BridgeHistoricalMaterializationPath::BranchHeadEnvelopeSnapshot
        }
    }
}
