use super::super::{
    WorthQueryApplicationQueryBatchAdmission, WorthQueryApplicationQueryBatchMemory,
    WorthQueryApplicationQueryBatchResourceDenial,
};
use super::*;

mod result;
pub use result::WorthQueryApplicationBatchResult;

/// A batch read preserves the ordinary execution cause and distinguishes an
/// aggregate admission refusal. It carries no partially completed result.
#[derive(Debug)]
pub enum WorthQueryApplicationBatchReadDenial {
    Resource(WorthQueryApplicationQueryBatchResourceDenial),
    Execution(WorthQueryApplicationOneShotDenial),
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Executes an ordinary installed plan under one shared read/custody loan.
    /// Both ordinary index postures and historical authorization remain intact.
    /// The returned claim keeps the admitted result-row envelope charged while
    /// the caller privately stages its result. Preparation and authorization
    /// keep their existing per-item protections, outside this read-work loan.
    /// Each admission consumes one finite slot, including a failed attempt;
    /// retained admission carries its slot so execution does not debit it twice.
    pub fn execute_application_query_one_shot_in_batch<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
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
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> Result<
        WorthQueryApplicationBatchResult<Query, QueryResult>,
        WorthQueryApplicationBatchReadDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        use WorthQueryApplicationBatchReadDenial::{Execution, Resource};
        match plan.planned_batch_item.take() {
            Some(item) if !item.belongs_to(batch) => {
                return Err(Resource(
                    WorthQueryApplicationQueryBatchResourceDenial::ForeignPlan,
                ))
            }
            Some(_) => {}
            None => {
                batch
                    .take_planned_read(
                        plan.runtime_authority.as_u64(),
                        plan.query.binding_identity(),
                        plan.query.identity(),
                        plan.controls.maximum_work().get(),
                        plan.controls.maximum_result_count().get(),
                    )
                    .map_err(Resource)?;
            }
        }
        let rows = batch
            .claim_memory(
                batch.inline_result_bytes(
                    plan.graph_read_plan()
                        .budget_check()
                        .max_inline_result_bytes(),
                ),
            )
            .map_err(Resource)?;
        let reservation = batch
            .reserve_read(plan.controls.maximum_work().get())
            .map_err(Resource)?;
        let maximum = reservation.maximum();
        let spent = OneShotReadWorkObservation::new();
        let result = self.execute_application_query_one_shot_core(
            plan,
            Some(&spent),
            Some(batch),
            Some(maximum),
        );
        let Some(actual) = spent.read() else {
            // Validation, refreshed authorization or buffer admission may stop
            // before the kernel initializes its work observation. Keep that
            // real terminal cause; the reserved loan stays conservatively spent.
            return match result {
                Err(error) => Err(match batch.memory_denial() {
                    Some(denial) => Resource(denial),
                    None => Execution(error),
                }),
                Ok(_) => Err(Resource(
                    WorthQueryApplicationQueryBatchResourceDenial::WorkAccountingMismatch,
                )),
            };
        };
        if let Ok(result) = &result {
            if result.receipt().work().total_work_units() != actual {
                return Err(Resource(
                    WorthQueryApplicationQueryBatchResourceDenial::WorkAccountingMismatch,
                ));
            }
        }
        reservation.settle(actual).map_err(Resource)?;
        match result {
            Ok(result) => Ok(WorthQueryApplicationBatchResult::issued(result, rows)),
            Err(error) => {
                if let Some(denial) = batch.memory_denial() {
                    return Err(Resource(denial));
                }
                Err(Execution(error))
            }
        }
    }
}
