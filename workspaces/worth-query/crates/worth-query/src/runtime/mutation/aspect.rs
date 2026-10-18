use std::collections::BTreeSet;

use worth_foundational::facade::AspectValue;

use super::WorthQueryMutationMetadata;
use crate::memory_workspace::{WorthQueryEntityIdentity, WorthQueryWorkspaceError};
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryContinuityMutationIntent,
    WorthQueryMutationTargetCollectionIdentity, WorthQueryNamingMutationIntent,
    WorthQueryRuntimeError, WorthQuerySymbolicAspectReference, WorthQuerySymbolicTargetReference,
    WorthQueryWriteCommand,
};

#[path = "aspect_builder_helpers.rs"]
mod aspect_builder_helpers;
#[path = "aspect_existing_truth.rs"]
mod aspect_existing_truth;
#[path = "aspect/authored_mutation.rs"]
mod authored_mutation;
#[path = "aspect/authored_value.rs"]
mod authored_value;

use aspect_builder_helpers::{finish_aspects, reject_symbolic_aspect_references};
pub use authored_mutation::WorthQueryAuthoredAspectMutation;
pub use authored_value::WorthQueryAuthoredAspectValue;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorthQueryAspectMutationBuilder {
    aspects: Vec<WorthQueryAuthoredAspectMutation>,
    symbolic_aspect_references: Vec<WorthQuerySymbolicAspectReference>,
    seen_aspects: BTreeSet<WorthQueryAspectTouch>,
    metadata: WorthQueryMutationMetadata,
    naming_intent: Option<WorthQueryNamingMutationIntent>,
    continuity_intent: Option<WorthQueryContinuityMutationIntent>,
    error: Option<String>,
}

impl WorthQueryAspectMutationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn aspect(
        mut self,
        authored_touch_text: impl Into<String>,
        value: impl Into<WorthQueryAuthoredAspectValue>,
    ) -> Self {
        if self.error.is_some() {
            return self;
        }
        match WorthQueryAspectTouch::from_authoring_ingress_text(authored_touch_text) {
            Ok(touch) => self.set_aspect(touch, value.into()),
            Err(error) => {
                self.error = Some(error.to_string());
                self
            }
        }
    }

    pub fn set_aspect(
        mut self,
        aspect_touch: WorthQueryAspectTouch,
        value: impl Into<WorthQueryAuthoredAspectValue>,
    ) -> Self {
        if self.error.is_some() {
            return self;
        }
        match WorthQueryAuthoredAspectMutation::new_set(
            aspect_touch,
            value.into().into_validation_input(),
        ) {
            Ok(aspect) => {
                let aspect_touch =
                    WorthQueryAspectTouch::from_parsed_target(aspect.parsed_target().clone());
                if !self.seen_aspects.insert(aspect_touch) {
                    self.error = Some(format!(
                        "aspect `{}` may only be declared once per mutation",
                        aspect.aspect_touch().admitted_touch_digest_part()
                    ));
                } else {
                    self.aspects.push(aspect);
                }
            }
            Err(error) => self.error = Some(error.to_string()),
        }
        self
    }

    pub fn symbolic_entity_identity(
        mut self,
        aspect_touch: WorthQueryAspectTouch,
        reference: WorthQuerySymbolicTargetReference,
    ) -> Self {
        if self.error.is_some() {
            return self;
        }
        if !self.seen_aspects.insert(aspect_touch.clone()) {
            let aspect_touch_digest = aspect_touch.admitted_touch_digest_part();
            self.error = Some(format!(
                "aspect `{aspect_touch_digest}` may only be declared once per mutation"
            ));
            return self;
        }
        self.symbolic_aspect_references.push(
            WorthQuerySymbolicAspectReference::same_batch_entity_identity(aspect_touch, reference),
        );
        self
    }

    pub fn existing_entity_identity(
        mut self,
        aspect_touch: WorthQueryAspectTouch,
        entity_identity: WorthQueryEntityIdentity,
    ) -> Self {
        if self.error.is_some() {
            return self;
        }
        let Some(parts) = entity_identity.relational_entity_record_parts() else {
            self.error =
                Some("existing aspect references require a relational entity identity".to_string());
            return self;
        };
        self.set_aspect(
            aspect_touch,
            WorthQueryAuthoredAspectValue::native(AspectValue::EntityRef(
                worth_foundational::facade::EntityId::new(
                    worth_foundational::facade::PartitionId(parts.partition_id()),
                    parts.local_slot(),
                    parts.generation(),
                ),
            )),
        )
    }

    pub fn clear(mut self, aspect_touch: WorthQueryAspectTouch) -> Self {
        if self.error.is_some() {
            return self;
        }
        match WorthQueryAuthoredAspectMutation::new_clear(aspect_touch) {
            Ok(aspect) => {
                let aspect_touch =
                    WorthQueryAspectTouch::from_parsed_target(aspect.parsed_target().clone());
                if !self.seen_aspects.insert(aspect_touch) {
                    self.error = Some(format!(
                        "aspect `{}` may only be declared once per mutation",
                        aspect.aspect_touch().admitted_touch_digest_part()
                    ));
                } else {
                    self.aspects.push(aspect);
                }
            }
            Err(error) => self.error = Some(error.to_string()),
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

    pub fn continuity_intent(mut self, intent: WorthQueryContinuityMutationIntent) -> Self {
        if self.error.is_some() {
            return self;
        }
        if self.continuity_intent.is_some() {
            self.error =
                Some("continuity intent may only be declared once per mutation".to_string());
            return self;
        }
        self.continuity_intent = Some(intent);
        self
    }

    pub fn build_insert(
        self,
        collection: impl Into<String>,
    ) -> Result<WorthQueryWriteCommand, WorthQueryRuntimeError> {
        self.build_insert_internal(collection, None)
    }

    pub(crate) fn build_insert_symbolic_reference(
        self,
        reference: WorthQuerySymbolicTargetReference,
        collection: impl Into<String>,
    ) -> Result<WorthQueryWriteCommand, WorthQueryRuntimeError> {
        self.build_insert_internal(collection, Some(reference))
    }

    fn build_insert_internal(
        self,
        collection: impl Into<String>,
        symbolic_target_reference: Option<WorthQuerySymbolicTargetReference>,
    ) -> Result<WorthQueryWriteCommand, WorthQueryRuntimeError> {
        let WorthQueryAspectMutationBuilder {
            aspects,
            symbolic_aspect_references,
            metadata,
            naming_intent,
            continuity_intent,
            error,
            ..
        } = self;
        let collection = collection.into();
        if collection.trim().is_empty() {
            return Err(WorthQueryRuntimeError::Workspace(
                WorthQueryWorkspaceError::new("collection may not be empty"),
            ));
        }
        let collection =
            WorthQueryMutationTargetCollectionIdentity::new("write-command-declared", collection);
        if let Some(reference_collection) = symbolic_target_reference
            .as_ref()
            .and_then(WorthQuerySymbolicTargetReference::target_collection_identity)
            && !collection.same_target_collection_as(reference_collection)
        {
            return Err(WorthQueryRuntimeError::Workspace(
                WorthQueryWorkspaceError::new(format!(
                    "symbolic target collection `{}` does not match insert collection `{}`",
                    reference_collection.as_str(),
                    collection.as_str()
                )),
            ));
        }
        Ok(WorthQueryWriteCommand::InsertAspects {
            collection,
            aspects: finish_aspects(aspects, error)?,
            metadata,
            naming_intent,
            continuity_intent,
            symbolic_target_reference,
            symbolic_aspect_references,
        })
    }

    pub fn build_update(
        self,
        entity_identity: WorthQueryEntityIdentity,
    ) -> Result<WorthQueryWriteCommand, WorthQueryRuntimeError> {
        let WorthQueryAspectMutationBuilder {
            aspects,
            symbolic_aspect_references,
            metadata,
            naming_intent,
            continuity_intent,
            error,
            ..
        } = self;
        reject_symbolic_aspect_references(&symbolic_aspect_references, "update-family authoring")?;
        Ok(WorthQueryWriteCommand::UpdateAspects {
            entity_identity,
            aspects: finish_aspects(aspects, error)?,
            metadata,
            naming_intent,
            continuity_intent,
        })
    }

    pub fn build_update_existing(
        self,
        binding: crate::runtime::WorthQueryExistingTruthTargetBinding,
    ) -> Result<WorthQueryWriteCommand, WorthQueryRuntimeError> {
        let WorthQueryAspectMutationBuilder {
            aspects,
            symbolic_aspect_references,
            metadata,
            naming_intent,
            continuity_intent,
            error,
            ..
        } = self;
        reject_symbolic_aspect_references(
            &symbolic_aspect_references,
            "existing-target update-family authoring",
        )?;
        Ok(WorthQueryWriteCommand::UpdateExistingAspects {
            binding,
            aspects: finish_aspects(aspects, error)?,
            metadata,
            naming_intent,
            continuity_intent,
        })
    }
    pub fn build_update_symbolic(
        self,
        reference: WorthQuerySymbolicTargetReference,
    ) -> Result<WorthQueryWriteCommand, WorthQueryRuntimeError> {
        let WorthQueryAspectMutationBuilder {
            aspects,
            symbolic_aspect_references,
            metadata,
            naming_intent,
            continuity_intent,
            error,
            ..
        } = self;
        reject_symbolic_aspect_references(
            &symbolic_aspect_references,
            "symbolic-target update-family authoring",
        )?;
        Ok(WorthQueryWriteCommand::UpdateSymbolicAspects {
            reference,
            aspects: finish_aspects(aspects, error)?,
            metadata,
            naming_intent,
            continuity_intent,
        })
    }
}
