//! Why a program migration could not be admitted or sealed; the attempt's migration lane carries it.

/// Why a program migration could not be admitted or sealed. Nothing changed.
#[derive(Debug)]
pub enum WorthQueryProgramMigrationPreparationDenial {
    /// The runtime has no installed program support.
    ProgramSupportUnavailable,
    /// The target program is not on the installed roster.
    TargetProgramUnrostered,
    /// The target program does not act through this mutation binding.
    TargetBindingUnowned,
    /// The candidate belongs to another runtime or schema binding.
    ForeignCandidate,
    /// The runtime could not issue a mutation partition for the migration.
    MutationPartitionUnavailable,
    /// The candidate does something a migration may not do, such as an external
    /// effect, an aftermath, a conditional definition, platform effects,
    /// emitted output, or output production. The text names which.
    UnsupportedPosture(&'static str),
    /// The candidate's effects were refused while building the migration.
    Candidate(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial),
}

impl From<crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>
    for WorthQueryProgramMigrationPreparationDenial
{
    fn from(
        denial: crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    ) -> Self {
        Self::Candidate(denial)
    }
}
