//! One typed refusal for the issued decision index read, owned twice: boxed
//! for a handler exactly as before, retained comparable for a computation.
use super::IndexDecisionStop;
use crate::domain_computation::primary_graph::{
    application_query::{
        managed_index::WorthQueryManagedIndexDenial, WorthQueryApplicationQueryAdmissionDenial,
    },
    HandlerExecutionDenial, WorthQueryInvariantDecisionPlanDenial,
    WorthQueryInvariantProjectionTraversalDenial,
};
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial;
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub enum IndexDecisionFailure {
    ContextUnavailable,
    ForeignProjection,
    Undeclared(WorthQueryInvariantDecisionPlanDenial),
    Installation(WorthQueryApplicationQueryInstallationDenial),
    NativeAccess(WorthQueryApplicationQueryAdmissionDenial),
    ManagedIndex(WorthQueryManagedIndexDenial),
    Native(CompanionPreflightStop),
    /// Reaching a declared peer's shared owner stopped (work, cancellation,
    /// deadline); it is not a foreign scope.
    ScopeTraversal(WorthQueryInvariantProjectionTraversalDenial),
}

impl IndexDecisionFailure {
    pub fn into_handler(self) -> HandlerExecutionDenial {
        match self {
            Self::ContextUnavailable => {
                HandlerExecutionDenial::new(IndexDecisionStop::ContextUnavailable)
            }
            Self::ForeignProjection => {
                HandlerExecutionDenial::new(IndexDecisionStop::ForeignProjection)
            }
            Self::Undeclared(cause) => HandlerExecutionDenial::new(cause),
            Self::Installation(cause) => HandlerExecutionDenial::new(cause),
            Self::NativeAccess(cause) => HandlerExecutionDenial::new(cause),
            Self::ManagedIndex(cause) => HandlerExecutionDenial::new(cause),
            Self::Native(cause) => HandlerExecutionDenial::new(IndexDecisionStop::Native(cause)),
            Self::ScopeTraversal(cause) => HandlerExecutionDenial::new(cause),
        }
    }

    pub fn into_computation(self) -> WorthQueryComputationManagedIndexDenial {
        let cause = match self {
            Self::ContextUnavailable => Cause::ContextUnavailable,
            Self::ForeignProjection => Cause::SourceMismatch,
            Self::Undeclared(cause) => Cause::Undeclared(cause),
            Self::Installation(cause) => Cause::Installation(cause),
            Self::NativeAccess(cause) => Cause::NativeAccess(cause),
            // The owned refusal keeps its own material custody, as the
            // handler's boxed refusal does. It is not retained by the request:
            // charging a stopped request would replace the original cause.
            Self::ManagedIndex(cause) => Cause::ManagedIndex(Arc::new(cause)),
            Self::Native(cause) => Cause::Native(cause),
            Self::ScopeTraversal(cause) => Cause::ScopeTraversal(cause),
        };
        WorthQueryComputationManagedIndexDenial {
            cause: Box::new(cause),
        }
    }
}

/// Why a partitioned computation's managed index read was refused.
///
/// The read is the operation's issued decision read: its Parent must be
/// declared with `operation_managed_index_read` (or, at a peer scope, with
/// `operation_shared_owner_managed_index_read`), its projected fields must be
/// declared reads, and its resolved scope must be the operation's admitted
/// scope or a declared shared-owner peer. It reads the committed admitted
/// basis, or a producer's applied candidate's aligned before basis. Each
/// refusal keeps its original typed cause.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryComputationManagedIndexDenial {
    cause: Box<Cause>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Cause {
    ContextUnavailable,
    SourceMismatch,
    Undeclared(WorthQueryInvariantDecisionPlanDenial),
    Installation(WorthQueryApplicationQueryInstallationDenial),
    NativeAccess(WorthQueryApplicationQueryAdmissionDenial),
    ManagedIndex(Arc<WorthQueryManagedIndexDenial>),
    Native(CompanionPreflightStop),
    ScopeTraversal(WorthQueryInvariantProjectionTraversalDenial),
}

/// The phase that refused a computation's managed index read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryComputationManagedIndexDenialKind {
    /// The reader has no admitted source-query posture: an inspection reader,
    /// or an applied reader without its issuing coupled assembly.
    ContextUnavailable,
    /// The reader's snapshot or selected product is not the borrowed basis.
    SourceMismatch,
    /// The Parent binding or one of its projected fields is not declared.
    UndeclaredDecisionRead,
    /// The Parent binding is not the installed binding.
    Installation,
    /// The original Native transport refused the read.
    NativeAccess,
    /// The managed index read refused; see `managed_index_denial`.
    ManagedIndex,
    /// Retaining the read's Source fact refused; see `native_decision_refusal`.
    NativeDecision,
    /// Reaching a declared peer's shared owner stopped; see
    /// `scope_traversal_denial`.
    ScopeTraversal,
}

impl WorthQueryComputationManagedIndexDenial {
    pub fn kind(&self) -> WorthQueryComputationManagedIndexDenialKind {
        use WorthQueryComputationManagedIndexDenialKind as Kind;
        match self.cause.as_ref() {
            Cause::ContextUnavailable => Kind::ContextUnavailable,
            Cause::SourceMismatch => Kind::SourceMismatch,
            Cause::Undeclared(_) => Kind::UndeclaredDecisionRead,
            Cause::Installation(_) => Kind::Installation,
            Cause::NativeAccess(_) => Kind::NativeAccess,
            Cause::ManagedIndex(_) => Kind::ManagedIndex,
            Cause::Native(_) => Kind::NativeDecision,
            Cause::ScopeTraversal(_) => Kind::ScopeTraversal,
        }
    }
    /// The undeclared Parent or projected-field refusal.
    pub fn decision_plan_denial(&self) -> Option<&WorthQueryInvariantDecisionPlanDenial> {
        match self.cause.as_ref() {
            Cause::Undeclared(cause) => Some(cause),
            _ => None,
        }
    }
    /// The managed index read's own refusal: scope, READ, provider,
    /// cancellation, deadline and work causes.
    pub fn managed_index_denial(&self) -> Option<&WorthQueryManagedIndexDenial> {
        match self.cause.as_ref() {
            Cause::ManagedIndex(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
    /// The stop that interrupted reaching a declared peer's shared owner.
    pub fn scope_traversal_denial(&self) -> Option<&WorthQueryInvariantProjectionTraversalDenial> {
        match self.cause.as_ref() {
            Cause::ScopeTraversal(cause) => Some(cause),
            _ => None,
        }
    }
    /// The exact Native stop retaining the read's Source fact.
    pub fn native_decision_refusal(&self) -> Option<&CompanionPreflightStop> {
        match self.cause.as_ref() {
            Cause::Native(cause) => Some(cause),
            _ => None,
        }
    }
    pub fn installation_denial(&self) -> Option<&WorthQueryApplicationQueryInstallationDenial> {
        match self.cause.as_ref() {
            Cause::Installation(cause) => Some(cause),
            _ => None,
        }
    }
    pub fn native_access_denial(&self) -> Option<&WorthQueryApplicationQueryAdmissionDenial> {
        match self.cause.as_ref() {
            Cause::NativeAccess(cause) => Some(cause),
            _ => None,
        }
    }
}

impl std::fmt::Display for WorthQueryComputationManagedIndexDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "computation managed index read refused: {:?}",
            self.cause
        )
    }
}

impl std::error::Error for WorthQueryComputationManagedIndexDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.cause.as_ref() {
            Cause::Undeclared(cause) => Some(cause),
            Cause::ManagedIndex(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
}

impl worth_execution::ChargedBytes for WorthQueryComputationManagedIndexDenial {
    fn additional_charged_bytes(&self) -> u64 {
        let inline = std::mem::size_of::<Cause>() as u64;
        inline.saturating_add(match self.cause.as_ref() {
            Cause::Undeclared(cause) => cause.additional_charged_bytes(),
            Cause::NativeAccess(cause) => cause.additional_charged_bytes(),
            Cause::Installation(cause) => cause.subject().len() as u64,
            // The refusal's material is held by its own custody.
            Cause::ManagedIndex(_)
            | Cause::ContextUnavailable
            | Cause::SourceMismatch
            | Cause::Native(_) => 0,
            Cause::ScopeTraversal(cause) => cause.relation().len() as u64,
        })
    }
}
