use serde::{Deserialize, Serialize};
use worth_foundational::facade::{
    AspectKey, AspectLocator, AspectValue, AspectValueLocator, LocatorAuthority,
};

use crate::aspect_wire::serde_canonical_aspect_value_locator;
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeValueMaterialization {
    EqualityWitnessDigest,
    SnapshotPinnedRead,
    InternedCanonicalValueHandle,
    EagerInlineAspectValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeValueSourceSide {
    Source,
    Target,
    Base,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterializedAspectValue {
    pub policy: MergeValueMaterialization,
    pub evidence: MaterializedAspectValueEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterializedAspectValueEvidence {
    EqualityWitnessDigest(String),
    PinnedVisibleAspect {
        side: MergeValueSourceSide,
        record: RecordRef,
        #[serde(with = "serde_canonical_aspect_value_locator")]
        locator: AspectValueLocator,
    },
    InlineAspectValue(
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_value")] AspectValue,
    ),
}

pub fn aspect_reference(
    side: MergeValueSourceSide,
    record: RecordRef,
    aspect_key: AspectKey,
) -> MaterializedAspectValue {
    MaterializedAspectValue {
        policy: MergeValueMaterialization::SnapshotPinnedRead,
        evidence: MaterializedAspectValueEvidence::PinnedVisibleAspect {
            side,
            record,
            locator: authoritative_whole_aspect_value_locator(aspect_key),
        },
    }
}

pub fn materialized_value_aspect_key(locator: &AspectValueLocator) -> &AspectKey {
    match locator {
        AspectValueLocator::WholeAspect(aspect) => aspect.aspect_key(),
        AspectValueLocator::StructField(field) => field.aspect().aspect_key(),
    }
}

fn authoritative_whole_aspect_value_locator(aspect_key: AspectKey) -> AspectValueLocator {
    AspectValueLocator::whole_aspect(AspectLocator::new(
        LocatorAuthority::Authoritative,
        aspect_key,
    ))
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{
        AspectFieldLocator, CanonicalFieldPath, FieldKey, LocatorAuthority,
    };

    use super::*;
    use crate::identity::data::{EntityId, PartitionId};

    #[test]
    fn pinned_visible_aspect_materialization_roundtrips_locator_as_canonical_bytes() {
        let locator = AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            AspectKey::new("deploy.config").expect("valid aspect key"),
            CanonicalFieldPath::new(vec![FieldKey::new("replicas").expect("valid field key")])
                .expect("valid field path"),
        );
        let value = MaterializedAspectValueEvidence::PinnedVisibleAspect {
            side: MergeValueSourceSide::Source,
            record: RecordRef::Entity(EntityId::new(PartitionId::main(), 7, 0)),
            locator: AspectValueLocator::struct_field(locator.clone()),
        };

        let MaterializedAspectValueEvidence::PinnedVisibleAspect {
            locator: materialized_locator,
            ..
        } = value
        else {
            panic!("expected pinned visible aspect evidence");
        };

        let encoded = crate::aspect_wire::encode_aspect_value_locator(&materialized_locator);
        let decoded =
            crate::aspect_wire::decode_aspect_value_locator(&encoded).expect("decode locator");

        assert_eq!(decoded, AspectValueLocator::struct_field(locator));
    }
}
