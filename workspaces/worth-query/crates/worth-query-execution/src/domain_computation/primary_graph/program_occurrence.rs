//! Which rostered program one occurrence of this application runs under.
//!
//! Support is what the host admitted once, at installation: an immutable
//! roster plus the identity of the single activation record that carries owner
//! truth through World. Nothing here decides activation; it resolves a stored
//! rendering back to the rostered program it names, and refuses when no
//! rostered program answers to it.

use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::Arc;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationSchema, WorthQueryProgramAdoptionRequirements,
    WorthQueryProgramAdoptionRequirementsDenial, WorthQueryProgramSupportEntry,
    WorthQueryProgramSupportRoster,
};

use super::WorthQueryApplicationCommitDenial;

mod activation_cell;
mod original_meaning;
mod presented_program;
pub(in crate::domain_computation) use original_meaning::OriginalProgramMeaning;
mod revision_rendering;
mod support_lifecycle;

pub(in crate::domain_computation::primary_graph) use activation_cell::WorthQueryProgramActivationCell;
pub(in crate::domain_computation::primary_graph) use presented_program::WorthQueryPresentedProgram;
pub(in crate::domain_computation::primary_graph) use revision_rendering::program_revision_rendering;
pub use support_lifecycle::WorthQueryProgramSupportRetirementReceipt;
pub(in crate::domain_computation::primary_graph) use support_lifecycle::{
    WorthQueryProgramSupportCustody, WorthQueryProgramSupportInterpretation,
};

/// The program support one published application runtime retains.
pub(in crate::domain_computation::primary_graph) struct WorthQueryInstalledProgramSupport<Schema> {
    native_programs: Box<[Arc<super::application_installation::WorthQuerySupportedProgramRecord>]>,
    roster: Arc<WorthQueryProgramSupportRoster<Schema>>,
    renderings: Box<[AspectValue]>,
    activation: WorthQueryProgramActivationCell,
    lifecycle: support_lifecycle::WorthQueryProgramSupportLifecycle,
    pub(in crate::domain_computation::primary_graph) adoption_admissions:
        std::collections::BTreeMap<
            ApplicationProgramRevision,
            super::product_operation::program_adoption::authorization::ProgramAdoptionAdmission<
                Schema,
            >,
        >,
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
            native_programs: Box::new([]),
            lifecycle: support_lifecycle::WorthQueryProgramSupportLifecycle::installed(&roster),
            adoption_admissions: std::collections::BTreeMap::new(),
            roster,
            renderings,
            activation,
        }
    }

    /// Completes the existing support owner before the assembled runtime is published.
    pub(in crate::domain_computation::primary_graph) fn complete_native_installation(
        &mut self,
        programs: Box<[Arc<super::application_installation::WorthQuerySupportedProgramRecord>]>,
    ) {
        assert!(self.native_programs.is_empty());
        assert_eq!(programs.len(), self.roster.entries().len());
        assert!(self.roster.entries().iter().all(|entry| programs
            .iter()
            .any(|program| entry.revision() == program.revision())));
        self.native_programs = programs;
    }

    pub(in crate::domain_computation::primary_graph) fn native_program(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<&super::application_installation::WorthQuerySupportedProgramRecord> {
        self.present(revision)?;
        self.native_programs
            .iter()
            .find(|program| program.revision() == revision)
            .map(AsRef::as_ref)
    }

    pub(in crate::domain_computation::primary_graph) fn adoption_authorization_matches(
        &self,
        revision: &ApplicationProgramRevision,
        meaning: &str,
    ) -> bool {
        self.roster
            .entries()
            .iter()
            .find(|entry| entry.revision() == revision)
            .is_some_and(|entry| {
                entry.semantic_description().facts().iter().any(|fact| {
                    fact.subject() == "program-adoption-authorization"
                        && fact.canonical_meaning() == meaning
                })
            })
    }

    pub(in crate::domain_computation::primary_graph) fn adoption_admission(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<
        super::product_operation::program_adoption::authorization::ProgramAdoptionAdmission<Schema>,
    > {
        self.present(revision)?;
        self.adoption_admissions.get(revision).copied()
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
        if !self.lifecycle.is_active(revision) {
            return None;
        }
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

    /// Resolves one revision into the program a commit presents, or the typed
    /// denial that refuses it before any effect.
    ///
    /// A revision this host never admitted is refused as program required. A
    /// rostered revision whose support is retiring or retired is refused as
    /// retired support, so a caller holding a stale owner learns why it lost
    /// standing.
    pub(in crate::domain_computation::primary_graph) fn present_for_commit(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Result<WorthQueryPresentedProgram<'_>, WorthQueryApplicationCommitDenial> {
        self.present(revision).ok_or_else(|| {
            if self
                .roster
                .entries()
                .iter()
                .any(|entry| entry.revision() == revision)
            {
                WorthQueryApplicationCommitDenial::program_support_retired(revision)
            } else {
                WorthQueryApplicationCommitDenial::application_program_required()
            }
        })
    }

    /// Installation completeness includes owner-only bindings, unlike call eligibility.
    pub(in crate::domain_computation::primary_graph) fn declares_mutation_binding(
        &self,
        binding: TypeId,
    ) -> bool {
        self.roster
            .entries()
            .iter()
            .any(|entry| entry.declares_mutation_binding(binding.into_external()))
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

    /// Resolves immutable native provenance to an already admitted revision.
    pub(in crate::domain_computation::primary_graph) fn rostered_for_revision_text(
        &self,
        text: &str,
    ) -> Option<&WorthQueryProgramSupportEntry> {
        self.renderings.iter().position(|value| matches!(value,
            AspectValue::String(worth_foundational::facade::InternedString::Raw(value)) if value == text))
            .and_then(|position| self.roster.entries().get(position))
    }

    pub(in crate::domain_computation::primary_graph) fn rostered_for_recovery(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<&WorthQueryProgramSupportEntry> {
        self.roster.entry(revision)
    }

    pub(in crate::domain_computation::primary_graph) fn retain_interpretation(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<WorthQueryProgramSupportInterpretation> {
        self.lifecycle.retain_interpretation(revision)
    }

    pub(in crate::domain_computation::primary_graph) fn retain_custody(
        &self,
        source: &ApplicationProgramRevision,
        target: &ApplicationProgramRevision,
    ) -> Option<WorthQueryProgramSupportCustody> {
        self.lifecycle.retain_custody(source, target)
    }

    pub(in crate::domain_computation::primary_graph) const fn lifecycle(
        &self,
    ) -> &support_lifecycle::WorthQueryProgramSupportLifecycle {
        &self.lifecycle
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
        if !self.lifecycle.is_active(target) {
            return Err(
                WorthQueryProgramAdoptionRequirementsDenial::UnrosteredTarget { revision: *target },
            );
        }
        self.roster
            .adoption_requirements(installed_schema, source, target)
    }
}
