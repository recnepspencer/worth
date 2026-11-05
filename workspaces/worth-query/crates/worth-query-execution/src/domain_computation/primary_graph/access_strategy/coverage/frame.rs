//! The receiver compares actual borrowed invocation objects, not reconstructed IDs.
use std::{any::Any, sync::Arc};
use worth_foundational::facade::AspectValue;
use worth_proof::BorrowedManagedCoverageFrame as Borrowed;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture as MatchPosture;
use worth_relational::facade::runtime::{ManagedIndexSourceObservation, ManagedIndexSourceRead};
use {
    crate::domain_computation::primary_graph::application_query::managed_index::SourceImageView,
    crate::domain_computation::primary_graph::vocabulary::application_query::invariant_read::ReadPosture,
    crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
};

#[derive(Clone, Copy)]
enum Audience<'a, 'loan> {
    Public(Borrowed<'a, WorthQueryManagedGraphWorkSession>),
    Committed(Borrowed<'a, ManagedIndexSourceRead<'loan>>),
    Proposed(Borrowed<'a, ManagedIndexSourceRead<'loan>>),
    Retained {
        frame: Borrowed<'a, super::super::realized_footprint::CapturedManagedInfluence>,
        observation: &'a ManagedIndexSourceObservation,
    },
}
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct CoverageFrame<'a, 'loan> {
    pub(in crate::domain_computation::primary_graph) image: SourceImageView<'a>,
    pub(in crate::domain_computation::primary_graph) nodes: &'a Arc<dyn Any + Send + Sync>,
    pub(in crate::domain_computation::primary_graph) probe: &'a AspectValue,
    pub(in crate::domain_computation::primary_graph) posture: MatchPosture,
    audience: Audience<'a, 'loan>,
}
impl<'a, 'loan> CoverageFrame<'a, 'loan> {
    /// Private consistency execution uses its actual target Source observation;
    /// it cannot impersonate a public READ or committed/proposed invariant loan.
    pub(in crate::domain_computation::primary_graph) fn retained(
        image: SourceImageView<'a>,
        nodes: &'a Arc<dyn Any + Send + Sync>,
        source: &'a dyn Any,
        probe: &'a AspectValue,
        observation: &'a ManagedIndexSourceObservation,
        retained: &'a super::super::realized_footprint::CapturedManagedInfluence,
        posture: MatchPosture,
    ) -> Self {
        Self {
            image,
            nodes,
            probe,
            posture,
            audience: Audience::Retained {
                frame: Borrowed::new(nodes, source, probe, retained),
                observation,
            },
        }
    }
    pub(in crate::domain_computation::primary_graph) fn public(
        image: SourceImageView<'a>,
        nodes: &'a Arc<dyn Any + Send + Sync>,
        source: &'a dyn Any,
        probe: &'a AspectValue,
        parent: &'a WorthQueryManagedGraphWorkSession,
        posture: MatchPosture,
    ) -> Self {
        Self {
            image,
            nodes,
            probe,
            posture,
            audience: Audience::Public(Borrowed::new(nodes, source, probe, parent)),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn invariant(
        image: SourceImageView<'a>,
        nodes: &'a Arc<dyn Any + Send + Sync>,
        source: &'a dyn Any,
        probe: &'a AspectValue,
        read: &'a ManagedIndexSourceRead<'loan>,
        posture: ReadPosture,
        match_posture: MatchPosture,
    ) -> Self {
        let borrowed = Borrowed::new(nodes, source, probe, read);
        Self {
            image,
            nodes,
            probe,
            posture: match_posture,
            audience: match posture {
                ReadPosture::Committed => Audience::Committed(borrowed),
                ReadPosture::Proposed => Audience::Proposed(borrowed),
            },
        }
    }
    pub(super) fn same_invocation(&self, other: &Self) -> bool {
        self.posture == other.posture
            && self.image.same_image(&other.image)
            && match (self.audience, other.audience) {
                (Audience::Public(a), Audience::Public(b)) => a.same_invocation(&b),
                (Audience::Committed(a), Audience::Committed(b))
                | (Audience::Proposed(a), Audience::Proposed(b)) => a.same_invocation(&b),
                (
                    Audience::Retained {
                        frame: a,
                        observation: x,
                    },
                    Audience::Retained {
                        frame: b,
                        observation: y,
                    },
                ) => a.same_invocation(&b) && std::ptr::eq(x, y),
                _ => false,
            }
    }
    pub(super) fn requires_complete(&self) -> bool {
        self.posture == MatchPosture::Refined || !matches!(self.audience, Audience::Public(_))
    }
}
