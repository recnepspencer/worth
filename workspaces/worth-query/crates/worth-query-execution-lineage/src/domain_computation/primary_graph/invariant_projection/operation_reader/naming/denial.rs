//! Why a decision reader refused a persistent-name read.

use std::fmt;

use worth_query_execution_naming::facade::ExecutionNameReadDenial;
use worth_relational::facade::lineage::LineageRecordTarget;

use super::super::current_output::WorthQueryCurrentOutputDenial;
use super::super::decision_plan::WorthQueryInvariantDecisionPlanDenial;
use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionTraversalDenial;

/// Why a decision reader refused a persistent-name read before reading.
///
/// On an applied candidate (a coupled producer or a later edit-envelope
/// stage) names are read from the committed name index at the commit basis
/// the candidate was proposed on. Such an answer holds for the candidate only
/// while every record it rests on is live at the same identity at both; the
/// `*Candidate*` variants refuse the reads where the unpublished candidate
/// changed what the answer rests on.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryDecisionNameDenial {
    /// The operation did not declare a decision read of the entity or
    /// relation kind.
    UndeclaredDecisionTarget(WorthQueryInvariantDecisionPlanDenial),
    /// The identity was issued by another reader, phase or kind, or a
    /// relation handle does not designate the live relation it claims.
    ForeignIdentity,
    /// The applied candidate has no aligned committed name index: the reader
    /// holds no committed-runtime access, or the candidate was not proposed
    /// on this reader's exact basis and runtime.
    CandidateBasis,
    /// The record was not live at the commit basis: the unpublished
    /// candidate created it, so it carries no published name yet.
    UnpublishedCandidateRecord(LineageRecordTarget),
    /// The record was live at the commit basis, but the unpublished
    /// candidate ended it (deleted, retired, split or merged it away), so its
    /// basis name no longer designates a live record.
    EndedInCandidate(LineageRecordTarget),
    /// The resolution rested on an ended name or a held name not yet born,
    /// which the unpublished candidate may restore, regenerate or bear.
    CandidateNameHistory,
    /// A resolved live record could not be admitted as a local identity.
    Identity(WorthQueryCurrentOutputDenial),
    /// A live relation's endpoint could not be admitted as a local identity.
    Endpoint(WorthQueryInvariantProjectionTraversalDenial),
    /// The kind declares no owner scope, so nothing can own its records.
    OwnerScopeUndeclared,
    /// A hop of the kind's declared owner chain could not be read.
    OwnerRead(WorthQueryInvariantProjectionTraversalDenial),
    /// The name read itself refused.
    Read(ExecutionNameReadDenial),
    /// A group read on an applied candidate: the candidate's own group
    /// partitions are not published until its envelope commits, so no basis
    /// answer is the candidate's answer. Read groups on a committed basis.
    CandidateGroupState,
}

impl fmt::Display for WorthQueryDecisionNameDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndeclaredDecisionTarget(denial) => write!(formatter, "{denial:?}"),
            Self::ForeignIdentity => formatter.write_str("identity is foreign to this reader"),
            Self::CandidateBasis => formatter
                .write_str("the applied candidate has no aligned committed name index"),
            Self::UnpublishedCandidateRecord(record) => write!(
                formatter,
                "{record:?} was created by the unpublished candidate and has no published name"
            ),
            Self::EndedInCandidate(record) => write!(
                formatter,
                "{record:?} was ended by the unpublished candidate"
            ),
            Self::CandidateNameHistory => formatter.write_str(
                "the resolution rests on an ended or unborn name the unpublished candidate may change",
            ),
            Self::Identity(denial) => write!(formatter, "{denial:?}"),
            Self::Endpoint(denial) | Self::OwnerRead(denial) => write!(formatter, "{denial:?}"),
            Self::OwnerScopeUndeclared => {
                formatter.write_str("the name kind declares no owner scope")
            }
            Self::Read(denial) => fmt::Display::fmt(denial, formatter),
            Self::CandidateGroupState => formatter
                .write_str("persistent groups are read on a committed basis, not a candidate"),
        }
    }
}

impl std::error::Error for WorthQueryDecisionNameDenial {}
