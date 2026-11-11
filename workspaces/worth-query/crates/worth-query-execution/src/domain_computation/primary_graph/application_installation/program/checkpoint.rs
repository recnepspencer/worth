use worth_query_declaration::facade::{
    application_program::{
        ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
    },
    application_schema::{ApplicationSchemaComposition, ApplicationSchemaDeclaration},
};

use crate::domain_computation::primary_graph::WorthQueryApplicationContributionTuple;

use super::{
    construction::construct_program_with_optional_authorization_time_source,
    WorthQueryApplicationProgramRoots, WorthQueryApplicationProgramRoster,
    WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::application_installation::{
    WorthQueryApplicationLimits, WorthQueryApplicationPreparationDenial,
};

/// The only operational interim checkpoint restore entry.
pub struct WorthQueryInterimCheckpointRestore;

impl WorthQueryInterimCheckpointRestore {
    /// Restores a program runtime from one Query-issued committed-world checkpoint.
    ///
    /// The fresh installation still validates declarations, contributions and the
    /// program. Initial-state authoring is intentionally absent: restored Query
    /// authority is the only model source for this path.
    pub fn program<Schema, Program>(
        program: ValidatedApplicationProgram<Schema, Program>,
        declaration: ApplicationSchemaDeclaration<Schema>,
        configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
        limits: WorthQueryApplicationLimits,
        checkpoint: crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint,
    ) -> Result<
        WorthQueryProgramApplicationRuntime<Schema, Program>,
        WorthQueryApplicationPreparationDenial,
    >
    where
        Schema: ApplicationSchemaComposition,
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs:
            ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
        Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
    {
        WorthQueryInterimCheckpointRestore::rostered_program(
            program,
            WorthQueryApplicationProgramRoster::new(),
            declaration,
            configuration,
            build_identity,
            limits,
            checkpoint,
        )
    }

    /// Restores a program runtime and the programs it supports from one
    /// Query-issued committed-world checkpoint. A host restores the same roster it
    /// installed, so a branch the checkpoint recorded under a supported program
    /// still runs it.
    pub fn rostered_program<Schema, Program>(
        program: ValidatedApplicationProgram<Schema, Program>,
        roster: WorthQueryApplicationProgramRoster<'_, Schema>,
        declaration: ApplicationSchemaDeclaration<Schema>,
        configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
        limits: WorthQueryApplicationLimits,
        checkpoint: crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint,
    ) -> Result<
        WorthQueryProgramApplicationRuntime<Schema, Program>,
        WorthQueryApplicationPreparationDenial,
    >
    where
        Schema: ApplicationSchemaComposition,
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs:
            ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
        Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
    {
        construct_program_with_optional_authorization_time_source(
            program,
            roster,
            declaration,
            configuration,
            build_identity,
            limits,
            |_phase, _graph, _installed| Ok(()),
            None,
            Some(checkpoint),
            None,
        )
    }

    /// Restores the sole checkpoint model source under the caller's original opening request.
    /// Work, memory, cancellation and deadline belong to that phase; the installed
    /// World retains its declared business request policy. No initializer or transition runs.
    pub fn rostered_program_in_advancement<Schema, Program>(
        phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
        program: ValidatedApplicationProgram<Schema, Program>,
        roster: WorthQueryApplicationProgramRoster<'_, Schema>,
        declaration: ApplicationSchemaDeclaration<Schema>,
        configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
        build_identity: worth_foundational::facade::ApplicationBuildIdentity,
        limits: WorthQueryApplicationLimits,
        checkpoint: crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint,
    ) -> Result<
        WorthQueryProgramApplicationRuntime<Schema, Program>,
        WorthQueryApplicationPreparationDenial,
    >
    where
        Schema: ApplicationSchemaComposition,
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs:
            ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
        Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
    {
        phase.execution_request().consult().map_err(|cause| {
        WorthQueryApplicationPreparationDenial::Graph(crate::domain_computation::primary_graph::bootstrap::request_denial(
            crate::domain_computation::primary_graph::application_contribution::advancement_denial(cause),
        ))
    })?;
        super::construction::construct_program_in_advancement(
            phase,
            program,
            roster,
            declaration,
            configuration,
            build_identity,
            limits,
            |_phase, _graph, _installed| Ok(()),
            None,
            Some(checkpoint),
            None,
        )
    }

    /// Restore while retaining privileged, read-only invariant projection authority.
    ///
    /// Only the composition root performing restoration receives this authority.
    /// The checkpoint remains the sole model source: this grants no bootstrap writes.
    /// The returned authority belongs to the restored installation and its current
    /// program activation, exactly like authority retained during initial install.
pub fn program_with_projection<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    limits: WorthQueryApplicationLimits,
    checkpoint: crate::domain_computation::primary_graph::WorthQueryApplicationCheckpoint,
) -> Result<
    (
        WorthQueryProgramApplicationRuntime<Schema, Program>,
        crate::domain_computation::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
            Schema,
        >,
    ),
    WorthQueryApplicationPreparationDenial,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
        let runtime = WorthQueryInterimCheckpointRestore::program(
            program,
            declaration,
            configuration,
            build_identity,
            limits,
            checkpoint,
        )?;
        let projection = runtime
            .runtime()
            .mutation_projection
            .retain_for_installation();
        Ok((runtime, projection))
    }
}
