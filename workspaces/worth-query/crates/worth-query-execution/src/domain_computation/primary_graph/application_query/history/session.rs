//! A page's real graph-session proof and both decisions close before aggregate handoff.
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_query::ApplicationHistoryQueryBinding, application_schema::ApplicationSchema,
};

use super::{
    cursor::HistoryPosition, storage, transition, HistoryMetadataAdmission, SubjectPlan,
    WorthQueryApplicationHistoryCursor as Cursor, WorthQueryApplicationHistoryDenial as Denial,
    WorthQueryApplicationHistoryPage as Page, WorthQueryApplicationHistoryPageLimits as Limits,
    WorthQueryCapturedApplicationHistorySubjects as Subjects,
};
use crate::domain_computation::{
    primary_graph::{
        application_query::{
            authorized_read::{
                execute_authorized_read_with_access, refresh_governed_authorization,
                WorthQueryAuthorizedApplicationReadDenial,
            },
            one_shot::plan_admission::{
                admit_request, validate_authentication_lifetime, validate_basis_lifetime,
                validate_one_shot_plan,
            },
        },
        WorthQueryPrimaryGraphApplicationRuntime,
    },
    provider_session::{WorthQueryObservedGraphReadWork, WorthQuerySessionGraphReadProof},
};

pub(super) fn open<Schema: ApplicationSchema, History: ApplicationHistoryQueryBinding<Schema>>(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    mut plan: SubjectPlan<'_, Schema, History>,
    metadata: HistoryMetadataAdmission<History>,
    subjects: Subjects<Schema, History>,
    maximum_owned_bytes: usize,
) -> Result<Cursor<Schema, History>, Denial> {
    validate_one_shot_plan(&execution, application, &plan)?;
    refresh_governed_authorization(&execution, application, &mut plan).map_err(Denial::from)?;
    execute_authorized_read_with_access(
        &execution,
        application,
        &plan,
        |runtime, _, security, plan| {
            metadata.validate(&execution, application, runtime, security, plan)
        },
    )?;
    let opened = Cursor::open(
        execution,
        application,
        subjects,
        plan.basis.retained_product(),
        maximum_owned_bytes,
    )?;
    // World opening never holds the Native owner. Revalidate on the genuine current
    // security root after that handoff before publishing even an empty cursor.
    let (_, _, proof) = execute_authorized_read_with_access(
        &execution,
        application,
        &plan,
        |runtime, _, security, plan| {
            metadata.validate(&execution, application, runtime, security, plan)
        },
    )?;
    complete::<Schema, History>(execution, application, plan, proof)?;
    execution
        .run_native(|_| opened)
        .map(|(opened, _)| opened)
        .map_err(Denial::Request)
}

pub(super) fn page<Schema: ApplicationSchema, History: ApplicationHistoryQueryBinding<Schema>>(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    mut plan: SubjectPlan<'_, Schema, History>,
    metadata: HistoryMetadataAdmission<History>,
    subjects: &Subjects<Schema, History>,
    original: &Cursor<Schema, History>,
    limits: Limits,
) -> Result<(Page, Cursor<Schema, History>), Denial> {
    validate_one_shot_plan(&execution, application, &plan)?;
    refresh_governed_authorization(&execution, application, &mut plan).map_err(Denial::from)?;
    let (staged, authorization_work, _) = execute_authorized_read_with_access(
        &execution,
        application,
        &plan,
        |runtime, _, security, plan| {
            metadata.validate(&execution, application, runtime, security, plan)?;
            if !Arc::ptr_eq(&subjects.owner, &original.source.subjects.owner)
                || plan.basis.selected_product().observation()
                    != original.source.target.observation()
            {
                return Err(Denial::InvalidContinuation);
            }
            storage::require(size_of::<Page>(), limits.maximum_owned_bytes.get())?;
            match &original.position {
                HistoryPosition::Complete => Ok(Some(transition::complete())),
                HistoryPosition::World(_) => Ok(None),
                HistoryPosition::Native {
                    world,
                    after_world,
                    cursor,
                } => transition::prepare_native(
                    execution,
                    runtime,
                    world,
                    after_world,
                    cursor,
                    limits,
                )
                .map(Some),
            }
        },
    )?;
    // Exactly one World parent is prepared outside the Native borrow. It never
    // also scans a canonical record, even when the visit limit is one.
    let world = match &original.position {
        HistoryPosition::World(world) => Some(transition::prepare_world::<Schema, History>(
            execution,
            application,
            world,
            limits,
        )?),
        _ => None,
    };
    let (mut accepted, _, proof) = execute_authorized_read_with_access(
        &execution,
        application,
        &plan,
        |runtime, _, security, plan| {
            metadata.validate(&execution, application, runtime, security, plan)?;
            match world {
                Some(world) => world.finish(runtime, execution, original),
                None => staged.ok_or(Denial::InvalidContinuation),
            }
        },
    )?;
    // Success performed both real security resolutions, without repeating the
    // admission facts/canonical work of either separate governance decision.
    accepted.page.authorization_work =
        authorization_work.with_execution_security_product_resolution();
    complete::<Schema, History>(execution, application, plan, proof)?;
    execution
        .run_native(|_| (accepted.page, original.successor(accepted.position)))
        .map(|(accepted, _)| accepted)
        .map_err(Denial::Request)
}

fn complete<Schema: ApplicationSchema, History: ApplicationHistoryQueryBinding<Schema>>(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    plan: SubjectPlan<'_, Schema, History>,
    proof: WorthQuerySessionGraphReadProof,
) -> Result<(), Denial> {
    let release = plan.basis.release();
    if !release.custody_released() {
        return Err(Denial::BasisReleaseFailed);
    }
    validate_basis_lifetime(&plan.controls, plan.query.name())?;
    validate_authentication_lifetime(application, plan.principal, plan.query.name())?;
    admit_request(plan.controls.request_scope(), plan.query.name())?;
    // This session admitted current permissions and canonical history, and did
    // not project ordinary graph rows. History reports its actual visits itself.
    plan.graph_work
        .complete_query_read(
            proof,
            WorthQueryObservedGraphReadWork {
                examined_candidates: 0,
                projected_records: 0,
                projected_fields: 0,
                relation_records_examined: 0,
                ordering_comparisons: 0,
            },
            release,
        )
        .map_err(|_| Denial::from(WorthQueryAuthorizedApplicationReadDenial::Session))?;
    execution.run_native(|_| ()).map_err(Denial::Request)?;
    Ok(())
}
