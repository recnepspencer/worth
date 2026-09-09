use worth_foundational::facade::{
    CanonicalFieldPath, PortableAspectPatchOperation, PortableRecordAspectPatch,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    ApplyEntityAspectPatchIntent, CreateIntent, EntityAspectCreateIntent, EntityMutationIntent,
    MutationIntent, WorkerIntentBatch,
};

use crate::runtime::WorthQueryAspectTouch;

use super::{
    WorthQueryEntityIdentity, WorthQueryMemoryWorkspace, WorthQueryMutationKind,
    WorthQueryMutationReceipt, WorthQueryWorkspaceError,
};

impl WorthQueryMemoryWorkspace {
    pub(super) fn portable_creation_patch_from_authored(
        &self,
        aspects: &[crate::runtime::WorthQueryAuthoredAspectMutation],
    ) -> Result<PortableRecordAspectPatch, WorthQueryWorkspaceError> {
        self.portable_patch_from_authored(aspects, |aspects, registry| {
            crate::runtime::native_aspect_contracts::admit_authored_creation_patch(
                aspects, registry,
            )
        })
    }

    pub(super) fn portable_mutation_patch_from_authored(
        &self,
        aspects: &[crate::runtime::WorthQueryAuthoredAspectMutation],
    ) -> Result<PortableRecordAspectPatch, WorthQueryWorkspaceError> {
        self.portable_patch_from_authored(aspects, |aspects, registry| {
            crate::runtime::native_aspect_contracts::admit_authored_mutation_patch(
                aspects, registry,
            )
        })
    }

    fn portable_patch_from_authored(
        &self,
        aspects: &[crate::runtime::WorthQueryAuthoredAspectMutation],
        admit: impl FnOnce(
            &[crate::runtime::WorthQueryAuthoredAspectMutation],
            &crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry,
        ) -> Result<
            worth_foundational::facade::AuthoritativeRecordAspectPatch,
            crate::runtime::WorthQueryMutationContractDenial,
        >,
    ) -> Result<PortableRecordAspectPatch, WorthQueryWorkspaceError> {
        let aspects = aspects
            .iter()
            .map(|aspect| self.normalize_scalar_field_alias(aspect))
            .collect::<Result<Vec<_>, _>>()?;
        let authoritative = admit(&aspects, &self.aspect_contracts)
            .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
        worth_foundational::facade::export_portable_record_aspect_patch(
            &authoritative,
            &self.aspect_contracts,
        )
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))
    }

    fn normalize_scalar_field_alias(
        &self,
        authored: &crate::runtime::WorthQueryAuthoredAspectMutation,
    ) -> Result<crate::runtime::WorthQueryAuthoredAspectMutation, WorthQueryWorkspaceError> {
        let touch = authored.aspect_touch();
        let Some(field_path) = touch.native_field_path() else {
            return Ok(authored.clone());
        };
        let Some(contract) = self.aspect_contracts.contract(touch.native_aspect_key()) else {
            return Ok(authored.clone());
        };
        if !matches!(
            contract.shape(),
            worth_foundational::facade::AspectShape::Scalar(_)
        ) {
            return Ok(authored.clone());
        }
        let declared = self.aspects.iter().any(|declared| {
            declared.aspect_touch().native_aspect_key() == touch.native_aspect_key()
                && declared.native_field_path().fields().get(1..) == Some(field_path.fields())
        });
        if !declared {
            return Err(WorthQueryWorkspaceError::new(format!(
                "scalar aspect field alias `{}` does not match its declared native field path",
                touch.admitted_touch_digest_part()
            )));
        }
        let whole = WorthQueryAspectTouch::whole_aspect(touch.native_aspect_key().clone());
        if authored.clears_existing_value() {
            return crate::runtime::WorthQueryAuthoredAspectMutation::new_clear(whole);
        }
        crate::runtime::WorthQueryAuthoredAspectMutation::new_set(
            whole,
            authored.validation_input().cloned().ok_or_else(|| {
                WorthQueryWorkspaceError::new("scalar aspect set has no validation input")
            })?,
        )
    }

    pub(crate) fn insert_portable_patch(
        &mut self,
        patch: &PortableRecordAspectPatch,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        self.insert_portable_patch_with_touches(patch, touches_from_portable_patch(patch))
    }

    pub(super) fn insert_portable_patch_with_touches(
        &mut self,
        patch: &PortableRecordAspectPatch,
        touches: Vec<WorthQueryAspectTouch>,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        self.next_client_key += 1;
        let client_key = ClientKey::raw(format!("{}:{}", self.kind_name, self.next_client_key));
        let intent =
            MutationIntent::Create(CreateIntent::EntityAspects(EntityAspectCreateIntent {
                partition_id: worth_relational::facade::identity::PartitionId::main(),
                kind_id: self.kind_id,
                client_key,
                aspect_patch: patch.clone(),
            }));
        self.commit_portable_patch(intent, WorthQueryMutationKind::Created, touches)
    }

    pub(crate) fn update_portable_patch(
        &mut self,
        entity_identity: WorthQueryEntityIdentity,
        patch: &PortableRecordAspectPatch,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        self.update_portable_patch_with_touches(
            entity_identity,
            patch,
            touches_from_portable_patch(patch),
        )
    }

    pub(super) fn update_portable_patch_with_touches(
        &mut self,
        entity_identity: WorthQueryEntityIdentity,
        patch: &PortableRecordAspectPatch,
        touches: Vec<WorthQueryAspectTouch>,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        let entity_id = super::runtime_identity::entity_id_from_identity(entity_identity)?;
        self.ensure_entity_exists(entity_id)?;
        let intent = MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
            ApplyEntityAspectPatchIntent {
                entity_id,
                aspect_patch: patch.clone(),
            },
        ));
        self.commit_portable_patch(intent, WorthQueryMutationKind::Updated, touches)
    }

    fn commit_portable_patch(
        &mut self,
        intent: MutationIntent,
        mutation_kind: WorthQueryMutationKind,
        touches: Vec<WorthQueryAspectTouch>,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        let (result, snapshot) = self.commit_batch(
            WorkerIntentBatch::new("query-memory-native-aspect-patch").push(intent),
        )?;
        let receipt = self.receipt_from_commit(result, snapshot, mutation_kind, touches);
        Ok(receipt)
    }
}

fn touches_from_portable_patch(patch: &PortableRecordAspectPatch) -> Vec<WorthQueryAspectTouch> {
    patch
        .operations()
        .iter()
        .flat_map(|operation| match operation {
            PortableAspectPatchOperation::SetWhole { basis, .. }
            | PortableAspectPatchOperation::ClearWhole { basis } => {
                vec![WorthQueryAspectTouch::whole_aspect(basis.key().clone())]
            }
            PortableAspectPatchOperation::PatchFields {
                basis,
                field_sets,
                field_clears,
                ..
            } => field_sets
                .iter()
                .map(|set| set.field().clone())
                .chain(field_clears.iter().cloned())
                .map(|field| {
                    WorthQueryAspectTouch::aspect_field_path(
                        basis.key().clone(),
                        CanonicalFieldPath::single(field),
                    )
                })
                .collect(),
        })
        .collect()
}
