//! Installed workflow vocabularies program adoption compares retained
//! definitions against.
//!
//! Registration happens only where a typed runtime installs a spec for a
//! rostered program, so the registry never holds coverage the host cannot
//! execute. Adoption reads it after program support has retained the target as
//! active, which is what keeps a retiring program's coverage from licensing a
//! carry.

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::WorthQueryWorkflowVocabularyCoverage;

#[derive(Debug, Default)]
pub(in crate::domain_computation::primary_graph) struct WorkflowVocabularyCoverageRegistry {
    coverage: Vec<WorthQueryWorkflowVocabularyCoverage>,
}

/// A second, different vocabulary for one spec at one program revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct ConflictingWorkflowVocabularyCoverage;

/// Holds exclusive registry custody between compatibility admission and installation.
pub(in crate::domain_computation::primary_graph) struct AdmittedWorkflowCoverage<'a> {
    registry: &'a mut WorkflowVocabularyCoverageRegistry,
    addition: Option<WorthQueryWorkflowVocabularyCoverage>,
}

impl AdmittedWorkflowCoverage<'_> {
    pub(in crate::domain_computation::primary_graph) fn install(self) {
        if let Some(coverage) = self.addition {
            self.registry.coverage.push(coverage);
        }
    }
}

impl WorkflowVocabularyCoverageRegistry {
    /// Admits one installed vocabulary. Re-registering the identical
    /// vocabulary is harmless; a different one under the same key is refused
    /// so adoption never compares against an ambiguous vocabulary.
    pub(in crate::domain_computation::primary_graph) fn admit(
        &mut self,
        coverage: WorthQueryWorkflowVocabularyCoverage,
    ) -> Result<AdmittedWorkflowCoverage<'_>, ConflictingWorkflowVocabularyCoverage> {
        let addition = match self.lookup(coverage.spec(), coverage.program_revision()) {
            Some(existing) if existing == &coverage => None,
            Some(_) => return Err(ConflictingWorkflowVocabularyCoverage),
            None => Some(coverage),
        };
        Ok(AdmittedWorkflowCoverage {
            registry: self,
            addition,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn lookup(
        &self,
        spec: &str,
        revision: &ApplicationProgramRevision,
    ) -> Option<&WorthQueryWorkflowVocabularyCoverage> {
        self.coverage
            .iter()
            .find(|coverage| coverage.spec() == spec && coverage.program_revision() == revision)
    }
}
