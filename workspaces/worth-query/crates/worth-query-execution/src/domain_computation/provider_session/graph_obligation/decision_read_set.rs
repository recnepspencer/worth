use worth_query_admission::facade::graph_read_access::{
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadPlanReview,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadDependencyEvidence {
    predicate_and_negative_space: bool,
    ordering: bool,
    membership_and_traversal: bool,
    projection: bool,
    examined_candidates: usize,
    projected_records: usize,
    projected_fields: usize,
    relation_records_examined: usize,
    ordering_comparisons: usize,
}

pub(in crate::domain_computation) struct WorthQueryObservedGraphReadWork {
    pub examined_candidates: usize,
    pub projected_records: usize,
    pub projected_fields: usize,
    pub relation_records_examined: usize,
    pub ordering_comparisons: usize,
}

impl WorthQueryGraphReadDependencyEvidence {
    pub(super) fn bind(
        review: &WorthQueryGraphReadPlanReview,
        observed: WorthQueryObservedGraphReadWork,
    ) -> Self {
        let requirements = review.requirements();
        let requires = |kind| requirements.requires_kind(kind);
        Self {
            predicate_and_negative_space: requires(
                WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
            ),
            ordering: requires(WorthQueryGraphReadAccessRequirementKind::OrderingSupport),
            membership_and_traversal: [
                WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
                WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency,
                WorthQueryGraphReadAccessRequirementKind::TraversalWorkset,
                WorthQueryGraphReadAccessRequirementKind::VisitedSet,
                WorthQueryGraphReadAccessRequirementKind::DedupSet,
                WorthQueryGraphReadAccessRequirementKind::ProofSupport,
            ]
            .into_iter()
            .any(requires),
            projection: requires(WorthQueryGraphReadAccessRequirementKind::ResultBuffer)
                || requires(WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle),
            examined_candidates: observed.examined_candidates,
            projected_records: observed.projected_records,
            projected_fields: observed.projected_fields,
            relation_records_examined: observed.relation_records_examined,
            ordering_comparisons: observed.ordering_comparisons,
        }
    }

    pub const fn includes_predicate_and_negative_space(&self) -> bool {
        self.predicate_and_negative_space
    }

    pub const fn includes_ordering(&self) -> bool {
        self.ordering
    }

    pub const fn includes_membership_and_traversal(&self) -> bool {
        self.membership_and_traversal
    }

    pub const fn includes_projection(&self) -> bool {
        self.projection
    }

    pub const fn examined_candidates(&self) -> usize {
        self.examined_candidates
    }

    pub const fn projected_records(&self) -> usize {
        self.projected_records
    }

    pub const fn projected_fields(&self) -> usize {
        self.projected_fields
    }

    pub const fn relation_records_examined(&self) -> usize {
        self.relation_records_examined
    }

    pub const fn ordering_comparisons(&self) -> usize {
        self.ordering_comparisons
    }
}
