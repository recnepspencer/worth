use crate::identity::data::KindId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalAuthorizationPlanDenial {
    NoPaths,
    PathStartsAtWrongKind {
        path: usize,
        expected: KindId,
        actual: KindId,
    },
    DiscontinuousTraversal {
        path: usize,
        traversal: usize,
        expected: KindId,
        actual: KindId,
    },
    PathEndsAtWrongKind {
        path: usize,
        expected: KindId,
        actual: KindId,
    },
    PredicateOutsidePath {
        path: usize,
        ordinal: usize,
        traversals: usize,
    },
    PredicateTargetsWrongKind {
        path: usize,
        ordinal: usize,
        expected: KindId,
        actual: KindId,
    },
    PredicateFieldPathNotSingle {
        path: usize,
        ordinal: usize,
        fields: usize,
    },
    FieldConstraintOutsidePath {
        path: usize,
        constraint: usize,
        ordinal: usize,
        traversals: usize,
    },
    FieldConstraintTargetsWrongKind {
        path: usize,
        constraint: usize,
        ordinal: usize,
        expected: KindId,
        actual: KindId,
    },
    FieldConstraintPathNotSingle {
        path: usize,
        constraint: usize,
        ordinal: usize,
        fields: usize,
    },
    EntityAnchorOutsidePath {
        path: usize,
        ordinal: usize,
        traversals: usize,
    },
    EntityAnchorTargetsWrongKind {
        path: usize,
        ordinal: usize,
        expected: KindId,
        actual: KindId,
    },
    RelatedEntityOutsidePath {
        path: usize,
        ordinal: usize,
        traversals: usize,
    },
    RelatedEntityStartsAtWrongKind {
        path: usize,
        ordinal: usize,
        expected: KindId,
        actual: KindId,
    },
    ExactAdjacencyOutsidePath {
        path: usize,
        ordinal: usize,
        traversals: usize,
    },
    ExactAdjacencyStartsAtWrongKind {
        path: usize,
        ordinal: usize,
        expected: KindId,
        actual: KindId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalAuthorizationObservationDenial {
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    SnapshotUnavailable,
    PrincipalUnavailableOrWrongKind,
    ScopeUnavailableOrWrongKind,
    ObservationIdentityExhausted,
    PreparationAccountingOverflow,
}
