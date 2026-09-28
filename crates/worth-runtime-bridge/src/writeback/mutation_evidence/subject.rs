use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_foundational::facade::{AspectKey, AuthoritativeRecordAspectPatch, CanonicalFieldPath};

use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;
use crate::writeback::BridgeWritebackEffectIntent;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeMutationSubjectKind {
    Created,
    Updated,
    Deleted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeMutationSubjectTouch {
    aspect_key: AspectKey,
    field_path: Option<CanonicalFieldPath>,
}

impl BridgeMutationSubjectTouch {
    pub fn whole_aspect(aspect_key: AspectKey) -> Self {
        Self {
            aspect_key,
            field_path: None,
        }
    }

    pub fn aspect_field_path(aspect_key: AspectKey, field_path: CanonicalFieldPath) -> Self {
        Self {
            aspect_key,
            field_path: Some(field_path),
        }
    }

    fn canonical_key(&self) -> String {
        let field_path = self
            .field_path
            .as_ref()
            .map(|path| {
                path.fields()
                    .iter()
                    .map(|field| field.as_str())
                    .collect::<Vec<_>>()
                    .join(".")
            })
            .unwrap_or_else(|| "<whole-aspect>".to_string());
        format!("{}:{field_path}", self.aspect_key.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeMutationSubject {
    target_collection: Arc<str>,
    target_record: RelationalBridgeRecordIdentityParts,
    mutation_kind: BridgeMutationSubjectKind,
    touches: Vec<BridgeMutationSubjectTouch>,
    effect_intent_patch_canonical_basis: Arc<str>,
    digest: Arc<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeMutationSubjectTarget {
    target_collection: Arc<str>,
    target_record: RelationalBridgeRecordIdentityParts,
    mutation_kind: BridgeMutationSubjectKind,
}

impl BridgeMutationSubjectTarget {
    pub fn new(
        target_collection: impl Into<Arc<str>>,
        target_record: RelationalBridgeRecordIdentityParts,
        mutation_kind: BridgeMutationSubjectKind,
    ) -> Self {
        Self {
            target_collection: target_collection.into(),
            target_record,
            mutation_kind,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeMutationSubjectError {
    DuplicateDeclaredTouch,
    ConcretePatchTouchMissing,
}

impl std::fmt::Display for BridgeMutationSubjectError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateDeclaredTouch => {
                formatter.write_str("bridge mutation subject contains a duplicate declared touch")
            }
            Self::ConcretePatchTouchMissing => formatter.write_str(
                "bridge mutation subject omits a touch present in the authoritative patch",
            ),
        }
    }
}

impl std::error::Error for BridgeMutationSubjectError {}

impl BridgeMutationSubject {
    pub fn from_effect_intent_and_touches(
        target: BridgeMutationSubjectTarget,
        effect_intent: &BridgeWritebackEffectIntent,
        declared_touches: impl IntoIterator<Item = BridgeMutationSubjectTouch>,
    ) -> Result<Self, BridgeMutationSubjectError> {
        let declared_touches = declared_touches.into_iter().collect::<Vec<_>>();
        let touches = canonical_touches(declared_touches.iter().cloned());
        if touches.len() != declared_touches.len() {
            return Err(BridgeMutationSubjectError::DuplicateDeclaredTouch);
        }
        if canonical_patch_touches(effect_intent.authoritative_patch())
            .iter()
            .any(|patch_touch| !touches.contains(patch_touch))
        {
            return Err(BridgeMutationSubjectError::ConcretePatchTouchMissing);
        }
        let digest =
            mutation_subject_digest(&target, &touches, effect_intent.patch_canonical_basis());
        Ok(Self {
            target_collection: target.target_collection,
            target_record: target.target_record,
            mutation_kind: target.mutation_kind,
            touches,
            effect_intent_patch_canonical_basis: Arc::from(
                effect_intent.patch_canonical_basis().to_owned(),
            ),
            digest,
        })
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub fn matches_projection(
        &self,
        target_collection: &str,
        target_record: RelationalBridgeRecordIdentityParts,
        mutation_kind: BridgeMutationSubjectKind,
        touches: &[BridgeMutationSubjectTouch],
    ) -> bool {
        let canonical_touches = canonical_touches(touches.iter().cloned());
        self.target_collection.as_ref() == target_collection
            && self.target_record == target_record
            && self.mutation_kind == mutation_kind
            && canonical_touches.len() == touches.len()
            && self.touches == canonical_touches
    }

    pub(crate) fn matches_effect_intent(
        &self,
        effect_intent: &BridgeWritebackEffectIntent,
    ) -> bool {
        self.effect_intent_patch_canonical_basis.as_ref() == effect_intent.patch_canonical_basis()
    }
}

fn canonical_patch_touches(
    patch: &AuthoritativeRecordAspectPatch,
) -> Vec<BridgeMutationSubjectTouch> {
    canonical_touches(
        patch
            .whole_aspect_sets()
            .map(|(key, _)| BridgeMutationSubjectTouch::whole_aspect(key.clone()))
            .chain(
                patch
                    .whole_aspect_clears()
                    .map(|key| BridgeMutationSubjectTouch::whole_aspect(key.clone())),
            )
            .chain(patch.field_patches().flat_map(|(aspect_key, fields)| {
                fields
                    .field_sets()
                    .map(|(field, _)| field.clone())
                    .chain(fields.field_clears().cloned())
                    .map(|field| {
                        let field_path = CanonicalFieldPath::new([field])
                            .expect("authoritative patch fields form a non-empty field path");
                        BridgeMutationSubjectTouch::aspect_field_path(
                            aspect_key.clone(),
                            field_path,
                        )
                    })
            })),
    )
}

fn canonical_touches(
    touches: impl IntoIterator<Item = BridgeMutationSubjectTouch>,
) -> Vec<BridgeMutationSubjectTouch> {
    let mut touches = touches.into_iter().collect::<Vec<_>>();
    touches.sort_by_key(BridgeMutationSubjectTouch::canonical_key);
    touches.dedup();
    touches
}

fn mutation_subject_digest(
    target: &BridgeMutationSubjectTarget,
    touches: &[BridgeMutationSubjectTouch],
    effect_intent_patch_canonical_basis: &str,
) -> Arc<str> {
    let mut hasher = Sha256::new();
    hasher.update(b"bridge-mutation-subject-v1\0");
    hasher.update(target.target_collection.as_bytes());
    hasher.update(b"\0");
    hasher.update(record_kind_label(target.target_record.kind()).as_bytes());
    hasher.update(b"\0");
    hasher.update(target.target_record.partition_id().to_be_bytes());
    hasher.update(target.target_record.local_slot().to_be_bytes());
    hasher.update(target.target_record.generation().to_be_bytes());
    hasher.update(b"\0");
    hasher.update(mutation_kind_label(target.mutation_kind).as_bytes());
    hasher.update(b"\0patch-basis=");
    hasher.update(effect_intent_patch_canonical_basis.as_bytes());
    for touch in touches {
        hasher.update(b"\0");
        hasher.update(touch.canonical_key().as_bytes());
    }
    Arc::from(format!(
        "bridge-mutation-subject:sha256:{:x}",
        hasher.finalize()
    ))
}

const fn record_kind_label(
    kind: crate::relational_source::identity_parts::RelationalBridgeRecordIdentityKind,
) -> &'static str {
    match kind {
        crate::relational_source::identity_parts::RelationalBridgeRecordIdentityKind::Entity => {
            "entity"
        }
        crate::relational_source::identity_parts::RelationalBridgeRecordIdentityKind::Relation => {
            "relation"
        }
    }
}

const fn mutation_kind_label(kind: BridgeMutationSubjectKind) -> &'static str {
    match kind {
        BridgeMutationSubjectKind::Created => "created",
        BridgeMutationSubjectKind::Updated => "updated",
        BridgeMutationSubjectKind::Deleted => "deleted",
    }
}
