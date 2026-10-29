use super::declarations::*;

pub(crate) fn assert_localization(
    artifact: &OfflineArtifactObservation,
    target: Target,
    operator: Operator,
) {
    if target == crate::store_offline_integrity_observer_support::Target::Root {
        assert_eq!(
            artifact.identity().as_str(),
            "root:0000000000000001",
            "a substituted payload must not replace the parent-addressed artifact identity"
        );
    }
    let artifact_bytes = if target == crate::store_offline_integrity_observer_support::Target::Root
    {
        384
    } else {
        107
    };
    assert_eq!(
        artifact
            .range()
            .map(|range| (range.offset(), range.length())),
        Some((0, artifact_bytes)),
        "canonical scope must not shrink with acquired bytes: {target:?} {operator:?}"
    );
    match operator {
        crate::root_protocol_corruption_matrix::Operator::D => assert_eq!(artifact.outcome(), &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Intact),
        crate::root_protocol_corruption_matrix::Operator::USchema => assert_unsupported(
            artifact,
            worth_store_offline_integrity_observer::OfflineUnsupportedVersionAxis::EnvelopeSchema,
            if target == crate::store_offline_integrity_observer_support::Target::Root { 8 } else { 3 },
            (9, 1),
        ),
        crate::root_protocol_corruption_matrix::Operator::UFormat => assert_unsupported(
            artifact,
            worth_store_offline_integrity_observer::OfflineUnsupportedVersionAxis::PhysicalRecordFormat,
            3,
            (10, 2),
        ),
        _ => {
            let (cause, range, field, blast) = expected_damage(target, operator, artifact_bytes);
            let worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Damaged(localization) = artifact.outcome() else {
                panic!(
                    "{target:?} {operator:?}: expected damage, got {:?}",
                    artifact.outcome()
                );
            };
            assert_eq!(localization.cause(), cause, "{target:?} {operator:?}");
            assert_eq!(
                localization
                    .damaged_range()
                    .map(|range| (range.offset(), range.length())),
                range,
                "{target:?} {operator:?}"
            );
            assert_eq!(localization.field(), field, "{target:?} {operator:?}");
            assert_eq!(
                localization.blast_radius(),
                blast,
                "{target:?} {operator:?}"
            );
        }
    }
}

pub(crate) fn expected_damage(
    target: Target,
    operator: Operator,
    artifact_bytes: u64,
) -> (
    OfflinePhysicalDamageCause,
    Option<(u64, u64)>,
    Option<OfflinePhysicalFormatField>,
    OfflinePhysicalBlastRadius,
) {
    match operator {
        crate::root_protocol_corruption_matrix::Operator::B | crate::root_protocol_corruption_matrix::Operator::K => (
            worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::ChecksumMismatch,
            Some((0, artifact_bytes)),
            None,
            worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius::Frame,
        ),
        crate::root_protocol_corruption_matrix::Operator::L => (
            worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::Framing,
            Some((24, 4)),
            Some(worth_store_offline_integrity_observer::OfflinePhysicalFormatField::PayloadLength),
            worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius::Field,
        ),
        crate::root_protocol_corruption_matrix::Operator::S if target == crate::store_offline_integrity_observer_support::Target::Root => (
            worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::ScopeMismatch,
            Some((48, 8)),
            Some(worth_store_offline_integrity_observer::OfflinePhysicalFormatField::ManifestGeneration),
            worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius::Field,
        ),
        crate::root_protocol_corruption_matrix::Operator::S => (
            worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::ScopeMismatch,
            Some((48, 16)),
            Some(worth_store_offline_integrity_observer::OfflinePhysicalFormatField::StoreIdentity),
            worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius::Field,
        ),
        crate::root_protocol_corruption_matrix::Operator::P if target == crate::store_offline_integrity_observer_support::Target::Root => (
            worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::Pointer,
            Some((296, 8)),
            Some(worth_store_offline_integrity_observer::OfflinePhysicalFormatField::ManifestPointer),
            worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius::ReachableRootSubtree,
        ),
        crate::root_protocol_corruption_matrix::Operator::P => (
            worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::Pointer,
            Some((65, 8)),
            Some(worth_store_offline_integrity_observer::OfflinePhysicalFormatField::RootGeneration),
            worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius::ReachableRootSubtree,
        ),
        crate::root_protocol_corruption_matrix::Operator::T => (
            worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::Truncation,
            Some((artifact_bytes - 1, 1)),
            None,
            worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius::Artifact,
        ),
        crate::root_protocol_corruption_matrix::Operator::R => (
            worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::MissingArtifact,
            None,
            None,
            if target == crate::store_offline_integrity_observer_support::Target::Root {
                worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius::ReachableRootSubtree
            } else {
                worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius::Artifact
            },
        ),
        crate::root_protocol_corruption_matrix::Operator::D => unreachable!(),
        crate::root_protocol_corruption_matrix::Operator::USchema | crate::root_protocol_corruption_matrix::Operator::UFormat => unreachable!(),
    }
}

pub(crate) fn assert_related_observations(
    report: &worth_store_offline_integrity_observer::OfflineIntegrityReport,
    target: Target,
    operator: Operator,
) {
    if matches!(
        operator,
        crate::root_protocol_corruption_matrix::Operator::D
    ) {
        let duplicates: Vec<_> = report
            .artifacts()
            .iter()
            .filter(|artifact| !artifact.duplicates().is_empty())
            .collect();
        assert_eq!(
            duplicates.len(),
            1,
            "{target:?} D: {:?}",
            report.artifacts()
        );
        assert_eq!(
            duplicates[0].outcome(),
            &worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Unknown(
                worth_store_offline_integrity_observer::OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected
            )
        );
        assert!(matches!(
            duplicates[0].duplicates(),
            [worth_store_offline_integrity_observer::OfflineArtifactDuplicateEvidence::PhysicalAlias { .. }]
        ));
    }
    if matches!(
        target,
        crate::store_offline_integrity_observer_support::Target::Current
            | crate::store_offline_integrity_observer_support::Target::Previous
    ) && matches!(
        operator,
        crate::root_protocol_corruption_matrix::Operator::P
    ) {
        let missing = report
            .artifacts()
            .iter()
            .find(|artifact| {
                artifact.relative_path() == "families/records/roots/root-0000000000000002.manifest"
            })
            .expect("selector P reports its missing addressed root");
        let worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Damaged(localization) =
            missing.outcome()
        else {
            panic!("missing addressed root must be damaged");
        };
        assert_eq!(
            localization.cause(),
            worth_store_offline_integrity_observer::OfflinePhysicalDamageCause::MissingArtifact
        );
        assert_eq!(
            localization.blast_radius(),
            worth_store_offline_integrity_observer::OfflinePhysicalBlastRadius::ReachableRootSubtree
        );
    }
}

pub(crate) fn assert_unsupported(
    artifact: &OfflineArtifactObservation,
    axis: OfflineUnsupportedVersionAxis,
    observed: u64,
    range: (u64, u64),
) {
    assert_eq!(
        artifact.outcome().posture(),
        worth_foundational::PhysicalIntegrityPosture::Unsupported
    );
    let worth_store_offline_integrity_observer::OfflineIntegrityOutcome::Unsupported(value) =
        artifact.outcome()
    else {
        unreachable!()
    };
    assert_eq!(value.axis(), axis);
    assert_eq!(value.observed(), observed);
    assert_eq!((value.range().offset(), value.range().length()), range);
}
