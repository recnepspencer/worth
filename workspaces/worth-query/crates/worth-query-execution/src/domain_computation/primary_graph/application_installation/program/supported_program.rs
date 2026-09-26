//! Typed access to the rostered programs one published host installed.
//!
//! A host may roster several programs over the same installed schema. Each one
//! is installed in its own right, and what installation yields is recorded
//! here so a caller can name a rostered program by its authoring Rust type
//! without the host keeping a mutable table of who may act.

use std::any::{Any, TypeId};

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
    /// The typed installed program, so owners that install against program
    /// vocabulary (such as a workflow spec) can name it by its authoring type.
    installed: Box<dyn Any + Send + Sync>,
    pub(in crate::domain_computation::primary_graph) action_bindings: Box<[TypeId]>,
    pub(in crate::domain_computation::primary_graph) output_source_bindings: Box<[TypeId]>,
}

impl WorthQuerySupportedProgramRecord {
    pub(in crate::domain_computation::primary_graph) fn installed<Schema, Program>(
        installed: WorthQueryInstalledApplicationProgram<Schema, Program>,
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
            installed: Box::new(installed),
        }
    }

    /// The typed installed program, when this record was minted for `Program`.
    fn installed_program<Schema, Program>(
        &self,
    ) -> Option<&WorthQueryInstalledApplicationProgram<Schema, Program>>
    where
        Schema: 'static,
        Program: 'static,
    {
        self.installed.downcast_ref()
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
    installed: &'support WorthQueryInstalledApplicationProgram<Schema, Program>,
}

impl<'support, Schema, Program> WorthQuerySupportedProgramHandle<'support, Schema, Program> {
    /// Names `record` as `Program`; a record minted for another program
    /// yields no handle.
    pub(in crate::domain_computation::primary_graph) fn rostered(
        runtime: &'support WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        record: &'support WorthQuerySupportedProgramRecord,
    ) -> Option<Self>
    where
        Schema: 'static,
        Program: 'static,
    {
        Some(Self {
            runtime,
            record,
            installed: record.installed_program()?,
        })
    }

    /// The rostered program's installed meaning, for owners that install
    /// vocabulary against it.
    pub const fn installed_program(
        &self,
    ) -> &'support WorthQueryInstalledApplicationProgram<Schema, Program> {
        self.installed
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
