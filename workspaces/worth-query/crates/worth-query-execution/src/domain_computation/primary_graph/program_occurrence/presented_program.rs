use std::any::TypeId;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_program::ApplicationProgramIdentity;
use worth_query_installation::facade::WorthQueryProgramSupportEntry;

/// The rostered program a caller presents for one program-gated commit.
///
/// Presenting is not activating. This carries only what the host already
/// admitted about a program — its identity, the stored rendering of its
/// canonical revision, and what it acts through — so that a commit can be
/// compared against the program its occurrence actually activated. It is
/// minted solely by the installed support roster, which is why no caller can
/// present meaning this host never admitted.
pub(in crate::domain_computation::primary_graph) struct WorthQueryPresentedProgram<'support> {
    entry: &'support WorthQueryProgramSupportEntry,
    rendering: &'support AspectValue,
}

impl<'support> WorthQueryPresentedProgram<'support> {
    pub(super) const fn rostered(
        entry: &'support WorthQueryProgramSupportEntry,
        rendering: &'support AspectValue,
    ) -> Self {
        Self { entry, rendering }
    }

    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> &'support ApplicationProgramIdentity {
        self.entry.identity()
    }

    pub(in crate::domain_computation::primary_graph) const fn rendering(&self) -> &AspectValue {
        self.rendering
    }

    pub(in crate::domain_computation::primary_graph) fn acts_through_operation(
        &self,
        operation: TypeId,
    ) -> bool {
        self.entry.acts_through_operation(operation)
    }
}
