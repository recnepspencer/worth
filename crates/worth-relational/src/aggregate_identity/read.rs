//! Scoped, read-only aggregate values; graph records and write custody stay private.
use super::projection::AggregateProjection;
use super::proposal::{binding_error, AggregateInvocation};
use super::stored_read::StoredAggregateRead;
mod port;
use crate::identity::data::EntityId;
use crate::transactions::data::{CommitExecutionDenialKind, TransactionCommitError};
use crate::validation::data::{StructuralRelationView, StructuralWalkDenialKind};
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_foundational::facade::{
    AggregateDefinitionIdentity, AggregateIdentity, AggregateLineageKind,
};

/// A port borrowed from one Native custom planner/context view. It grants no
/// metadata access beyond aggregate values of the installed business graph.
pub struct AggregateReadPort<'view, 'runtime> {
    view: &'view StructuralRelationView<'runtime>,
}

/// Why a scoped aggregate read refused. A refusal never contains a partial group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AggregateReadDenial {
    Budget(StructuralWalkDenialKind),
    SourceUnavailable,
    NotAggregateRelation,
    WrongMemberKind,
    PreparationRefused,
    Execution(crate::execution::RelationalExecutionDenialCause),
}

/// Retained aggregate truth from one proposal or its committed before-image.
/// Members are in canonical identity order, not successor order. This value
/// holds the original admitted payer and contains no mutation authority.
#[derive(Clone)]
pub struct AggregateRead {
    backing: ReadBacking,
}
#[derive(Clone)]
enum ReadBacking {
    Proposed {
        projection: Arc<AggregateProjection>,
        definition: AggregateDefinitionIdentity,
        assignment: usize,
    },
    Stored(Arc<StoredAggregateRead>),
}

/// Borrow of an actual event retained by a Native aggregate read. The enclosing
/// reader distinguishes stored incident history from a proposed transition;
/// latest-identity resolution is not inferred from either event view.
#[derive(Clone, Copy)]
pub struct AggregateReadLineageEvent<'read> {
    event: &'read super::AggregateLineageEvent,
}

impl<'read> AggregateReadLineageEvent<'read> {
    pub(super) fn new(event: &'read super::AggregateLineageEvent) -> Self {
        Self { event }
    }
}

impl AggregateRead {
    pub fn definition(&self) -> AggregateDefinitionIdentity {
        match &self.backing {
            ReadBacking::Proposed {
                projection,
                definition,
                ..
            } => projection.data.definitions[definition].contract.definition,
            ReadBacking::Stored(stored) => stored.contract.definition,
        }
    }
    pub fn identity(&self) -> AggregateIdentity {
        match &self.backing {
            ReadBacking::Proposed {
                projection,
                definition,
                assignment,
            } => {
                projection.data.definitions[definition]
                    .changed
                    .as_ref()
                    .expect("proposed assignment")
                    .transition
                    .assignments[*assignment]
                    .identity
            }
            ReadBacking::Stored(stored) => stored.identity,
        }
    }
    pub fn anchor(&self) -> EntityId {
        *self
            .member_set()
            .first()
            .expect("complete nonempty aggregate")
    }
    pub fn len(&self) -> usize {
        self.member_set().len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    pub fn members(&self) -> impl ExactSizeIterator<Item = EntityId> + '_ {
        self.member_set().iter().copied()
    }

    /// Membership in this complete owned group, without another graph read.
    pub fn contains(&self, member: EntityId) -> bool {
        self.member_set().contains(&member)
    }

    /// Only events incident to this identity in the actual proposed transition.
    /// Committed/before-image reads contribute no new transition events.
    pub fn transition_lineage(&self) -> impl Iterator<Item = AggregateReadLineageEvent<'_>> {
        let events = match &self.backing {
            ReadBacking::Proposed {
                projection,
                definition,
                ..
            } => Some(
                &projection.data.definitions[definition]
                    .changed
                    .as_ref()
                    .expect("proposed assignment")
                    .transition
                    .lineage,
            ),
            ReadBacking::Stored(_) => None,
        };
        let identity = self.identity();
        events
            .into_iter()
            .flat_map(|events| events.iter())
            .filter(move |event| event.from == identity || event.to == identity)
            .map(|event| AggregateReadLineageEvent { event })
    }

    fn member_set(&self) -> &BTreeSet<EntityId> {
        match &self.backing {
            ReadBacking::Proposed {
                projection,
                definition,
                assignment,
            } => {
                &projection.data.definitions[definition]
                    .changed
                    .as_ref()
                    .expect("proposed assignment")
                    .transition
                    .assignments[*assignment]
                    .members
            }
            ReadBacking::Stored(stored) => &stored.members,
        }
    }
}

impl AggregateReadLineageEvent<'_> {
    pub fn occurrence(&self) -> crate::history::data::CommitId {
        self.event.occurrence
    }
    pub fn kind(&self) -> AggregateLineageKind {
        self.event.kind
    }
    pub fn from(&self) -> AggregateIdentity {
        self.event.from
    }
    pub fn to(&self) -> AggregateIdentity {
        self.event.to
    }
}

fn inconsistent(source: AggregateInvocation<'_>, detail: &str) -> AggregateReadDenial {
    source.latch_failure(binding_error(detail));
    AggregateReadDenial::PreparationRefused
}
fn from_preparation(error: TransactionCommitError) -> AggregateReadDenial {
    match error {
        TransactionCommitError::Execution { denial, .. } => {
            let CommitExecutionDenialKind::Cause(cause) = denial.kind;
            AggregateReadDenial::Execution(cause)
        }
        _ => AggregateReadDenial::PreparationRefused,
    }
}
impl std::fmt::Display for AggregateReadDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Native aggregate read refused: {self:?}")
    }
}
impl std::error::Error for AggregateReadDenial {}
impl std::fmt::Debug for AggregateRead {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AggregateRead")
            .field("definition", &self.definition())
            .field("identity", &self.identity())
            .field("anchor", &self.anchor())
            .field("members", &self.len())
            .finish_non_exhaustive()
    }
}
