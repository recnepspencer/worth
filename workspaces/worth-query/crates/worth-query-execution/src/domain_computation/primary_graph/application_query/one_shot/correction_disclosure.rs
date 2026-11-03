//! Execute a declared correction read without calling an application projector.
use super::super::read_execution::project_non_live_kernel_roots;
use super::*;
use crate::domain_computation::primary_graph::value_decode::ValueDecodeState;
use worth_relational::facade::identity::EntityId;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn execute_correction_disclosure<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        original_subject: EntityId,
    ) -> Result<bool, WorthQueryApplicationOneShotDenial> {
        validate_one_shot_plan(execution, self, &plan)?;
        refresh_governed_authorization(execution, self, &mut plan)
            .map_err(|read| map_authorized_read_denial(read, plan.query.name()))?;
        let (raw, authorization_work, read_proof) =
            execute_authorized_read(execution, self, &plan, |runtime, graph, plan| {
                read_bounded_root_rows(runtime, execution, graph, plan, None)
            })
            .map_err(|read| map_authorized_read_denial(read, plan.query.name()))?;
        let decode = ValueDecodeState::new(self.installed_schema().value_bindings().clone());
        let completed = super::outcome::complete_one_shot_with_projection(
            execution,
            self,
            &decode,
            plan,
            raw,
            authorization_work,
            read_proof,
            |kernel, _, request, query| {
                admit_request(request, query)?;
                Ok(project_non_live_kernel_roots(kernel))
            },
        )?;
        Ok(completed.read.rows().contains(&original_subject))
    }
}
