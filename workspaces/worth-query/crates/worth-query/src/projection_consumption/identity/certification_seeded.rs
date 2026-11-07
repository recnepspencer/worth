use super::scope::{certification_scope_encoder, compose_certification_sequence_digest, seal};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

pub(crate) fn compose_seeded_replay_digest(
    seed: u64,
    scenario: &str,
    executed_artifact_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_seeded_replay_v1")
            .field_shape(WorthQueryEvidenceTag::new("seed"), seed.to_string())
            .field_shape(WorthQueryEvidenceTag::new("scenario"), scenario)
            .field_shape(
                WorthQueryEvidenceTag::new("artifact"),
                executed_artifact_digest,
            ),
    )?)
}

fn compose_seeded_generator_choice_entry_digest(
    class: &str,
    choice: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_seeded_generator_choice_entry_v1")
            .field_shape(WorthQueryEvidenceTag::new("class"), class)
            .field_shape(WorthQueryEvidenceTag::new("choice"), choice),
    )?)
}

pub(crate) fn compose_seeded_row_digest(
    seed: u64,
    replay_digest: &str,
    generator_choices: &[(&str, &str)],
) -> Result<String, IdentityDerivationDenial> {
    let choices = generator_choices
        .iter()
        .map(|(class, choice)| compose_seeded_generator_choice_entry_digest(class, choice))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(seal(
        certification_scope_encoder("projection_consumption_seeded_row_v2")
            .field_shape(WorthQueryEvidenceTag::new("seed"), seed.to_string())
            .field_shape(WorthQueryEvidenceTag::new("replay"), replay_digest)
            .field_value_sequence(WorthQueryEvidenceTag::new("generator_choice"), choices),
    )?)
}

pub(crate) fn compose_seeded_sequence_replay_check_digest(
    first_replay_digest: &str,
    stable_replay_digest: &str,
    stable_match: bool,
    changed_match: bool,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_seed_replay_v2")
            .field_shape(
                WorthQueryEvidenceTag::new("first_replay"),
                first_replay_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("stable_replay"),
                stable_replay_digest,
            )
            .field_bool(WorthQueryEvidenceTag::new("stable"), stable_match)
            .field_bool(WorthQueryEvidenceTag::new("changed"), changed_match),
    )?)
}

pub(crate) fn compose_executed_lifecycle_artifact_digest(
    declaration_digest: &str,
    contract_digest: &str,
    fact_set_digest: &str,
    receipt_digest: &str,
    envelope_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "projection_consumption_executed_lifecycle_artifact_v1",
        "artifact",
        [
            declaration_digest,
            contract_digest,
            fact_set_digest,
            receipt_digest,
            envelope_digest,
        ],
    )?)
}
