use crate::merge::data::VisibleMergeRecordKind;
use worth_foundational::facade::AspectKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScalarPolicyBindingDenial {
    MissingBinding,
    InvalidBuiltInPolicyValueShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScalarPolicyAspectBinding {
    Entity { aspect_key: AspectKey },
    Relation { aspect_key: AspectKey },
}

impl ScalarPolicyAspectBinding {
    pub(crate) fn entity(
        record_kind: VisibleMergeRecordKind,
        aspect_key: AspectKey,
    ) -> Result<Self, ScalarPolicyBindingDenial> {
        if record_kind != VisibleMergeRecordKind::Entity {
            return Err(ScalarPolicyBindingDenial::MissingBinding);
        }
        Ok(Self::Entity { aspect_key })
    }

    pub(crate) fn relation(
        record_kind: VisibleMergeRecordKind,
        aspect_key: AspectKey,
    ) -> Result<Self, ScalarPolicyBindingDenial> {
        if record_kind != VisibleMergeRecordKind::Relation {
            return Err(ScalarPolicyBindingDenial::MissingBinding);
        }
        Ok(Self::Relation { aspect_key })
    }

    pub(crate) fn aspect_key(&self) -> &AspectKey {
        match self {
            Self::Entity { aspect_key } | Self::Relation { aspect_key } => aspect_key,
        }
    }
}
