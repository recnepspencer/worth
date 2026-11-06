use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryAuthoredAspectMutation, WorthQueryContinuityMutationIntent,
    WorthQueryExistingTruthTargetBinding, WorthQueryMutationMetadata,
    WorthQueryMutationTargetCollectionIdentity, WorthQueryNamingMutationIntent,
    WorthQuerySymbolicAspectReference, WorthQuerySymbolicTargetReference, WorthQueryWriteCommand,
};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum WorthQueryBackendAdmissibleMutationShape {
    Insert {
        collection: WorthQueryMutationTargetCollectionIdentity,
        aspects: Vec<WorthQueryAuthoredAspectMutation>,
        symbolic_aspect_references: Vec<WorthQuerySymbolicAspectReference>,
        metadata: WorthQueryMutationMetadata,
        naming_intent: Option<WorthQueryNamingMutationIntent>,
        continuity_intent: Option<WorthQueryContinuityMutationIntent>,
        symbolic_target_reference: Option<WorthQuerySymbolicTargetReference>,
    },
    UpdateDirect {
        entity_identity: WorthQueryEntityIdentity,
        aspects: Vec<WorthQueryAuthoredAspectMutation>,
        metadata: WorthQueryMutationMetadata,
        naming_intent: Option<WorthQueryNamingMutationIntent>,
        continuity_intent: Option<WorthQueryContinuityMutationIntent>,
    },
    UpdateExisting {
        binding: WorthQueryExistingTruthTargetBinding,
        asserted_aspects: Vec<WorthQueryAuthoredAspectMutation>,
        aspects: Vec<WorthQueryAuthoredAspectMutation>,
        symbolic_aspect_references: Vec<WorthQuerySymbolicAspectReference>,
        metadata: WorthQueryMutationMetadata,
        naming_intent: Option<WorthQueryNamingMutationIntent>,
        continuity_intent: Option<WorthQueryContinuityMutationIntent>,
    },
    Assertion {
        binding: WorthQueryExistingTruthTargetBinding,
        aspects: Vec<WorthQueryAuthoredAspectMutation>,
        metadata: WorthQueryMutationMetadata,
    },
    UpdateSymbolic {
        reference: WorthQuerySymbolicTargetReference,
        aspects: Vec<WorthQueryAuthoredAspectMutation>,
        metadata: WorthQueryMutationMetadata,
        naming_intent: Option<WorthQueryNamingMutationIntent>,
        continuity_intent: Option<WorthQueryContinuityMutationIntent>,
    },
    DeleteDirect {
        entity_identity: WorthQueryEntityIdentity,
        declared_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
        touched_aspects: Vec<WorthQueryAspectTouch>,
        metadata: WorthQueryMutationMetadata,
        naming_intent: Option<WorthQueryNamingMutationIntent>,
    },
    DeleteExisting {
        binding: WorthQueryExistingTruthTargetBinding,
        asserted_aspects: Vec<WorthQueryAuthoredAspectMutation>,
        touched_aspects: Vec<WorthQueryAspectTouch>,
        metadata: WorthQueryMutationMetadata,
        naming_intent: Option<WorthQueryNamingMutationIntent>,
    },
    DeleteSymbolic {
        reference: WorthQuerySymbolicTargetReference,
        touched_aspects: Vec<WorthQueryAspectTouch>,
        metadata: WorthQueryMutationMetadata,
        naming_intent: Option<WorthQueryNamingMutationIntent>,
    },
}

impl WorthQueryBackendAdmissibleMutationShape {
    pub(super) fn from_admitted_command(
        command: WorthQueryWriteCommand,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(match command {
            WorthQueryWriteCommand::InsertAspects {
                collection,
                aspects,
                symbolic_aspect_references,
                metadata,
                naming_intent,
                continuity_intent,
                symbolic_target_reference,
            } => Self::Insert {
                collection: WorthQueryMutationTargetCollectionIdentity::new(
                    "backend-admissible-declared",
                    collection.as_str(),
                )?,
                aspects,
                symbolic_aspect_references,
                metadata,
                naming_intent,
                continuity_intent,
                symbolic_target_reference,
            },
            WorthQueryWriteCommand::UpdateAspect {
                entity_identity,
                aspect,
            } => Self::UpdateDirect {
                entity_identity,
                aspects: vec![aspect],
                metadata: WorthQueryMutationMetadata::default(),
                naming_intent: None,
                continuity_intent: None,
            },
            WorthQueryWriteCommand::UpdateAspects {
                entity_identity,
                aspects,
                metadata,
                naming_intent,
                continuity_intent,
            } => Self::UpdateDirect {
                entity_identity,
                aspects,
                metadata,
                naming_intent,
                continuity_intent,
            },
            WorthQueryWriteCommand::UpdateExistingAspects {
                binding,
                aspects,
                metadata,
                naming_intent,
                continuity_intent,
            } => Self::UpdateExisting {
                binding,
                asserted_aspects: Vec::new(),
                aspects,
                symbolic_aspect_references: Vec::new(),
                metadata,
                naming_intent,
                continuity_intent,
            },
            WorthQueryWriteCommand::VerifyThenUpdateExistingAspects {
                binding,
                asserted_aspects: _,
                aspects,
                symbolic_aspect_references,
                metadata,
                naming_intent,
                continuity_intent,
            } => Self::UpdateExisting {
                binding,
                asserted_aspects: Vec::new(),
                aspects,
                symbolic_aspect_references,
                metadata,
                naming_intent,
                continuity_intent,
            },
            WorthQueryWriteCommand::AssertExistingAspects {
                binding,
                aspects,
                metadata,
            }
            | WorthQueryWriteCommand::VerifyExistingAspects {
                binding,
                aspects,
                metadata,
            } => Self::Assertion {
                binding,
                aspects,
                metadata,
            },
            WorthQueryWriteCommand::UpdateSymbolicAspects {
                reference,
                aspects,
                metadata,
                naming_intent,
                continuity_intent,
            } => Self::UpdateSymbolic {
                reference,
                aspects,
                metadata,
                naming_intent,
                continuity_intent,
            },
            WorthQueryWriteCommand::DeleteAspects {
                entity_identity,
                declared_collection,
                touched_aspects,
                metadata,
                naming_intent,
            } => Self::DeleteDirect {
                entity_identity,
                declared_collection: declared_collection.map(|collection| {
                    WorthQueryMutationTargetCollectionIdentity::new(
                        "backend-admissible-declared",
                        collection.as_str(),
                    )
                })?,
                touched_aspects,
                metadata,
                naming_intent,
            },
            WorthQueryWriteCommand::Delete { entity_identity } => Self::DeleteDirect {
                entity_identity,
                declared_collection: None,
                touched_aspects: Vec::new(),
                metadata: WorthQueryMutationMetadata::default(),
                naming_intent: None,
            },
            WorthQueryWriteCommand::VerifyThenDeleteExistingAspects {
                binding,
                asserted_aspects: _,
                touched_aspects,
                metadata,
                naming_intent,
            } => Self::DeleteExisting {
                binding,
                asserted_aspects: Vec::new(),
                touched_aspects,
                metadata,
                naming_intent,
            },
            WorthQueryWriteCommand::DeleteExistingAspects {
                binding,
                touched_aspects,
                metadata,
                naming_intent,
            } => Self::DeleteExisting {
                binding,
                asserted_aspects: Vec::new(),
                touched_aspects,
                metadata,
                naming_intent,
            },
            WorthQueryWriteCommand::DeleteSymbolicAspects {
                reference,
                touched_aspects,
                metadata,
                naming_intent,
            } => Self::DeleteSymbolic {
                reference,
                touched_aspects,
                metadata,
                naming_intent,
            },
        })
    }
}
