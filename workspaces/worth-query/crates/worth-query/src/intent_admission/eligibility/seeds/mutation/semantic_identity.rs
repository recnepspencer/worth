use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::WorthQueryWriteCommand;

pub(super) fn metadata_identities(
    command: &WorthQueryWriteCommand,
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(command
        .mutation_metadata_ref()
        .into_iter()
        .flat_map(|metadata| metadata.entries())
        .map(|(key, value)| {
            worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeMutationIntentSeed)
                .field_shape(WorthQueryEvidenceTag::new("role"), "mutation-metadata")
                .field_value(WorthQueryEvidenceTag::new("key"), key.as_str())
                .field_value(
                    WorthQueryEvidenceTag::new("value"),
                    value.terminal_digest_text(),
                )
                .seal()
        })
        .collect::<Result<Vec<_>, _>>()?)
}

pub(super) fn naming_identity(
    command: &WorthQueryWriteCommand,
) -> Result<Option<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(command
        .naming_intent()
        .map(|intent| {
            worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeMutationIntentSeed)
                .field_shape(WorthQueryEvidenceTag::new("role"), "naming-intent")
                .field_shape(
                    WorthQueryEvidenceTag::new("family"),
                    intent.family().as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("attachment"),
                    intent.attachment_identity().evidence_identity(),
                )
                .optional_evidence_identity(
                    WorthQueryEvidenceTag::new("prior_authority"),
                    intent
                        .prior_authoritative_identity()
                        .map(|identity| identity.evidence_identity()),
                )
                .optional_evidence_identity(
                    WorthQueryEvidenceTag::new("target_authority"),
                    intent
                        .target_authoritative_identity()
                        .map(|identity| identity.evidence_identity()),
                )
                .seal()
        })
        .transpose()?)
}

pub(super) fn continuity_identity(
    command: &WorthQueryWriteCommand,
) -> Result<Option<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(command
        .continuity_intent()
        .map(|intent| {
            worth_query_evidence_identity(WorthQueryEvidenceScope::AuthoritativeMutationIntentSeed)
                .field_shape(WorthQueryEvidenceTag::new("role"), "continuity-intent")
                .field_shape(
                    WorthQueryEvidenceTag::new("family"),
                    intent.family().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("outcome_class"),
                    intent.outcome_class().as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("prior_authority"),
                    intent.prior_authoritative_identity().evidence_identity(),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("successor_authorities"),
                    intent
                        .successor_authoritative_identities()
                        .iter()
                        .map(|identity| identity.evidence_identity()),
                )
                .seal()
        })
        .transpose()?)
}

pub(super) fn mutation_command_shape(command: &WorthQueryWriteCommand) -> &'static str {
    match command {
        WorthQueryWriteCommand::InsertAspects { .. } => "insert-aspects",
        WorthQueryWriteCommand::UpdateAspect { .. } => "update-aspect",
        WorthQueryWriteCommand::UpdateAspects { .. } => "update-aspects",
        WorthQueryWriteCommand::UpdateExistingAspects { .. } => "update-existing-aspects",
        WorthQueryWriteCommand::VerifyThenUpdateExistingAspects { .. } => {
            "verify-then-update-existing-aspects"
        }
        WorthQueryWriteCommand::VerifyThenDeleteExistingAspects { .. } => {
            "verify-then-delete-existing-aspects"
        }
        WorthQueryWriteCommand::AssertExistingAspects { .. } => "assert-existing-aspects",
        WorthQueryWriteCommand::VerifyExistingAspects { .. } => "verify-existing-aspects",
        WorthQueryWriteCommand::UpdateSymbolicAspects { .. } => "update-symbolic-aspects",
        WorthQueryWriteCommand::DeleteAspects { .. } => "delete-aspects",
        WorthQueryWriteCommand::DeleteExistingAspects { .. } => "delete-existing-aspects",
        WorthQueryWriteCommand::DeleteSymbolicAspects { .. } => "delete-symbolic-aspects",
        WorthQueryWriteCommand::Delete { .. } => "delete",
    }
}
