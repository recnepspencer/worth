use crate::identity::hash_parts;

use super::super::fixtures::{
    certified_admitted_intent_fixture, certified_advisory_intent_fixture,
    certified_failure_intent_fixture, certified_violation_intent_fixture,
    legacy_delegation_parity_fixture,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryIntentAdmissionSeedGeneratorClass {
    FamilyChoice,
    SourceLaneChoice,
    OutcomeClassChoice,
    ExecutionSurfaceChoice,
    FailurePathChoice,
}

impl WorthQueryIntentAdmissionSeedGeneratorClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::FamilyChoice => "family_choice_generator",
            Self::SourceLaneChoice => "source_lane_choice_generator",
            Self::OutcomeClassChoice => "outcome_class_generator",
            Self::ExecutionSurfaceChoice => "execution_surface_generator",
            Self::FailurePathChoice => "failure_path_generator",
        }
    }

    fn all() -> &'static [Self] {
        &[
            Self::FamilyChoice,
            Self::SourceLaneChoice,
            Self::OutcomeClassChoice,
            Self::ExecutionSurfaceChoice,
            Self::FailurePathChoice,
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionSeedReplayRow {
    seed: u64,
    replay_digest: String,
    row_digest: String,
}

impl WorthQueryIntentAdmissionSeedReplayRow {
    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionSeededCertificationReport {
    rows: Vec<WorthQueryIntentAdmissionSeedReplayRow>,
    seeded_sequence_digest: String,
    seed_replay_digest: String,
    seed_generator_class_digest: String,
}

impl WorthQueryIntentAdmissionSeededCertificationReport {
    pub fn rows(&self) -> &[WorthQueryIntentAdmissionSeedReplayRow] {
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

pub fn worth_query_intent_admission_seeded_certification_report(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryIntentAdmissionSeededCertificationReport {
    let rows = [17_u64, 18_u64, 23_u64, 29_u64]
        .into_iter()
        .map(|seed| seed_row(execution, seed))
        .collect::<Vec<_>>();
    let seeded_sequence_digest = hash_parts(
        &rows
            .iter()
            .map(|row| row.row_digest().to_string())
            .collect::<Vec<_>>(),
    );
    let stable_replay = seed_row(execution, 17);
    let changed_seed = seed_row(execution, 30);
    let seed_replay_digest = hash_parts(&[
        "worth_query_intent_admission_seed_replay_v1".to_string(),
        format!("primary:{}", rows[0].replay_digest),
        format!("stable:{}", rows[0] == stable_replay),
        format!(
            "changed:{}",
            rows[0].row_digest() != changed_seed.row_digest()
        ),
    ]);
    let seed_generator_class_digest = hash_parts(
        &WorthQueryIntentAdmissionSeedGeneratorClass::all()
            .iter()
            .map(|class| class.as_str().to_string())
            .collect::<Vec<_>>(),
    );
    WorthQueryIntentAdmissionSeededCertificationReport {
        rows,
        seeded_sequence_digest,
        seed_replay_digest,
        seed_generator_class_digest,
    }
}

fn seed_row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    seed: u64,
) -> WorthQueryIntentAdmissionSeedReplayRow {
    let replay_digest = executed_artifact_digest(execution, seed);
    let row_digest = hash_parts(&[
        "worth_query_intent_admission_seed_row_v1".to_string(),
        format!("seed:{seed}"),
        format!("replay:{replay_digest}"),
        format!("family:{}", family_choice(seed)),
        format!("source:{}", source_lane_choice(seed)),
        format!("outcome:{}", outcome_class_choice(seed)),
        format!("surface:{}", execution_surface_choice(seed)),
        format!("failure-path:{}", failure_path_choice(seed)),
    ]);
    WorthQueryIntentAdmissionSeedReplayRow {
        seed,
        replay_digest,
        row_digest,
    }
}

fn executed_artifact_digest(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    seed: u64,
) -> String {
    match seed % 5 {
        0 => certified_failure_intent_fixture(execution).failure_digest,
        1 => certified_admitted_intent_fixture(execution)
            .receipt
            .receipt_digest()
            .to_string(),
        2 => certified_advisory_intent_fixture()
            .trace
            .trace_digest()
            .to_string(),
        3 => legacy_delegation_parity_fixture(execution)
            .effect_canonical
            .receipt_digest()
            .to_string(),
        _ => certified_violation_intent_fixture()
            .trace
            .trace_digest()
            .to_string(),
    }
}

fn family_choice(seed: u64) -> &'static str {
    if seed % 5 == 3 {
        "effect_triggered_write_intent"
    } else {
        "authoritative_user_intent"
    }
}

fn source_lane_choice(seed: u64) -> &'static str {
    if seed % 5 == 3 {
        "EffectTriggered"
    } else {
        "UserAuthored"
    }
}

fn outcome_class_choice(seed: u64) -> &'static str {
    match seed % 5 {
        0 => "failure",
        1 | 3 => "admitted",
        2 => "advisory",
        _ => "violation",
    }
}

fn execution_surface_choice(seed: u64) -> &'static str {
    match seed % 5 {
        3 => "effect_execution_surface",
        0 => "post_execution_routing_surface",
        _ => "authoritative_execution_surface",
    }
}

fn failure_path_choice(seed: u64) -> &'static str {
    if seed.is_multiple_of(2) {
        "signal_routing_failure"
    } else {
        "typed_decision_surface"
    }
}
