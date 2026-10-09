//! Who may present a program for one program-gated commit.
//!
//! Exactly two owner categories exist: the runtime published with a host's
//! initial program, and handles onto programs that host rostered. A selected
//! owner is a branch-resolved form of either category. The trait is sealed so no
//! outside owner can appear, and every program-gated
//! commit entry point accepts owners only through it. Owning a program is not
//! activating it: the shared implementations resolve the occurrence's active
//! program before any effect, and refuse a presented program that is not it.

use std::any::TypeId;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramRevision, ApplicationWorkflowSpec,
};
use worth_query_installation::facade::ApplicationSchema;

use super::supported_program::WorthQuerySupportedProgramHandle;
use super::{WorthQueryProgramApplicationRuntime, WorthQueryWorkflowApplicationRuntime};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationRetainedCommitOutcome, WorthQueryPrimaryGraphApplicationRuntime,
};

mod occurrence_commit;
mod ownership;
mod selected_owner;
use crate::domain_computation::application_aftermath::ApplicationCommitCausality;

use occurrence_commit::{commit_program_action, commit_program_action_retained};

mod sealed {
    /// Closes program ownership to the two categories the platform publishes.
    pub trait WorthQueryProgramOwnership {}
}

/// A commit owner resolved from the program carried by one live, typed branch
/// selection. Its fields are private so a revision or lookalike activation
/// cannot manufacture commit authority.
pub struct WorthQuerySelectedProgramOwner<'runtime, Schema> {
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    revision: &'runtime ApplicationProgramRevision,
    action_bindings: &'runtime [TypeId],
    output_source_bindings: &'runtime [TypeId],
    root_graph_types: &'runtime [TypeId],
}

#[derive(Debug)]
pub enum WorthQuerySelectedProgramOwnerDenial {
    ProductSelection(
        crate::domain_computation::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    Inspection(crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial),
    InstalledOwnerUnavailable,
}

mod contract;
pub use contract::WorthQueryProgramOwner;

/// The denial every owner reaches for when it presents meaning this host
/// cannot attribute to an admitted rostered program.
pub(super) fn program_required() -> WorthQueryApplicationCommitDenial {
    WorthQueryApplicationCommitDenial::application_program_required()
}
