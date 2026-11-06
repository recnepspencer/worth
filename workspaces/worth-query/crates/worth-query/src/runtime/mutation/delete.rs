use std::collections::BTreeSet;

use super::{WorthQueryAspectTouch, WorthQueryMutationMetadata};
use crate::memory_workspace::{WorthQueryEntityIdentity, WorthQueryWorkspaceError};
use crate::runtime::{
    WorthQueryAuthoredAspectMutation, WorthQueryExistingTruthTargetBinding,
    WorthQueryMutationTargetCollectionIdentity, WorthQueryNamingMutationIntent,
    WorthQueryRuntimeError, WorthQuerySymbolicTargetReference, WorthQueryWriteCommand,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorthQueryDeleteMutationBuilder {
    declared_collection: Option<WorthQueryMutationTargetCollectionIdentity>,
    touched_aspects: Vec<WorthQueryAspectTouch>,
    seen_aspects: BTreeSet<WorthQueryAspectTouch>,
    metadata: WorthQueryMutationMetadata,
    naming_intent: Option<WorthQueryNamingMutationIntent>,
    error: Option<String>,
}

impl WorthQueryDeleteMutationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn target_collection(mut self, collection: impl Into<String>) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        if self.error.is_some() {
            return Ok(self);
        }
        let collection = collection.into();
        if collection.trim().is_empty() {
            self.error = Some("delete target collection may not be empty".to_string());
            return Ok(self);
        }
        self.declared_collection = Some(WorthQueryMutationTargetCollectionIdentity::new(
            "write-command-declared",
            collection,
        )?);
        Ok(self)
    }

    pub fn touch(mut self, touch: WorthQueryAspectTouch) -> Self {
        if self.error.is_some() {
            return self;
        }
        if !self.seen_aspects.insert(touch.clone()) {
            let aspect_touch_digest = touch.admitted_touch_digest_part();
            self.error = Some(format!(
                "delete touch aspect `{aspect_touch_digest}` may only be declared once per mutation"
            ));
            return self;
        }
        self.touched_aspects.push(touch);
        self
    }

    pub fn touches(mut self, touches: impl IntoIterator<Item = WorthQueryAspectTouch>) -> Self {
        for touch in touches {
            self = self.touch(touch);
            if self.error.is_some() {
                break;
            }
        }
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.error.is_some() {
            return self;
        }
        if let Err(error) = self.metadata.insert(key, value) {
            self.error = Some(error.to_string());
        }
        self
    }

    pub fn naming_intent(mut self, intent: WorthQueryNamingMutationIntent) -> Self {
        if self.error.is_some() {
            return self;
        }
        if self.naming_intent.is_some() {
            self.error = Some("naming intent may only be declared once per mutation".to_string());
            return self;
        }
        self.naming_intent = Some(intent);
        self
    }

    pub fn naming_remove(
        self,
        attachment_identity: crate::runtime::WorthQueryMutationAuthorityIdentity,
        prior_authoritative_identity: crate::runtime::WorthQueryMutationAuthorityIdentity,
    ) -> Self {
        self.naming_intent(WorthQueryNamingMutationIntent::remove(
            attachment_identity,
            prior_authoritative_identity,
        ))
    }

    pub fn build_delete(
        self,
        entity_identity: WorthQueryEntityIdentity,
    ) -> Result<WorthQueryWriteCommand, WorthQueryRuntimeError> {
        if let Some(error) = self.error {
            return Err(WorthQueryRuntimeError::Workspace(
                WorthQueryWorkspaceError::new(error),
            ));
        }
        Ok(WorthQueryWriteCommand::DeleteAspects {
            entity_identity,
            declared_collection: self.declared_collection,
            touched_aspects: self.touched_aspects,
            metadata: self.metadata,
            naming_intent: self.naming_intent,
        })
    }

    pub fn build_delete_existing(
        self,
        binding: WorthQueryExistingTruthTargetBinding,
    ) -> Result<WorthQueryWriteCommand, WorthQueryRuntimeError> {
        if let Some(error) = self.error {
            return Err(WorthQueryRuntimeError::Workspace(
                WorthQueryWorkspaceError::new(error),
            ));
        }
        Ok(WorthQueryWriteCommand::DeleteExistingAspects {
            binding,
            touched_aspects: self.touched_aspects,
            metadata: self.metadata,
            naming_intent: self.naming_intent,
        })
    }

    pub(crate) fn build_delete_existing_verified(
        self,
        binding: WorthQueryExistingTruthTargetBinding,
        asserted_aspects: Vec<WorthQueryAuthoredAspectMutation>,
    ) -> Result<WorthQueryWriteCommand, WorthQueryRuntimeError> {
        if let Some(error) = self.error {
            return Err(WorthQueryRuntimeError::Workspace(
                WorthQueryWorkspaceError::new(error),
            ));
        }
        Ok(WorthQueryWriteCommand::VerifyThenDeleteExistingAspects {
            binding,
            asserted_aspects,
            touched_aspects: self.touched_aspects,
            metadata: self.metadata,
            naming_intent: self.naming_intent,
        })
    }

    pub fn build_delete_symbolic(
        self,
        reference: WorthQuerySymbolicTargetReference,
    ) -> Result<WorthQueryWriteCommand, WorthQueryRuntimeError> {
        if let Some(error) = self.error {
            return Err(WorthQueryRuntimeError::Workspace(
                WorthQueryWorkspaceError::new(error),
            ));
        }
        Ok(WorthQueryWriteCommand::DeleteSymbolicAspects {
            reference,
            touched_aspects: self.touched_aspects,
            metadata: self.metadata,
            naming_intent: self.naming_intent,
        })
    }
}
