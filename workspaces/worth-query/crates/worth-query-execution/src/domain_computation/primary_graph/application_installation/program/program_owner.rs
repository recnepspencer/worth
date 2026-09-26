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
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
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

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
{
    /// Resolves the installed commit owner carried by one exact branch.
    ///
    /// ```compile_fail,E0451
    /// use worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwner;
    ///
    /// fn a_revision_cannot_forge_an_owner<Schema>() {
    ///     let _owner = WorthQuerySelectedProgramOwner::<Schema> {
    ///         runtime: todo!(),
    ///         revision: todo!(),
    ///         action_bindings: &[],
    ///         output_source_bindings: &[],
    ///         root_graph_types: &[],
    ///     };
    /// }
    /// ```
    pub fn selected_program_owner(
        &self,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Result<WorthQuerySelectedProgramOwner<'_, Schema>, WorthQuerySelectedProgramOwnerDenial>
    {
        if !std::ptr::eq(selected.application(), &self.runtime) {
            return Err(WorthQuerySelectedProgramOwnerDenial::ProductSelection(
                crate::basis::WorthQueryProductBranchAdmissionDenial::ForeignOwner,
            ));
        }
        let inspection = selected
            .inspect_selected_program()
            .map_err(WorthQuerySelectedProgramOwnerDenial::Inspection)?;
        let revision = inspection.revision();
        if revision == self.program.revision() {
            return Ok(WorthQuerySelectedProgramOwner {
                runtime: &self.runtime,
                revision: self.program.revision(),
                action_bindings: &self.action_bindings,
                output_source_bindings: &self.output_source_bindings,
                root_graph_types: &self.root_graph_types,
            });
        }
        let record = self
            .supported
            .iter()
            .find(|record| record.revision() == revision)
            .ok_or(WorthQuerySelectedProgramOwnerDenial::InstalledOwnerUnavailable)?;
        Ok(WorthQuerySelectedProgramOwner {
            runtime: &self.runtime,
            revision: record.revision(),
            action_bindings: &record.action_bindings,
            output_source_bindings: &record.output_source_bindings,
            root_graph_types: &record.root_graph_types,
        })
    }
}

/// One owner of a rostered program on one published host.
pub trait WorthQueryProgramOwner<Schema>: sealed::WorthQueryProgramOwnership {
    /// The plain application runtime this owner commits through.
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema>;

    /// The canonical revision this owner presents.
    fn owned_revision(&self) -> &ApplicationProgramRevision;

    /// Whether the owned program acts through one mutation binding.
    fn owns_action(&self, binding: TypeId) -> bool;

    /// Whether one mutation binding is an output source rather than an action.
    fn owns_output_source(&self, binding: TypeId) -> bool;

    fn runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.owned_runtime()
    }

    fn contains_action<Binding: 'static>(&self) -> bool {
        self.owns_action(TypeId::of::<Binding>())
    }

    /// Commits one action through the program this owner presents, only when
    /// that program is the one this occurrence activated.
    fn compare_and_commit_program_action<Binding>(
        &self,
        program: WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        commit_program_action::<Schema, Binding, Self>(self, program, idempotency)
    }

    /// Commits one action through the presented program and retains its
    /// client-observed result.
    fn compare_and_commit_program_action_retained<Binding>(
        &self,
        program: WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationRetainedCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        commit_program_action_retained::<Schema, Binding, Self>(self, program, idempotency)
    }
}

impl<Schema> WorthQuerySelectedProgramOwner<'_, Schema> {
    /// Whether the selected program declares one output graph as a root.
    pub(in crate::domain_computation::primary_graph) fn owns_output_root(
        &self,
        root: TypeId,
    ) -> bool {
        self.root_graph_types.contains(&root)
    }
}

impl<Schema> sealed::WorthQueryProgramOwnership for WorthQuerySelectedProgramOwner<'_, Schema> {}

impl<Schema> WorthQueryProgramOwner<Schema> for WorthQuerySelectedProgramOwner<'_, Schema>
where
    Schema: ApplicationSchema,
{
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime
    }

    fn owned_revision(&self) -> &ApplicationProgramRevision {
        self.revision
    }

    fn owns_action(&self, binding: TypeId) -> bool {
        self.action_bindings.contains(&binding)
    }

    fn owns_output_source(&self, binding: TypeId) -> bool {
        self.output_source_bindings.contains(&binding)
    }
}

impl<Schema, Program> sealed::WorthQueryProgramOwnership
    for WorthQueryProgramApplicationRuntime<Schema, Program>
{
}

impl<Schema, Program> WorthQueryProgramOwner<Schema>
    for WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime()
    }

    fn owned_revision(&self) -> &ApplicationProgramRevision {
        self.installed_program().revision()
    }

    fn owns_action(&self, binding: TypeId) -> bool {
        self.declares_action_binding(binding)
    }

    fn owns_output_source(&self, binding: TypeId) -> bool {
        self.requires_output_source(binding)
    }
}

impl<Schema, Spec, Program> sealed::WorthQueryProgramOwnership
    for WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
}

impl<Schema, Spec, Program> WorthQueryProgramOwner<Schema>
    for WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.program_runtime().owned_runtime()
    }

    fn owned_revision(&self) -> &ApplicationProgramRevision {
        self.program_runtime().owned_revision()
    }

    fn owns_action(&self, binding: TypeId) -> bool {
        self.program_runtime().owns_action(binding)
    }

    fn owns_output_source(&self, binding: TypeId) -> bool {
        self.program_runtime().owns_output_source(binding)
    }
}

impl<Schema, Program> sealed::WorthQueryProgramOwnership
    for WorthQuerySupportedProgramHandle<'_, Schema, Program>
{
}

impl<Schema, Program> WorthQueryProgramOwner<Schema>
    for WorthQuerySupportedProgramHandle<'_, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime()
    }

    fn owned_revision(&self) -> &ApplicationProgramRevision {
        self.record().revision()
    }

    fn owns_action(&self, binding: TypeId) -> bool {
        self.record().declares_action(binding)
    }

    fn owns_output_source(&self, binding: TypeId) -> bool {
        self.record().requires_output_source(binding)
    }
}

/// The denial every owner reaches for when it presents meaning this host
/// cannot attribute to an admitted rostered program.
pub(super) fn program_required() -> WorthQueryApplicationCommitDenial {
    WorthQueryApplicationCommitDenial::application_program_required()
}
