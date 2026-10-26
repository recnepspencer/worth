use worth_foundational::facade::CanonicalDigestId;

use crate::admission_digest::canonical_hash_parts;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadAccessShapeDigest {
    canonical: CanonicalDigestId,
    rendered: String,
}

impl WorthQueryGraphReadAccessShapeDigest {
    pub fn as_str(&self) -> &str {
        &self.rendered
    }

    pub fn canonical_digest(&self) -> &CanonicalDigestId {
        &self.canonical
    }

    pub fn from_parts(parts: &[String]) -> Self {
        let canonical = canonical_hash_parts(parts);
        let rendered = canonical.render_hex();
        Self {
            canonical,
            rendered,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadRootPosture {
    Local,
    Anchored,
    ExplicitBroadSearch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadTraversalOperator {
    DirectEdge,
    CompleteClosure,
    SuccessorWalk,
    BoundedAncestor,
    BoundedDescendant,
    AnchoredFrontier,
    SharedEndpoint,
    SharedAttachment,
    FrontierSearch,
    DeclarationTraversal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadFanoutPosture {
    None,
    SingleRelation,
    MultiRelation,
    Frontier,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadPredicateFamily {
    None,
    Equality,
    Range,
    Text,
    Membership,
    Presence,
    Mixed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadOrderingPosture {
    Unordered,
    ProviderOrdered,
    BoundedProjectedCollection,
    IndexedRelatedCollectionSeek,
    Mixed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadResultPressure {
    Detail,
    CollectionNarrow,
    CollectionWide,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadLifecycleClass {
    ReusableReadFamily,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryAdmittedGraphReadRelationDirection {
    Forward,
    Ancestor,
    Descendant,
}

macro_rules! string_vocabulary {
    ($type:ty, {$($variant:path => $name:literal),+ $(,)?}) => {
        impl $type {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $($variant => $name),+
                }
            }
        }
    };
}

string_vocabulary!(WorthQueryGraphReadRootPosture, {
    WorthQueryGraphReadRootPosture::Local => "local",
    WorthQueryGraphReadRootPosture::Anchored => "anchored",
    WorthQueryGraphReadRootPosture::ExplicitBroadSearch => "explicit_broad_search",
});
string_vocabulary!(WorthQueryGraphReadTraversalOperator, {
    WorthQueryGraphReadTraversalOperator::DirectEdge => "direct_edge",
    WorthQueryGraphReadTraversalOperator::CompleteClosure => "complete_closure",
    WorthQueryGraphReadTraversalOperator::SuccessorWalk => "successor_walk",
    WorthQueryGraphReadTraversalOperator::BoundedAncestor => "bounded_ancestor",
    WorthQueryGraphReadTraversalOperator::BoundedDescendant => "bounded_descendant",
    WorthQueryGraphReadTraversalOperator::AnchoredFrontier => "anchored_frontier",
    WorthQueryGraphReadTraversalOperator::SharedEndpoint => "shared_endpoint",
    WorthQueryGraphReadTraversalOperator::SharedAttachment => "shared_attachment",
    WorthQueryGraphReadTraversalOperator::FrontierSearch => "frontier_search",
    WorthQueryGraphReadTraversalOperator::DeclarationTraversal => "declaration_traversal",
});
string_vocabulary!(WorthQueryGraphReadFanoutPosture, {
    WorthQueryGraphReadFanoutPosture::None => "none",
    WorthQueryGraphReadFanoutPosture::SingleRelation => "single_relation",
    WorthQueryGraphReadFanoutPosture::MultiRelation => "multi_relation",
    WorthQueryGraphReadFanoutPosture::Frontier => "frontier",
});
string_vocabulary!(WorthQueryGraphReadPredicateFamily, {
    WorthQueryGraphReadPredicateFamily::None => "none",
    WorthQueryGraphReadPredicateFamily::Equality => "equality",
    WorthQueryGraphReadPredicateFamily::Range => "range",
    WorthQueryGraphReadPredicateFamily::Text => "text",
    WorthQueryGraphReadPredicateFamily::Membership => "membership",
    WorthQueryGraphReadPredicateFamily::Presence => "presence",
    WorthQueryGraphReadPredicateFamily::Mixed => "mixed",
});
string_vocabulary!(WorthQueryGraphReadOrderingPosture, {
    WorthQueryGraphReadOrderingPosture::Unordered => "unordered",
    WorthQueryGraphReadOrderingPosture::ProviderOrdered => "provider_ordered",
    WorthQueryGraphReadOrderingPosture::BoundedProjectedCollection => "bounded_projected_collection",
    WorthQueryGraphReadOrderingPosture::IndexedRelatedCollectionSeek => "indexed_related_collection_seek",
    WorthQueryGraphReadOrderingPosture::Mixed => "mixed",
});
string_vocabulary!(WorthQueryGraphReadResultPressure, {
    WorthQueryGraphReadResultPressure::Detail => "detail",
    WorthQueryGraphReadResultPressure::CollectionNarrow => "collection_narrow",
    WorthQueryGraphReadResultPressure::CollectionWide => "collection_wide",
});
string_vocabulary!(WorthQueryGraphReadLifecycleClass, {
    WorthQueryGraphReadLifecycleClass::ReusableReadFamily => "reusable_read_family",
});
string_vocabulary!(WorthQueryAdmittedGraphReadRelationDirection, {
    WorthQueryAdmittedGraphReadRelationDirection::Forward => "forward",
    WorthQueryAdmittedGraphReadRelationDirection::Ancestor => "ancestor",
    WorthQueryAdmittedGraphReadRelationDirection::Descendant => "descendant",
});
