use super::{
    BridgeConditionalCondition, BridgeConditionalDenial, BridgeConditionalDenialKind,
    BridgeInstalledConditionalLowering,
};

type TruthSnapshotContext =
    crate::snapshot::AdmittedSnapshotContext<Box<dyn crate::snapshot::TruthSnapshotReader>>;

pub(super) fn read_condition_observations(
    snapshot: Option<&TruthSnapshotContext>,
    lowering: &BridgeInstalledConditionalLowering,
    previous: &super::observation_retention::BridgeRetainedObservations,
    managed_source_record: Option<crate::relational_identity::RelationalBridgeRecordIdentityParts>,
    ledger: &std::sync::Arc<super::retention::BridgeRetentionLedger>,
) -> Result<super::observation_retention::BridgeRetainedObservations, BridgeConditionalDenial> {
    let condition = lowering.contract.condition();
    let Some(snapshot) = admit_observation_snapshot(condition, snapshot)? else {
        return Ok(Default::default());
    };
    let plan = lowering.semantic_observation_plan.as_ref().ok_or_else(|| {
        BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::SnapshotAdmission,
            "installed semantic condition lost its compiled observation plan",
        )
    })?;
    let packet = plan.packet(managed_source_record)?;
    let result = snapshot.read_packet(&packet).map_err(|error| {
        BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::SnapshotAdmission,
            format!("conditional semantic observation failed: {error}"),
        )
    })?;
    if result.snapshot_identity() != snapshot.snapshot_identity() {
        return Err(BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::SnapshotMismatch,
            "conditional observation packet belongs to another admitted source snapshot",
        ));
    }
    let validated = crate::snapshot::validate_snapshot_read_result_contract(&packet, result)
        .map_err(|error| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SnapshotAdmission,
                format!("conditional semantic observation violated its contract: {error}"),
            )
        })?;
    super::observation_retention::prepare_observations(
        plan,
        previous,
        validated
            .records()
            .iter()
            .map(|record| record.validated_value_posture()),
        ledger,
    )
}

fn admit_observation_snapshot<'a>(
    condition: &BridgeConditionalCondition,
    snapshot: Option<&'a TruthSnapshotContext>,
) -> Result<Option<&'a TruthSnapshotContext>, BridgeConditionalDenial> {
    if !matches!(
        condition,
        BridgeConditionalCondition::DeltaThreshold(_)
            | BridgeConditionalCondition::RuntimePredicate
            | BridgeConditionalCondition::TemporalWake
    ) {
        return Ok(None);
    }
    if snapshot.is_none() && matches!(condition, BridgeConditionalCondition::DeltaThreshold(_)) {
        return Err(BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::SnapshotAdmission,
            "a typed delta threshold requires an admitted truth snapshot",
        ));
    }
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    #[test]
    fn authoritative_clear_materializes_an_explicit_absent_current_observation() {
        let plan = super::super::semantic_observation_plan::BridgeConditionalSemanticObservationPlan::managed_test_plan();
        let ledger = super::super::retention::BridgeRetentionLedger::new(
            crate::policy::BridgeConditionalRetentionBudget::development(),
        )
        .unwrap();
        let observations = super::super::observation_retention::prepare_observations(
            &plan,
            &Default::default(),
            [None].into_iter(),
            &ledger,
        )
        .unwrap();

        assert_eq!(observations.len(), 1);
        assert!(observations[0].previous().is_none());
        assert!(observations[0].current().is_none());
    }
}
