//! A predecessor recipe and author remain concrete request configuration.
use super::super::request::*;
use worth_query_declaration::facade::{application_program::*, application_schema::*};
use worth_query_execution::facade::application_contribution::WorthQueryApplicationContributionTuple;

pub struct NoHomeProgramTransition;
pub struct WorthQueryHomeProgramTransition<'a, Before, Program, Author>
where
    Before: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Before>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Before>,
{
    pub(super) predecessor: WorthQueryApplicationHomeRequest<'a, Before, Program>,
    pub(super) author: Author,
    pub(super) resources: worth_relational::facade::indexes::DerivedIndexMaintenanceBudget,
}
impl<'a, Schema, Program> WorthQueryApplicationHomeRequest<'a, Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    /// Supply the exact predecessor declaration and a pure complete-plan author.
    /// Fresh and already-current Homes never invoke this author or A's seed.
    pub fn with_program_transition<Before, BeforeProgram, Author>(
        self,
        predecessor: WorthQueryApplicationHomeRequest<'a, Before, BeforeProgram>,
        resources: worth_relational::facade::indexes::DerivedIndexMaintenanceBudget,
        author: Author,
    ) -> WorthQueryApplicationHomeRequest<
        'a,
        Schema,
        Program,
        WorthQueryHomeProgramTransition<'a, Before, BeforeProgram, Author>,
    >
    where
        Before: ApplicationSchemaComposition,
        BeforeProgram: ApplicationProgramDefinition<Before>,
        BeforeProgram::Contributions: WorthQueryApplicationContributionTuple<Before>,
        Author: FnOnce(
            &worth_query_execution::facade::application_home_integration::WorthQueryHomeProgramTransitionReadSource<'_, '_>,
            &worth_query_execution::facade::application_home_integration::WorthQueryHomeProgramTransitionTarget<'_>,
        ) -> Result<super::plan::WorthQueryHomeProgramTransitionPlan, worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial>,
{
        WorthQueryApplicationHomeRequest {
            transition: WorthQueryHomeProgramTransition {
                predecessor,
                author,
                resources,
            },
            location: self.location,
            program: self.program,
            roster: self.roster,
            declaration: self.declaration,
            configuration: self.configuration,
            build: self.build,
            limits: self.limits,
            checkpoint_limits: self.checkpoint_limits,
            opening_control: self.opening_control,
            authorization_time_source: self.authorization_time_source,
            initial_state: self.initial_state,
        }
    }
}
