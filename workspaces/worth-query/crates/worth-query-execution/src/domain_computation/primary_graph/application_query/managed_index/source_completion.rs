//! One ordinary source execution, retained only after its genuine terminal.
use super::source_image::{SourceImage, SourcePlan};
use crate::domain_computation::primary_graph::{
    application_query::{
        authorized_read::{execute_authorized_read, refresh_governed_authorization},
        disclosure::WorthQueryApplicationQueryGovernance,
        one_shot::{
            self,
            outcome::{self, material::MaterialCompletion},
        },
        read_execution::{self, read_plan::ReadPlan, ReadInterruption},
        WorthQueryApplicationOneShotDenial as Denial,
        WorthQueryApplicationOneShotDenialKind as Kind,
    },
    source_facts::SourceFactsAllocation,
    value_decode::ValueDecodeState,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndex;

pub(super) enum SourceDenial {
    Read(Denial),
    Work(super::WorthQueryManagedIndexWorkDenial),
    Source(super::super::WorthQuerySourceExpectationDenial),
}
impl From<worth_relational::facade::transactions::RelationalExecutionDenialCause> for SourceDenial {
    fn from(cause: worth_relational::facade::transactions::RelationalExecutionDenialCause) -> Self {
        Self::Read(one_shot::map_authorized_read_denial(
            super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial::Read(
                read_execution::relational_read_denial(cause, "managed-source"),
            ),
            "managed-source",
        ))
    }
}
impl From<Denial> for SourceDenial {
    fn from(cause: Denial) -> Self {
        Self::Read(cause)
    }
}
impl From<super::super::WorthQuerySourceExpectationDenial> for SourceDenial {
    fn from(cause: super::super::WorthQuerySourceExpectationDenial) -> Self {
        Self::Source(cause)
    }
}
impl From<super::WorthQueryManagedIndexWorkDenial> for SourceDenial {
    fn from(cause: super::WorthQueryManagedIndexWorkDenial) -> Self {
        Self::Work(cause)
    }
}

pub(super) fn complete_source<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    installed: WorthQueryInstalledManagedIndex<S, I>,
    mut plan: SourcePlan<'_, S, I>,
    allocation: Arc<SourceFactsAllocation>,
) -> Result<Arc<SourceImage<S, I>>, SourceDenial> {
    let query = plan.query.name();
    if !installed.is_installed_in(application.installed_schema())
        || !installed.admits_source(plan.query)
        || !matches!(
            plan.governance,
            WorthQueryApplicationQueryGovernance::Public
        )
        || plan.retain_expression_context
    {
        return Err(one_shot::denial(Kind::ForeignPlan, query, query).into());
    }
    one_shot::plan_admission::validate_one_shot_plan(execution, application, &plan)?;
    refresh_governed_authorization(execution, application, &mut plan)
        .map_err(|cause| one_shot::map_authorized_read_denial(cause, query))?;
    let (kernel, authorization_work, read_proof) =
        execute_authorized_read(execution, application, &plan, |runtime, layout, plan| {
            let mut read = ReadPlan::of(plan);
            read.material_source = Some(&allocation);
            read_execution::read_bounded_plan_rows(
                runtime,
                execution,
                layout,
                &read,
                None,
                ReadInterruption::Query(plan.controls.request_scope()),
            )
        })
        .map_err(|cause| one_shot::map_authorized_read_denial(cause, query))?;
    let decode = ValueDecodeState::new(application.installed_schema().value_bindings().clone());
    let storage = MaterialCompletion::new(execution, &allocation);
    let failure = |cause| {
        one_shot::map_authorized_read_denial(
            super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial::Read(
                read_execution::relational_read_denial(cause, query),
            ),
            query,
        )
    };
    let mut tree = None;
    let completed = outcome::complete_one_shot_with_projection(
        execution,
        application,
        &decode,
        plan,
        kernel,
        authorization_work,
        read_proof,
        |kernel, _, _, _| {
            let (projected, retained) = read_execution::project_non_live_kernel_source(
                kernel,
                |count| {
                    storage.work(u64::try_from(count).map_err(|_| failure(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow))?).map_err(failure)
                },
            )?;
            tree = Some(retained);
            Ok(projected)
        },
    )?;
    let parts = completed.into_source_parts(execution)?;
    let tree = tree.expect("successful completion moved the genuine disclosed source once");
    let order = super::source_order::canonical(execution, &tree)?;
    let dependencies = super::source_dependencies::capture(
        execution,
        &parts.sources,
        &parts.result_set,
        &allocation,
    )?;
    storage
        .arc::<crate::domain_computation::provider_session::WorthQueryCompletedSourceRead>()
        .map_err(failure)?;
    let terminal = Arc::new(parts.terminal);
    storage.arc::<SourceImage<S, I>>().map_err(failure)?;
    Ok(Arc::new(SourceImage {
        installed,
        tree,
        order,
        dependencies,
        _sources: parts.sources,
        _result_set: parts.result_set,
        _affinity: parts.affinity,
        terminal,
        _allocation: parts.allocation,
    }))
}
