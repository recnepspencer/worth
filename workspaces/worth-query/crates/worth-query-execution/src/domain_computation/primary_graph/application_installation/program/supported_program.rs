//! Typed access to the rostered programs one published host installed.
//!
//! A host may roster several programs over the same installed schema. Each one
//! is installed in its own right, and what installation yields is recorded
//! here so a caller can name a rostered program by its authoring Rust type
//! without the host keeping a mutable table of who may act.

use std::any::TypeId;
use std::marker::PhantomData;

use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramRevision,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationProgram};

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// What one rostered, installed program authorizes on this host.
///
/// A record is minted only from a [`WorthQueryInstalledApplicationProgram`], so
/// holding one is evidence the program passed rostered installation against
/// this exact installed schema. It grants nothing by itself: activation still
/// decides which rostered program an occurrence is actually running.
pub(in crate::domain_computation::primary_graph) struct WorthQuerySupportedProgramRecord {
    program_type: TypeId,
    revision: ApplicationProgramRevision,
    action_bindings: Box<[TypeId]>,
    output_source_bindings: Box<[TypeId]>,
}

impl WorthQuerySupportedProgramRecord {
    pub(in crate::domain_computation::primary_graph) fn installed<Schema, Program>(
        installed: &WorthQueryInstalledApplicationProgram<Schema, Program>,
        output_source_bindings: &[TypeId],
    ) -> Self
    where
        Schema: ApplicationSchema,
        Program: ApplicationProgramDefinition<Schema> + 'static,
    {
        Self {
            program_type: TypeId::of::<Program>(),
            revision: installed.revision().clone(),
            action_bindings: installed
                .actions()
                .iter()
                .filter_map(|action| action.mutation_binding_type())
                .collect(),
            output_source_bindings: output_source_bindings.into(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn answers_to(&self, program: TypeId) -> bool {
        self.program_type == program
    }

    pub(in crate::domain_computation::primary_graph) const fn revision(
        &self,
    ) -> &ApplicationProgramRevision {
        &self.revision
    }

    pub(in crate::domain_computation::primary_graph) fn declares_action(
        &self,
        binding: TypeId,
    ) -> bool {
        self.action_bindings.contains(&binding)
    }

    pub(in crate::domain_computation::primary_graph) fn requires_output_source(
        &self,
        binding: TypeId,
    ) -> bool {
        self.output_source_bindings.contains(&binding)
    }
}

/// One rostered program, named by its authoring type, as a commit owner.
///
/// The handle borrows the host that installed the program, so it cannot
/// outlive the installation that admitted it and cannot be built for a program
/// this host never rostered. Presenting it still proves nothing about
/// activation: a commit through a handle whose program is not the one this
/// occurrence activated is refused before any effect.
pub struct WorthQuerySupportedProgramHandle<'support, Schema, Program> {
    runtime: &'support WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    record: &'support WorthQuerySupportedProgramRecord,
    marker: PhantomData<fn() -> Program>,
}

impl<'support, Schema, Program> WorthQuerySupportedProgramHandle<'support, Schema, Program> {
    pub(in crate::domain_computation::primary_graph) const fn rostered(
        runtime: &'support WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        record: &'support WorthQuerySupportedProgramRecord,
    ) -> Self {
        Self {
            runtime,
            record,
            marker: PhantomData,
        }
    }

    pub const fn runtime(&self) -> &'support WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime
    }

    pub(in crate::domain_computation::primary_graph) const fn record(
        &self,
    ) -> &'support WorthQuerySupportedProgramRecord {
        self.record
    }

    pub fn contains_action<Binding: 'static>(&self) -> bool {
        self.record.declares_action(TypeId::of::<Binding>())
    }
}
