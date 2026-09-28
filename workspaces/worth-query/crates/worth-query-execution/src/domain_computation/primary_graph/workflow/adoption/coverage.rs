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

impl WorkflowVocabularyCoverageRegistry {
    /// Records one installed vocabulary. Re-registering the identical
    /// vocabulary is harmless; a different one under the same key is refused
    /// so adoption never compares against an ambiguous vocabulary.
    pub(in crate::domain_computation::primary_graph) fn register(
        &mut self,
        coverage: WorthQueryWorkflowVocabularyCoverage,
    ) -> Result<(), ConflictingWorkflowVocabularyCoverage> {
        match self.lookup(coverage.spec(), coverage.program_revision()) {
            Some(existing) if existing == &coverage => Ok(()),
            Some(_) => Err(ConflictingWorkflowVocabularyCoverage),
            None => {
                self.coverage.push(coverage);
                Ok(())
            }
        }
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
