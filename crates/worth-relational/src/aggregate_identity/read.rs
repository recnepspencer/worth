//! Scoped, read-only aggregate values; graph records and write custody stay private.
use super::projection::AggregateProjection;
use super::proposal::{binding_error, AggregateInvocation};
use super::stored_read::StoredAggregateRead;
use crate::identity::data::{EntityId, KindId};
use crate::transactions::data::{CommitExecutionDenialKind, TransactionCommitError};
use crate::validation::data::{StructuralRelationView, StructuralWalkDenialKind};
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_foundational::facade::{
    AggregateDefinitionIdentity, AggregateIdentity, AggregateLineageKind,
};

/// A port borrowed from one Native custom planner/context view. It grants no
/// metadata access beyond aggregate values of the declared business relation.
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
        relation: KindId,
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

impl<'view, 'runtime> AggregateReadPort<'view, 'runtime> {
    pub(crate) fn new(view: &'view StructuralRelationView<'runtime>) -> Self {
        Self { view }
    }

    /// Read one aggregate of the installed stored source relation. Shared
    /// proposal preparation is paid once by the original Native request. A
    /// committed/unaffected group costs O((v+a) log(v+a)) per deliberate read,
    /// debiting this rule's existing allowance and the request once each.
    pub fn of_source_relation(
        &self,
        relation: KindId,
        member: EntityId,
    ) -> Result<AggregateRead, AggregateReadDenial> {
        let budget = self.view.read_budget();
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        self.view
            .require_relation_kind(relation)
            .map_err(|error| AggregateReadDenial::Budget(budget.refusal(error)))?;
        let actual_kind = self
            .view
            .entity_kind(member)
            .map_err(|error| AggregateReadDenial::Budget(budget.refusal(error)))?;
        let (state, execution, source) = self.view.aggregate_read_parts();
        let (source, committed) = source.ok_or(AggregateReadDenial::SourceUnavailable)?;
        budget
            .checkpoint(source.contract_count(committed).saturating_add(1))
            .map_err(AggregateReadDenial::Budget)?;
        let contract = source
            .contract(relation, committed)
            .ok_or(AggregateReadDenial::NotAggregateRelation)?;
        if actual_kind != contract.member_kind {
            return Err(AggregateReadDenial::WrongMemberKind);
        }
        if !committed && source.is_logical_mutation() {
            let projection = source
                .projection_in_custom_scope(&execution)
                .map_err(from_preparation)?;
            let definition =
                projection.data.definitions.get(&relation).ok_or_else(|| {
                    inconsistent(source, "prepared aggregate definition is absent")
                })?;
            if definition.contract.definition != contract.definition {
                return Err(inconsistent(
                    source,
                    "prepared aggregate definition affinity differs",
                ));
            }
            if let Some(changed) = &definition.changed {
                let entries = changed
                    .member_assignments
                    .len()
                    .saturating_add(changed.prior_memberships.len());
                budget
                    .checkpoint((usize::BITS - entries.max(1).leading_zeros()) as u64 * 22 + 1)
                    .map_err(AggregateReadDenial::Budget)?;
                if let Some(&assignment) = changed.member_assignments.get(&member) {
                    if changed
                        .transition
                        .assignments
                        .get(assignment)
                        .is_none_or(|group| !group.members.contains(&member))
                    {
                        return Err(inconsistent(
                            source,
                            "prepared member assignment differs from its transition",
                        ));
                    }
                    budget.check_live().map_err(AggregateReadDenial::Budget)?;
                    return Ok(AggregateRead {
                        backing: ReadBacking::Proposed {
                            projection,
                            relation,
                            assignment,
                        },
                    });
                }
                if changed.prior_memberships.contains_key(&member) {
                    return Err(inconsistent(
                        source,
                        "affected proposed member has no current assignment",
                    ));
                }
            }
            // The complete actual affected-cohort proof excludes this member.
            // Only this proven-unaffected path may read its committed binding.
        }
        let mut allowance = |units| budget.debit_native_rule_work(units);
        let stored = super::stored_read::in_custom_scope(
            &execution,
            state,
            contract,
            member,
            &mut allowance,
        )
        .map_err(|error| {
            source.latch_failure(error.clone());
            from_preparation(error)
        })?;
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        Ok(AggregateRead {
            backing: ReadBacking::Stored(stored),
        })
    }
}

impl AggregateRead {
    pub fn definition(&self) -> AggregateDefinitionIdentity {
        match &self.backing {
            ReadBacking::Proposed {
                projection,
                relation,
                ..
            } => projection.data.definitions[relation].contract.definition,
            ReadBacking::Stored(stored) => stored.contract.definition,
        }
    }
    pub fn identity(&self) -> AggregateIdentity {
        match &self.backing {
            ReadBacking::Proposed {
                projection,
                relation,
                assignment,
            } => {
                projection.data.definitions[relation]
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

    /// Only events incident to this identity in the actual proposed transition.
    /// Committed/before-image reads contribute no new transition events.
    pub fn transition_lineage(&self) -> impl Iterator<Item = AggregateReadLineageEvent<'_>> {
        let events = match &self.backing {
            ReadBacking::Proposed {
                projection,
                relation,
                ..
            } => Some(
                &projection.data.definitions[relation]
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
                relation,
                assignment,
            } => {
                &projection.data.definitions[relation]
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
