use crate::equality_type_id::EqualityTypeId as TypeId;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_program::ApplicationProgramIdentity;
use worth_query_installation::facade::WorthQueryProgramSupportEntry;

/// The rostered program a caller presents for one program-gated commit.
///
/// Presenting is not activating. This carries only what the host already
/// admitted about a program - its identity, the stored rendering of its
/// canonical revision, and what it acts through - so that a commit can be
/// compared against the program its occurrence actually activated. It is
/// minted solely by the installed support roster, which is why no caller can
/// present meaning this host never admitted.
pub struct WorthQueryPresentedProgram<'support> {
    entry: &'support WorthQueryProgramSupportEntry,
    rendering: &'support AspectValue,
}

impl<'support> WorthQueryPresentedProgram<'support> {
    pub fn revision(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationProgramRevision {
        self.entry.revision()
    }
    pub const fn rostered(
        entry: &'support WorthQueryProgramSupportEntry,
        rendering: &'support AspectValue,
    ) -> Self {
        Self { entry, rendering }
    }

    pub fn identity(
        &self,
    ) -> &'support ApplicationProgramIdentity {
        self.entry.identity()
    }

    pub const fn rendering(&self) -> &AspectValue {
        self.rendering
    }

    pub fn acts_through_operation(
        &self,
        operation: TypeId,
    ) -> bool {
        self.entry.acts_through_operation(operation.into_external())
    }

    pub fn acts_through_mutation_binding(
        &self,
        binding: TypeId,
    ) -> bool {
        self.entry
            .acts_through_mutation_binding(binding.into_external())
    }
}
