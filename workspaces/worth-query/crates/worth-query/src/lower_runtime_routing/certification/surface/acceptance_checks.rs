use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::BTreeMap;

use crate::identity::hash_parts;
use crate::lower_runtime_routing::{
    worth_query_lower_runtime_closeout_registry, worth_query_lower_runtime_crossing_inventory,
    worth_query_lower_runtime_gap_registry, worth_query_lower_runtime_support_matrix,
    WorthQueryLowerRuntimeCloseoutPosture, WorthQueryLowerRuntimeCrossingClassification,
    WorthQueryLowerRuntimeSeamKey, WorthQueryLowerRuntimeSupportPosture,
};

use super::acceptance_cardinality::admitted_crossing_cardinality_digest;
use super::acceptance_policy::{
    allowed_phase_six_synthetic_seams, required_phase_six_concrete_seams,
};
use super::evidence::WorthQueryLowerRuntimeRepresentativeEvidenceSource;
use super::evidence::WorthQueryLowerRuntimeRepresentativeSurface;

pub(super) fn control_digest(
    surface: &WorthQueryLowerRuntimeRepresentativeSurface,
) -> Result<String, IdentityDerivationDenial> {
    Ok(hash_parts(&[
        classification_exactness_digest(),
        deletable_seam_deletion_digest(),
        surviving_specialist_justification_digest(),
        admitted_crossing_cardinality_digest(surface)?,
        support_behavior_agreement_digest(surface),
        required_concrete_seam_coverage_digest(surface),
        synthetic_tail_exactness_digest(surface),
    ]))
}

pub(super) fn hostile_digest(surface: &WorthQueryLowerRuntimeRepresentativeSurface) -> String {
    hash_parts(&[
        "deleted-seam-survival-is-forbidden".to_string(),
        "specialist-gap-survival-is-forbidden".to_string(),
        "crossing-cardinality-drift-is-forbidden".to_string(),
        "support-behavior-drift-is-forbidden".to_string(),
        "required-phase-six-seams-must-not-fall-back-to-synthetic".to_string(),
        "synthetic-tail-overclaim-is-forbidden".to_string(),
        surface.route_parity_digest().to_string(),
    ])
}

pub(super) fn required_concrete_seam_coverage_digest(
    surface: &WorthQueryLowerRuntimeRepresentativeSurface,
) -> String {
    for seam_key in required_phase_six_concrete_seams() {
        assert_eq!(
            surface.evidence_source_for(*seam_key),
            Some(WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture),
            "required phase six seam {} must remain runtime-backed",
            seam_key.as_str()
        );
    }

    hash_parts(
        &required_phase_six_concrete_seams()
            .iter()
            .map(|seam_key| seam_key.as_str().to_string())
            .collect::<Vec<_>>(),
    )
}

pub(super) fn synthetic_tail_exactness_digest(
    surface: &WorthQueryLowerRuntimeRepresentativeSurface,
) -> String {
    let expected = allowed_phase_six_synthetic_seams();
    let actual = surface.synthetic_surface_seams();

    assert_eq!(
        actual.len(),
        expected.len(),
        "synthetic surface width drifted from the explicit phase six allowlist"
    );

    for row in expected {
        assert!(
            actual.contains(&row.seam_key().as_str()),
            "allowed synthetic seam {} disappeared from the certified synthetic tail",
            row.seam_key().as_str()
        );
        assert_eq!(
            surface.evidence_source_for(row.seam_key()),
            Some(WorthQueryLowerRuntimeRepresentativeEvidenceSource::InventorySynthesized),
            "allowed synthetic seam {} must remain inventory-synthesized until Phase 6 closeout converts it",
            row.seam_key().as_str()
        );
    }

    hash_parts(
        &expected
            .iter()
            .map(|row| format!("{}|{}", row.seam_key().as_str(), row.justification()))
            .collect::<Vec<_>>(),
    )
}

fn classification_exactness_digest() -> String {
    let crossings = worth_query_lower_runtime_crossing_inventory();
    let mut counts = BTreeMap::new();
    for row in crossings.rows() {
        *counts.entry(row.seam_key().as_str()).or_insert(0usize) += 1;
    }
    assert!(
        counts.values().all(|count| *count == 1),
        "every crossing must be classified exactly once"
    );
    hash_parts(
        &crossings
            .rows()
            .iter()
            .map(|row| {
                format!(
                    "{}|{}|{}",
                    row.seam_key().as_str(),
                    row.classification().as_str(),
                    row.route_kind().as_str()
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn deletable_seam_deletion_digest() -> String {
    let crossings = worth_query_lower_runtime_crossing_inventory();
    let gaps = worth_query_lower_runtime_gap_registry();
    let support = worth_query_lower_runtime_support_matrix();
    let eliminated = worth_query_lower_runtime_closeout_registry()
        .rows()
        .iter()
        .filter(|row| row.posture() == WorthQueryLowerRuntimeCloseoutPosture::SeamEliminated)
        .collect::<Vec<_>>();

    for row in &eliminated {
        assert!(
            !crossings
                .rows()
                .iter()
                .any(|crossing| crossing.seam_key() == row.seam_key()),
            "deleted seam must not survive in the crossing inventory"
        );
        assert!(
            !gaps
                .rows()
                .iter()
                .any(|gap| gap.seam_key() == row.seam_key()),
            "deleted seam must not survive in the gap registry"
        );
        let support_row = support
            .support_for(row.seam_key())
            .expect("deleted seam must remain locatable in support");
        assert_eq!(
            support_row.posture(),
            WorthQueryLowerRuntimeSupportPosture::SeamEliminated
        );
    }

    hash_parts(
        &eliminated
            .iter()
            .map(|row| {
                format!(
                    "{}|{}|{}",
                    row.seam_key().as_str(),
                    row.closeout_target(),
                    row.required_closeout()
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn surviving_specialist_justification_digest() -> String {
    let crossings = worth_query_lower_runtime_crossing_inventory();
    let gaps = worth_query_lower_runtime_gap_registry();
    let specialists = [
        WorthQueryLowerRuntimeSeamKey::FrontierEvidenceIntake,
        WorthQueryLowerRuntimeSeamKey::EffectBackedBridgeWriteback,
        WorthQueryLowerRuntimeSeamKey::HistoricalBridgeLowering,
        WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromQueryReceipts,
        WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromRelationalArtifacts,
        WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromBridgeArtifacts,
        WorthQueryLowerRuntimeSeamKey::CausalBridgeMaterialization,
    ];

    for seam_key in specialists {
        let row = crossings
            .rows()
            .iter()
            .find(|row| row.seam_key() == seam_key)
            .unwrap_or_else(|| {
                panic!(
                    "specialist seam {} must remain inventoried",
                    seam_key.as_str()
                )
            });
        assert_ne!(
            row.classification(),
            WorthQueryLowerRuntimeCrossingClassification::CompatibilityDebtLane,
            "specialist seam {} must not survive as compatibility debt",
            seam_key.as_str()
        );
        assert!(
            !gaps.rows().iter().any(|gap| gap.seam_key() == seam_key),
            "specialist seam {} must not survive in the gap registry",
            seam_key.as_str()
        );
    }

    hash_parts(
        &specialists
            .iter()
            .map(|seam_key| {
                let row = crossings
                    .rows()
                    .iter()
                    .find(|row| row.seam_key() == *seam_key)
                    .expect("specialist row should exist");
                format!(
                    "{}|{}|{}",
                    row.seam_key().as_str(),
                    row.classification().as_str(),
                    row.required_action()
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn support_behavior_agreement_digest(
    surface: &WorthQueryLowerRuntimeRepresentativeSurface,
) -> String {
    let support = worth_query_lower_runtime_support_matrix();
    let closeout = worth_query_lower_runtime_closeout_registry();

    for envelope in surface.envelopes() {
        let support_row = support
            .support_for(envelope.seam_key())
            .expect("every admitted envelope seam must exist in the support matrix");
        assert_eq!(support_row.posture(), envelope.support_posture());
        assert_eq!(support_row.authority_owner(), envelope.authority_owner());
        assert_eq!(support_row.route_kind(), envelope.route_kind());
        assert_eq!(support_row.capability_label(), envelope.capability_label());
    }
    for row in closeout.rows() {
        let support_row = support
            .support_for(row.seam_key())
            .expect("every closeout seam must exist in the support matrix");
        assert_eq!(support_row.authority_owner(), row.owner());
        assert_eq!(support_row.route_kind(), row.route_kind());
        match row.posture() {
            WorthQueryLowerRuntimeCloseoutPosture::SeamEliminated => {
                assert_eq!(
                    support_row.posture(),
                    WorthQueryLowerRuntimeSupportPosture::SeamEliminated
                );
            }
            WorthQueryLowerRuntimeCloseoutPosture::DeferredNeighbor => {
                assert_eq!(
                    support_row.posture(),
                    WorthQueryLowerRuntimeSupportPosture::Deferred
                );
            }
        }
    }

    hash_parts(&[
        hash_parts(
            &surface
                .envelopes()
                .iter()
                .map(|envelope| {
                    let support_row = support
                        .support_for(envelope.seam_key())
                        .expect("support row must exist");
                    format!(
                        "{}|{}|{}|{}",
                        envelope.seam_key().as_str(),
                        support_row.posture().as_str(),
                        support_row.authority_owner().as_str(),
                        support_row.route_kind().as_str()
                    )
                })
                .collect::<Vec<_>>(),
        ),
        hash_parts(
            &closeout
                .rows()
                .iter()
                .map(|row| {
                    let support_row = support
                        .support_for(row.seam_key())
                        .expect("support row must exist");
                    format!(
                        "{}|{}|{}",
                        row.seam_key().as_str(),
                        support_row.posture().as_str(),
                        row.certification_row()
                    )
                })
                .collect::<Vec<_>>(),
        ),
    ])
}
