#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorthQueryGraphReadAccessRequirementKind {
    DirectionalAdjacency,
    ReverseAdjacency,
    AggregateSelection,
    PredicateSupport,
    OrderingSupport,
    TraversalWorkset,
    VisitedSet,
    DedupSet,
    ProofSupport,
    ResultBuffer,
    MaterializationLifecycle,
    LiveMaintenanceSupport,
    DomainOperationCapabilityRegistration,
}

impl WorthQueryGraphReadAccessRequirementKind {
    pub fn all() -> &'static [Self] {
        &[
            Self::DirectionalAdjacency,
            Self::ReverseAdjacency,
            Self::AggregateSelection,
            Self::PredicateSupport,
            Self::OrderingSupport,
            Self::TraversalWorkset,
            Self::VisitedSet,
            Self::DedupSet,
            Self::ProofSupport,
            Self::ResultBuffer,
            Self::MaterializationLifecycle,
            Self::LiveMaintenanceSupport,
            Self::DomainOperationCapabilityRegistration,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DirectionalAdjacency => "directional_adjacency",
            Self::ReverseAdjacency => "reverse_adjacency",
            Self::AggregateSelection => "aggregate_selection",
            Self::PredicateSupport => "predicate_support",
            Self::OrderingSupport => "ordering_support",
            Self::TraversalWorkset => "traversal_workset",
            Self::VisitedSet => "visited_set",
            Self::DedupSet => "dedup_set",
            Self::ProofSupport => "proof_support",
            Self::ResultBuffer => "result_buffer",
            Self::MaterializationLifecycle => "materialization_lifecycle",
            Self::LiveMaintenanceSupport => "live_maintenance_support",
            Self::DomainOperationCapabilityRegistration => {
                "domain_operation_capability_registration"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorthQueryGraphReadAccessRebuildBasis {
    AuthoritativeRelationTruth,
    AuthoritativeFieldTruth,
    ReadGraphProof,
    OperationResolutionProof,
    SelectivityProof,
    RuntimeSupportRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadAccessInvalidationBasis {
    AuthoritativeRelationDelta,
    AuthoritativeFieldDelta,
    ReadGraphProofDelta,
    RuntimeLifecycleDelta,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadAccessComplexityContract {
    DirectionalRelationLookup,
    ReverseRelationLookup,
    BoundedTraversalWorkset,
    CompleteTraversalWorkset,
    CandidatePredicateSupport,
    CandidateOrderingSupport,
    ProofEvidenceSupport,
    ResultPressureBuffer,
    LifecycleSupportAdmission,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphReadAccessMemoryEstimateBasis {
    RelationDegreeBound,
    FrontierDepthBound,
    DiscoveredWorkset,
    PredicateCandidateSet,
    OrderedCandidateSet,
    ProofEvidenceSet,
    ResultPressureBound,
    LifecycleManagedSupport,
}

macro_rules! names {
    ($type:ty, {$($variant:path => $name:literal),+ $(,)?}) => {
        impl $type {
            pub fn as_str(&self) -> &'static str {
                match self { $($variant => $name),+ }
            }
        }
    };
}

names!(WorthQueryGraphReadAccessRebuildBasis, {
    WorthQueryGraphReadAccessRebuildBasis::AuthoritativeRelationTruth => "authoritative_relation_truth",
    WorthQueryGraphReadAccessRebuildBasis::AuthoritativeFieldTruth => "authoritative_field_truth",
    WorthQueryGraphReadAccessRebuildBasis::ReadGraphProof => "read_graph_proof",
    WorthQueryGraphReadAccessRebuildBasis::OperationResolutionProof => "operation_resolution_proof",
    WorthQueryGraphReadAccessRebuildBasis::SelectivityProof => "selectivity_proof",
    WorthQueryGraphReadAccessRebuildBasis::RuntimeSupportRequired => "runtime_support_required",
});
names!(WorthQueryGraphReadAccessInvalidationBasis, {
    WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeRelationDelta => "authoritative_relation_delta",
    WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeFieldDelta => "authoritative_field_delta",
    WorthQueryGraphReadAccessInvalidationBasis::ReadGraphProofDelta => "read_graph_proof_delta",
    WorthQueryGraphReadAccessInvalidationBasis::RuntimeLifecycleDelta => "runtime_lifecycle_delta",
});
names!(WorthQueryGraphReadAccessComplexityContract, {
    WorthQueryGraphReadAccessComplexityContract::DirectionalRelationLookup => "directional_relation_lookup",
    WorthQueryGraphReadAccessComplexityContract::ReverseRelationLookup => "reverse_relation_lookup",
    WorthQueryGraphReadAccessComplexityContract::BoundedTraversalWorkset => "bounded_traversal_workset",
    WorthQueryGraphReadAccessComplexityContract::CompleteTraversalWorkset => "complete_traversal_workset",
    WorthQueryGraphReadAccessComplexityContract::CandidatePredicateSupport => "candidate_predicate_support",
    WorthQueryGraphReadAccessComplexityContract::CandidateOrderingSupport => "candidate_ordering_support",
    WorthQueryGraphReadAccessComplexityContract::ProofEvidenceSupport => "proof_evidence_support",
    WorthQueryGraphReadAccessComplexityContract::ResultPressureBuffer => "result_pressure_buffer",
    WorthQueryGraphReadAccessComplexityContract::LifecycleSupportAdmission => "lifecycle_support_admission",
});
names!(WorthQueryGraphReadAccessMemoryEstimateBasis, {
    WorthQueryGraphReadAccessMemoryEstimateBasis::RelationDegreeBound => "relation_degree_bound",
    WorthQueryGraphReadAccessMemoryEstimateBasis::FrontierDepthBound => "frontier_depth_bound",
    WorthQueryGraphReadAccessMemoryEstimateBasis::DiscoveredWorkset => "discovered_workset",
    WorthQueryGraphReadAccessMemoryEstimateBasis::PredicateCandidateSet => "predicate_candidate_set",
    WorthQueryGraphReadAccessMemoryEstimateBasis::OrderedCandidateSet => "ordered_candidate_set",
    WorthQueryGraphReadAccessMemoryEstimateBasis::ProofEvidenceSet => "proof_evidence_set",
    WorthQueryGraphReadAccessMemoryEstimateBasis::ResultPressureBound => "result_pressure_bound",
    WorthQueryGraphReadAccessMemoryEstimateBasis::LifecycleManagedSupport => "lifecycle_managed_support",
});
