//! Physical owner denials remain typed until the concrete read adapter maps them.
use super::super::source_facts::SourceFactReadDenial;
use worth_relational::facade::{
    runtime::{BoundedFrontierReadStop, PinnedStructuralWalkDenialKind},
    transactions::RelationalExecutionDenialCause,
};
pub(in crate::domain_computation::primary_graph) enum RootPathReadDenial<E> {
    Adapter(E),
    Native {
        subject: String,
        cause: RelationalExecutionDenialCause,
    },
    Source {
        subject: String,
        cause: SourceFactReadDenial,
    },
    Walk {
        subject: String,
        kind: PinnedStructuralWalkDenialKind,
    },
    Traversal {
        subject: String,
    },
    FrontierLimit {
        subject: String,
    },
}
impl<E> RootPathReadDenial<E> {
    pub(in crate::domain_computation::primary_graph) fn native(
        subject: &str,
        cause: RelationalExecutionDenialCause,
    ) -> Self {
        Self::Native {
            subject: subject.to_owned(),
            cause,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn traversal(subject: &str) -> Self {
        Self::Traversal {
            subject: subject.to_owned(),
        }
    }
    pub(super) fn source(subject: &str, cause: SourceFactReadDenial) -> Self {
        Self::Source {
            subject: subject.to_owned(),
            cause,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn frontier<L>(
        subject: &str,
        cause: BoundedFrontierReadStop<L, worth_execution::WorkCeilingDenial>,
    ) -> Self {
        match cause {
            BoundedFrontierReadStop::Limit(_) => Self::FrontierLimit {
                subject: subject.to_owned(),
            },
            BoundedFrontierReadStop::Admission(stop) => Self::native(subject, stop.into()),
        }
    }
}
