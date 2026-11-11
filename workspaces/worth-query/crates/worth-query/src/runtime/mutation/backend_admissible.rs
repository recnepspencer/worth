use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{
    WorthQueryAspectMutationOperation, WorthQueryAspectMutationOperationKind,
    WorthQueryAspectTouch, WorthQueryAuthoredAspectMutation, WorthQueryContinuityMutationIntent,
    WorthQueryExistingTruthTargetBinding, WorthQueryMutationFamily, WorthQueryMutationMetadata,
    WorthQueryMutationTargetCollectionIdentity, WorthQueryNamingMutationIntent,
    WorthQuerySymbolicAspectReference, WorthQuerySymbolicTargetReference, WorthQueryWriteCommand,
};

#[path = "backend_admissible/shape.rs"]
mod shape;

use shape::WorthQueryBackendAdmissibleMutationShape;

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryBackendAdmissibleMutation {
    shape: WorthQueryBackendAdmissibleMutationShape,
    authoritative_patch: worth_foundational::facade::AuthoritativeRecordAspectPatch,
    portable_patch: worth_foundational::facade::PortableRecordAspectPatch,
}

impl WorthQueryBackendAdmissibleMutation {
    /// Construct and validate one native field update without exposing
    /// Query's authored-command constructors or mutable contract registry.
    ///
    /// The returned mutation has crossed the same contract boundary used by
    /// runtime-authored writes, but it carries no write or bridge authority.
    pub fn admit_native_field_update(
        entity_identity: WorthQueryEntityIdentity,
        aspect_key: worth_foundational::facade::AspectKey,
        field_path: worth_foundational::facade::CanonicalFieldPath,
        value: worth_foundational::facade::AspectValue,
        contracts: impl IntoIterator<Item = worth_foundational::facade::AspectContract>,
    ) -> Result<Self, crate::runtime::WorthQueryAuthoredMutationAdmissionDenial> {
        let contracts =
            crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry::from_contracts(
                contracts,
            )?;
        let aspect = WorthQueryAuthoredAspectMutation::new_set(
            WorthQueryAspectTouch::aspect_field_path(aspect_key, field_path),
            value,
        )
        .expect("typed native field updates are infallible before contract admission");
        let command = WorthQueryWriteCommand::UpdateAspect {
            entity_identity,
            aspect,
        };
        Self::from_authored_command(command, &contracts).map_err(Into::into)
    }

    pub(crate) fn from_authored_command(
        command: WorthQueryWriteCommand,
        contracts: &crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry,
    ) -> Result<Self, crate::runtime::WorthQueryMutationContractDenial> {
        let authoritative_patch =
            crate::runtime::native_aspect_contracts::admit_authoritative_mutation_patch(
                &command, contracts,
            )?;
        let portable_patch = worth_foundational::facade::export_portable_record_aspect_patch(
            &authoritative_patch,
            contracts,
        )
        .map_err(
            crate::runtime::native_aspect_contracts::WorthQueryMutationContractDenial::portable_export_denied,
        )?;
        Ok(Self {
            shape: WorthQueryBackendAdmissibleMutationShape::from_admitted_command(command)?,
            authoritative_patch,
            portable_patch,
        })
    }

    pub(crate) fn portable_patch(&self) -> &worth_foundational::facade::PortableRecordAspectPatch {
        &self.portable_patch
    }

    pub(crate) fn authoritative_patch(
        &self,
    ) -> &worth_foundational::facade::AuthoritativeRecordAspectPatch {
        &self.authoritative_patch
    }

    pub fn mutation_family(&self) -> WorthQueryMutationFamily {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::Insert { .. } => {
                WorthQueryMutationFamily::Insert
            }
            WorthQueryBackendAdmissibleMutationShape::UpdateDirect { .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateExisting { .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateSymbolic { .. } => {
                WorthQueryMutationFamily::Update
            }
            WorthQueryBackendAdmissibleMutationShape::Assertion { .. } => {
                WorthQueryMutationFamily::Assertion
            }
            WorthQueryBackendAdmissibleMutationShape::DeleteDirect { .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteExisting { .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteSymbolic { .. } => {
                WorthQueryMutationFamily::Delete
            }
        }
    }

    pub fn declared_collection_identity(
        &self,
    ) -> Option<WorthQueryMutationTargetCollectionIdentity> {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::Insert { collection, .. } => {
                Some(collection.clone())
            }
            WorthQueryBackendAdmissibleMutationShape::UpdateExisting {
                binding,
                symbolic_aspect_references,
                ..
            } => binding.target_collection_identity().cloned().or_else(|| {
                symbolic_aspect_references
                    .first()
                    .and_then(|reference| reference.reference().target_collection_identity())
                    .cloned()
            }),
            WorthQueryBackendAdmissibleMutationShape::Assertion { binding, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteExisting { binding, .. } => {
                binding.target_collection_identity().cloned()
            }
            WorthQueryBackendAdmissibleMutationShape::UpdateSymbolic { reference, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteSymbolic { reference, .. } => {
                reference.target_collection_identity().cloned()
            }
            WorthQueryBackendAdmissibleMutationShape::DeleteDirect {
                declared_collection,
                ..
            } => declared_collection.clone(),
            WorthQueryBackendAdmissibleMutationShape::UpdateDirect { .. } => None,
        }
    }

    pub fn declared_entity_identity_ref(&self) -> Option<&WorthQueryEntityIdentity> {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::UpdateDirect {
                entity_identity, ..
            }
            | WorthQueryBackendAdmissibleMutationShape::DeleteDirect {
                entity_identity, ..
            } => Some(entity_identity),
            _ => None,
        }
    }

    pub fn declared_entity_identity(&self) -> Option<WorthQueryEntityIdentity> {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::UpdateExisting { binding, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteExisting { binding, .. }
            | WorthQueryBackendAdmissibleMutationShape::Assertion { binding, .. } => {
                Some(binding.resolved_entity_artifact_identity())
            }
            _ => self.declared_entity_identity_ref().cloned(),
        }
    }

    pub fn existing_truth_binding(&self) -> Option<&WorthQueryExistingTruthTargetBinding> {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::UpdateExisting { binding, .. }
            | WorthQueryBackendAdmissibleMutationShape::Assertion { binding, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteExisting { binding, .. } => {
                Some(binding)
            }
            _ => None,
        }
    }

    pub fn symbolic_target_reference(&self) -> Option<&WorthQuerySymbolicTargetReference> {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::Insert {
                symbolic_target_reference,
                ..
            } => symbolic_target_reference.as_ref(),
            WorthQueryBackendAdmissibleMutationShape::UpdateSymbolic { reference, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteSymbolic { reference, .. } => {
                Some(reference)
            }
            _ => None,
        }
    }

    pub fn admitted_aspect_values(&self) -> &[WorthQueryAuthoredAspectMutation] {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::Insert { aspects, .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateDirect { aspects, .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateExisting { aspects, .. }
            | WorthQueryBackendAdmissibleMutationShape::Assertion { aspects, .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateSymbolic { aspects, .. } => aspects,
            WorthQueryBackendAdmissibleMutationShape::DeleteDirect { .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteExisting { .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteSymbolic { .. } => &[],
        }
    }

    pub fn asserted_admitted_aspect_values(&self) -> &[WorthQueryAuthoredAspectMutation] {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::UpdateExisting {
                asserted_aspects, ..
            }
            | WorthQueryBackendAdmissibleMutationShape::DeleteExisting {
                asserted_aspects, ..
            } => asserted_aspects,
            _ => &[],
        }
    }

    pub fn symbolic_aspect_references(&self) -> &[WorthQuerySymbolicAspectReference] {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::Insert {
                symbolic_aspect_references,
                ..
            }
            | WorthQueryBackendAdmissibleMutationShape::UpdateExisting {
                symbolic_aspect_references,
                ..
            } => symbolic_aspect_references,
            _ => &[],
        }
    }

    pub fn admitted_touched_aspects(&self) -> &[WorthQueryAspectTouch] {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::DeleteDirect {
                touched_aspects, ..
            }
            | WorthQueryBackendAdmissibleMutationShape::DeleteExisting {
                touched_aspects, ..
            }
            | WorthQueryBackendAdmissibleMutationShape::DeleteSymbolic {
                touched_aspects, ..
            } => touched_aspects,
            _ => &[],
        }
    }

    pub fn declared_aspect_touches(&self) -> Vec<WorthQueryAspectTouch> {
        self.declared_aspect_operations()
            .into_iter()
            .map(|operation| operation.aspect_touch().clone())
            .collect()
    }

    pub fn declared_aspect_operations(&self) -> Vec<WorthQueryAspectMutationOperation> {
        let patch = self.authoritative_patch();
        patch
            .whole_aspect_sets()
            .map(|(key, _)| {
                WorthQueryAspectMutationOperation::from_touch(
                    WorthQueryAspectTouch::whole_aspect(key.clone()),
                    WorthQueryAspectMutationOperationKind::Set,
                )
            })
            .chain(patch.whole_aspect_clears().map(|key| {
                WorthQueryAspectMutationOperation::clear(WorthQueryAspectTouch::whole_aspect(
                    key.clone(),
                ))
            }))
            .chain(patch.field_patches().flat_map(|(key, field_patch)| {
                field_patch
                    .field_sets()
                    .map(|(field, _)| {
                        WorthQueryAspectMutationOperation::from_touch(
                            WorthQueryAspectTouch::aspect_field_path(
                                key.clone(),
                                worth_foundational::facade::CanonicalFieldPath::single(
                                    field.clone(),
                                ),
                            ),
                            WorthQueryAspectMutationOperationKind::Set,
                        )
                    })
                    .chain(field_patch.field_clears().map(|field| {
                        WorthQueryAspectMutationOperation::clear(
                            WorthQueryAspectTouch::aspect_field_path(
                                key.clone(),
                                worth_foundational::facade::CanonicalFieldPath::single(
                                    field.clone(),
                                ),
                            ),
                        )
                    }))
                    .collect::<Vec<_>>()
            }))
            .chain(self.symbolic_aspect_references().iter().map(|reference| {
                WorthQueryAspectMutationOperation::from_touch(
                    reference.aspect_touch().clone(),
                    WorthQueryAspectMutationOperationKind::Set,
                )
            }))
            .collect()
    }

    pub fn mutation_metadata_ref(&self) -> Option<&WorthQueryMutationMetadata> {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::Insert { metadata, .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateDirect { metadata, .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateExisting { metadata, .. }
            | WorthQueryBackendAdmissibleMutationShape::Assertion { metadata, .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateSymbolic { metadata, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteDirect { metadata, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteExisting { metadata, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteSymbolic { metadata, .. } => {
                Some(metadata)
            }
        }
    }

    pub fn mutation_metadata(&self) -> WorthQueryMutationMetadata {
        self.mutation_metadata_ref().cloned().unwrap_or_default()
    }

    pub fn naming_intent(&self) -> Option<&WorthQueryNamingMutationIntent> {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::Insert { naming_intent, .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateDirect { naming_intent, .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateExisting { naming_intent, .. }
            | WorthQueryBackendAdmissibleMutationShape::UpdateSymbolic { naming_intent, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteDirect { naming_intent, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteExisting { naming_intent, .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteSymbolic { naming_intent, .. } => {
                naming_intent.as_ref()
            }
            WorthQueryBackendAdmissibleMutationShape::Assertion { .. } => None,
        }
    }

    pub fn continuity_intent(&self) -> Option<&WorthQueryContinuityMutationIntent> {
        match &self.shape {
            WorthQueryBackendAdmissibleMutationShape::Insert {
                continuity_intent, ..
            }
            | WorthQueryBackendAdmissibleMutationShape::UpdateDirect {
                continuity_intent, ..
            }
            | WorthQueryBackendAdmissibleMutationShape::UpdateExisting {
                continuity_intent, ..
            }
            | WorthQueryBackendAdmissibleMutationShape::UpdateSymbolic {
                continuity_intent, ..
            } => continuity_intent.as_ref(),
            WorthQueryBackendAdmissibleMutationShape::Assertion { .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteDirect { .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteExisting { .. }
            | WorthQueryBackendAdmissibleMutationShape::DeleteSymbolic { .. } => None,
        }
    }
}
