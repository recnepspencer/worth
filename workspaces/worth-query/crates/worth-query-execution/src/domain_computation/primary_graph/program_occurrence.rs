//! Which rostered program one occurrence of this application runs under.
//!
//! Support is what the host admitted once, at installation: an immutable
//! roster plus the identity of the single activation record that carries owner
//! truth through World. Nothing here decides activation; it resolves a stored
//! rendering back to the rostered program it names, and refuses when no
//! rostered program answers to it.

use std::sync::Arc;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationSchema, WorthQueryProgramAdoptionRequirements,
    WorthQueryProgramAdoptionRequirementsDenial, WorthQueryProgramSupportEntry,
    WorthQueryProgramSupportRoster,
};

mod activation_cell;
mod presented_program;
mod revision_rendering;

pub(in crate::domain_computation::primary_graph) use activation_cell::WorthQueryProgramActivationCell;
pub(in crate::domain_computation::primary_graph) use presented_program::WorthQueryPresentedProgram;
pub(in crate::domain_computation::primary_graph) use revision_rendering::program_revision_rendering;

/// The program support one published application runtime retains.
pub(in crate::domain_computation::primary_graph) struct WorthQueryInstalledProgramSupport<Schema> {
    roster: Arc<WorthQueryProgramSupportRoster<Schema>>,
    renderings: Box<[AspectValue]>,
    activation: WorthQueryProgramActivationCell,
}

impl<Schema> WorthQueryInstalledProgramSupport<Schema> {
    pub(in crate::domain_computation::primary_graph) fn installed(
        roster: Arc<WorthQueryProgramSupportRoster<Schema>>,
        activation: WorthQueryProgramActivationCell,
    ) -> Self {
        let renderings = roster
            .entries()
            .iter()
            .map(|entry| program_revision_rendering(entry.revision()))
            .collect();
        Self {
            roster,
            renderings,
            activation,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn activation(
        &self,
    ) -> &WorthQueryProgramActivationCell {
        &self.activation
    }

    /// Resolves one rostered revision into the program a commit may present.
    ///
    /// A revision this host never admitted resolves to nothing, so no caller
    /// can present meaning the roster does not carry.
    pub(in crate::domain_computation::primary_graph) fn present(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<WorthQueryPresentedProgram<'_>> {
        let position = self
            .roster
            .entries()
            .iter()
            .position(|entry| entry.revision() == revision)?;
        Some(WorthQueryPresentedProgram::rostered(
            self.roster.entries().get(position)?,
            self.renderings.get(position)?,
        ))
    }

    /// The mutation bindings every rostered program between them acts through.
    pub(in crate::domain_computation::primary_graph) fn rostered_mutation_bindings(
        &self,
    ) -> impl Iterator<Item = std::any::TypeId> + '_ {
        self.roster
            .entries()
            .iter()
            .flat_map(|entry| entry.mutation_bindings().iter().copied())
    }

    /// Resolves the rostered program one stored activation rendering names.
    ///
    /// A rendering no rostered program answers to resolves to nothing, which is
    /// how a host refuses work it cannot attribute to admitted meaning.
    pub(in crate::domain_computation::primary_graph) fn rostered_for_rendering(
        &self,
        rendering: &AspectValue,
    ) -> Option<&WorthQueryProgramSupportEntry> {
        self.renderings
            .iter()
            .position(|admitted| admitted == rendering)
            .and_then(|position| self.roster.entries().get(position))
    }

    pub(in crate::domain_computation::primary_graph) fn adoption_requirements(
        &self,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        source: &ApplicationProgramRevision,
        target: &ApplicationProgramRevision,
    ) -> Result<WorthQueryProgramAdoptionRequirements, WorthQueryProgramAdoptionRequirementsDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        self.roster
            .adoption_requirements(installed_schema, source, target)
    }
}
