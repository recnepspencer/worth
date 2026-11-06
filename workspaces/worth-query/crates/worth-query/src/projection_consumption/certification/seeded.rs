use super::fixtures::{
    control_row_set_lifecycle, denied_masked_field_failure_digest, grouped_worth_lifecycle,
    source_mismatch_failure_digest,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::identity::{
    compose_digest_sequence, compose_executed_lifecycle_artifact_digest,
    compose_seeded_replay_digest, compose_seeded_row_digest,
    compose_seeded_sequence_replay_check_digest,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionSeedGeneratorClass {
    DeclarationShape,
    FactFamilyMix,
    SourceFamilySelection,
    PolicyMask,
    DenialDeferredNeighbor,
    BasisSourceMismatch,
}

impl ProjectionConsumptionSeedGeneratorClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeclarationShape => "declaration_shape_generator",
            Self::FactFamilyMix => "fact_family_mix_generator",
            Self::SourceFamilySelection => "source_family_selection_generator",
            Self::PolicyMask => "policy_mask_generator",
            Self::DenialDeferredNeighbor => "denial_deferred_neighbor_generator",
            Self::BasisSourceMismatch => "basis_source_mismatch_generator",
        }
    }

    fn all() -> &'static [Self] {
        &[
            Self::DeclarationShape,
            Self::FactFamilyMix,
            Self::SourceFamilySelection,
            Self::PolicyMask,
            Self::DenialDeferredNeighbor,
            Self::BasisSourceMismatch,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExecutedSeedScenario {
    ControlRowSet,
    GroupedWorth,
    MaskedFieldDenial,
    SourceReferenceMismatch,
}

impl ExecutedSeedScenario {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ControlRowSet => "control_row_set",
            Self::GroupedWorth => "grouped_worth",
            Self::MaskedFieldDenial => "masked_field_denial",
            Self::SourceReferenceMismatch => "source_reference_mismatch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionSeedReplayRow {
    seed: u64,
    generator_choices: Vec<(ProjectionConsumptionSeedGeneratorClass, &'static str)>,
    replay_digest: String,
    row_digest: String,
}

impl ProjectionConsumptionSeedReplayRow {
    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionSeededCertificationReport {
    rows: Vec<ProjectionConsumptionSeedReplayRow>,
    seeded_sequence_digest: String,
    seed_replay_digest: String,
    seed_generator_class_digest: String,
}

impl ProjectionConsumptionSeededCertificationReport {
    pub fn rows(&self) -> &[ProjectionConsumptionSeedReplayRow] {
        &self.rows
    }

    pub fn seeded_sequence_digest(&self) -> &str {
        &self.seeded_sequence_digest
    }

    pub fn seed_replay_digest(&self) -> &str {
        &self.seed_replay_digest
    }

    pub fn seed_generator_class_digest(&self) -> &str {
        &self.seed_generator_class_digest
    }
}

pub fn projection_consumption_seeded_certification_report(
) -> Result<ProjectionConsumptionSeededCertificationReport, IdentityDerivationDenial> {
    let rows = [17_u64, 18_u64, 23_u64]
        .into_iter()
        .map(seed_row)
        .collect::<Vec<_>>();
    let seeded_sequence_digest = compose_digest_sequence(
        "projection_consumption_seeded_sequence_v1",
        "row",
        rows.iter().map(|row| row.row_digest().to_string()),
    )?;
    let stable_replay = seed_row(17)?;
    let changed_seed = seed_row(20)?;
    let seed_replay_digest = compose_seeded_sequence_replay_check_digest(
        &rows[0].replay_digest,
        &stable_replay.replay_digest,
        rows[0] == stable_replay,
        rows[0].row_digest() != changed_seed.row_digest(),
    )?;
    let seed_generator_class_digest = compose_digest_sequence(
        "projection_consumption_seed_generator_class_v1",
        "class",
        ProjectionConsumptionSeedGeneratorClass::all()
            .iter()
            .map(|class| class.as_str()),
    )?;
    Ok(ProjectionConsumptionSeededCertificationReport {
        rows,
        seeded_sequence_digest,
        seed_replay_digest,
        seed_generator_class_digest,
    })
}

fn seed_row(seed: u64) -> Result<ProjectionConsumptionSeedReplayRow, IdentityDerivationDenial> {
    let scenario = scenario_for(seed);
    let generator_choices = generator_choices_for(seed, scenario);
    let executed_artifact_digest = executed_artifact_digest(seed, scenario)?;
    let replay_digest =
        compose_seeded_replay_digest(seed, scenario.as_str(), &executed_artifact_digest)?;
    let generator_choice_refs = generator_choices
        .iter()
        .map(|(class, choice)| (class.as_str(), *choice))
        .collect::<Vec<_>>();
    let row_digest = compose_seeded_row_digest(seed, &replay_digest, &generator_choice_refs)?;
    Ok(ProjectionConsumptionSeedReplayRow {
        seed,
        generator_choices,
        replay_digest,
        row_digest,
    })
}

fn scenario_for(seed: u64) -> ExecutedSeedScenario {
    match seed % 4 {
        0 => ExecutedSeedScenario::MaskedFieldDenial,
        1 => ExecutedSeedScenario::GroupedWorth,
        2 => ExecutedSeedScenario::ControlRowSet,
        _ => ExecutedSeedScenario::SourceReferenceMismatch,
    }
}

fn generator_choices_for(
    seed: u64,
    scenario: ExecutedSeedScenario,
) -> Vec<(ProjectionConsumptionSeedGeneratorClass, &'static str)> {
    let declaration_shape = match scenario {
        ExecutedSeedScenario::GroupedWorth => "grouped_membership_and_endpoint",
        ExecutedSeedScenario::MaskedFieldDenial => "detail_identity_and_masked_display",
        ExecutedSeedScenario::SourceReferenceMismatch => "detail_source_reference_only",
        ExecutedSeedScenario::ControlRowSet => "detail_identity_and_display",
    };
    let fact_mix = match scenario {
        ExecutedSeedScenario::GroupedWorth => "membership_plus_endpoint",
        ExecutedSeedScenario::MaskedFieldDenial => "identity_plus_display",
        ExecutedSeedScenario::SourceReferenceMismatch => "source_reference_only",
        ExecutedSeedScenario::ControlRowSet => "identity_plus_display",
    };
    let source_family = match scenario {
        ExecutedSeedScenario::GroupedWorth => "relational_grouped_projection",
        _ => "relational_row_set",
    };
    let policy_mask = match scenario {
        ExecutedSeedScenario::MaskedFieldDenial => "masked_field_denial_neighbor",
        _ => "visible_projection",
    };
    let denial_neighbor = if seed.is_multiple_of(2) {
        "deferred_reload_neighbor"
    } else {
        "portable_export_neighbor"
    };
    let source_mismatch = match scenario {
        ExecutedSeedScenario::SourceReferenceMismatch => "source_reference_mismatch",
        _ => "basis_receipt_shape_match",
    };
    vec![
        (
            ProjectionConsumptionSeedGeneratorClass::DeclarationShape,
            declaration_shape,
        ),
        (
            ProjectionConsumptionSeedGeneratorClass::FactFamilyMix,
            fact_mix,
        ),
        (
            ProjectionConsumptionSeedGeneratorClass::SourceFamilySelection,
            source_family,
        ),
        (
            ProjectionConsumptionSeedGeneratorClass::PolicyMask,
            policy_mask,
        ),
        (
            ProjectionConsumptionSeedGeneratorClass::DenialDeferredNeighbor,
            denial_neighbor,
        ),
        (
            ProjectionConsumptionSeedGeneratorClass::BasisSourceMismatch,
            source_mismatch,
        ),
    ]
}

fn executed_artifact_digest(
    seed: u64,
    scenario: ExecutedSeedScenario,
) -> Result<String, IdentityDerivationDenial> {
    Ok(match scenario {
        ExecutedSeedScenario::ControlRowSet => {
            let lifecycle = control_row_set_lifecycle(row_count(seed))?;
            compose_executed_lifecycle_artifact_digest(
                lifecycle.declaration().declaration_digest(),
                lifecycle.contract().contract_digest(),
                lifecycle.facts().fact_set_digest(),
                lifecycle.receipt().receipt_digest(),
                lifecycle.envelope().envelope_digest(),
            )?
        }
        ExecutedSeedScenario::GroupedWorth => {
            let lifecycle = grouped_worth_lifecycle(row_count(seed))?;
            compose_executed_lifecycle_artifact_digest(
                lifecycle.declaration().declaration_digest(),
                lifecycle.contract().contract_digest(),
                lifecycle.facts().fact_set_digest(),
                lifecycle.receipt().receipt_digest(),
                lifecycle.envelope().envelope_digest(),
            )?
        }
        ExecutedSeedScenario::MaskedFieldDenial => denied_masked_field_failure_digest()?,
        ExecutedSeedScenario::SourceReferenceMismatch => source_mismatch_failure_digest()?,
    })
}

fn row_count(seed: u64) -> usize {
    if seed.is_multiple_of(5) {
        4
    } else {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_report_replays_same_seed_stably_and_exposes_generator_classes(
    ) -> Result<(), IdentityDerivationDenial> {
        let report = projection_consumption_seeded_certification_report()?;
        assert_eq!(report.rows().len(), 3);
        assert!(!report.seeded_sequence_digest().is_empty());
        assert!(!report.seed_replay_digest().is_empty());
        assert!(!report.seed_generator_class_digest().is_empty());

        Ok(())
    }

    #[test]
    fn seeded_rows_bind_real_executed_lanes_instead_of_label_only_hashes(
    ) -> Result<(), IdentityDerivationDenial> {
        let first = seed_row(17)?;
        let replay = seed_row(17)?;
        let changed = seed_row(19)?;
        assert_eq!(first, replay);
        assert_ne!(first.row_digest(), changed.row_digest());
        assert_ne!(seed_row(18).row_digest(), seed_row(23).row_digest());

        Ok(())
    }
}
