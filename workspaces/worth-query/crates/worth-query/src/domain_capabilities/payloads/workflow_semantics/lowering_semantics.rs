use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use worth_relational::facade::transactions::AspectFieldPatch;

use crate::workflow::{MergeLoweringInput, MutationLoweringInput, WritebackLoweringInput};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowLoweringSemantics {
    Mutation {
        authority_binding_identity: WorthQueryEvidenceIdentity,
        input: MutationLoweringInput,
    },
    Merge {
        input: MergeLoweringInput,
    },
    Writeback {
        input: WritebackLoweringInput,
    },
}

impl WorthQueryWorkflowLoweringSemantics {
    pub fn mutation(
        authority_binding_identity: WorthQueryEvidenceIdentity,
        input: MutationLoweringInput,
    ) -> Self {
        Self::Mutation {
            authority_binding_identity,
            input,
        }
    }

    pub fn merge(input: MergeLoweringInput) -> Self {
        Self::Merge { input }
    }

    pub fn writeback(input: WritebackLoweringInput) -> Self {
        Self::Writeback { input }
    }

    pub fn mutation_parts(&self) -> Option<(&WorthQueryEvidenceIdentity, &MutationLoweringInput)> {
        match self {
            Self::Mutation {
                authority_binding_identity,
                input,
            } => Some((authority_binding_identity, input)),
            _ => None,
        }
    }

    pub fn merge_input(&self) -> Option<&MergeLoweringInput> {
        match self {
            Self::Merge { input } => Some(input),
            _ => None,
        }
    }

    pub fn writeback_input(&self) -> Option<&WritebackLoweringInput> {
        match self {
            Self::Writeback { input } => Some(input),
            _ => None,
        }
    }

    pub(crate) fn semantics_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(match self {
            Self::Mutation {
                authority_binding_identity,
                input,
            } => {
                let mut encoder =
                    domain_capability_scope_encoder("worth_query_workflow_lowering_v1")
                        .field_shape(WorthQueryEvidenceTag::new("kind"), "mutation")
                        .field_evidence_identity(
                            WorthQueryEvidenceTag::new("authority_binding"),
                            authority_binding_identity,
                        );
                encoder = match input {
                    MutationLoweringInput::IntentReconciliation {
                        entity_id,
                        desired_aspect_fields,
                    } => encoder
                        .field_shape(
                            WorthQueryEvidenceTag::new("input_kind"),
                            "intent_reconciliation",
                        )
                        .field_usize(
                            WorthQueryEvidenceTag::new("partition_id"),
                            entity_id.partition_id.0 as usize,
                        )
                        .field_usize(
                            WorthQueryEvidenceTag::new("local_slot"),
                            entity_id.local_slot.0 as usize,
                        )
                        .field_usize(
                            WorthQueryEvidenceTag::new("generation"),
                            entity_id.generation.0 as usize,
                        )
                        .field_shape(
                            WorthQueryEvidenceTag::new("desired_aspect_fields"),
                            aspect_field_patch_identity_text(desired_aspect_fields),
                        ),
                };
                encoder.seal()?
            }
            Self::Merge { input } => {
                domain_capability_scope_encoder("worth_query_workflow_lowering_v1")
                    .field_shape(WorthQueryEvidenceTag::new("kind"), "merge")
                    .field_shape(
                        WorthQueryEvidenceTag::new("intent"),
                        input.intent().as_str(),
                    )
                    .field_shape(
                        WorthQueryEvidenceTag::new("target_branch"),
                        &input.target_branch().0,
                    )
                    .field_shape(
                        WorthQueryEvidenceTag::new("source_branch"),
                        &input.source_branch().0,
                    )
                    .seal()?
            }
            Self::Writeback { input } => {
                domain_capability_scope_encoder("worth_query_workflow_lowering_v1")
                    .field_shape(WorthQueryEvidenceTag::new("kind"), "writeback")
                    .field_shape(
                        WorthQueryEvidenceTag::new("family"),
                        input.family().as_str(),
                    )
                    .seal()?
            }
        })
    }
}

fn aspect_field_patch_identity_text(patch: &AspectFieldPatch) -> String {
    patch
        .to_canonical_bytes()
        .map(|bytes| {
            bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        })
        .unwrap_or_else(|_| "canonical_patch_serialization_failed".to_string())
}
