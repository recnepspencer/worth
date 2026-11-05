//! One ordinary source execution, retained only after its genuine terminal.
use super::source_image::{SourceImage, SourcePlan};
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndex;
use {
    crate::domain_computation::primary_graph::application_query::authorized_read::execute_authorized_read,
    crate::domain_computation::primary_graph::application_query::authorized_read::refresh_governed_authorization,
    crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    crate::domain_computation::primary_graph::application_query::one_shot,
    crate::domain_computation::primary_graph::application_query::one_shot::outcome,
    crate::domain_computation::primary_graph::application_query::one_shot::outcome::material::MaterialCompletion,
    crate::domain_computation::primary_graph::application_query::read_execution,
    crate::domain_computation::primary_graph::application_query::read_execution::read_plan::ReadPlan,
    crate::domain_computation::primary_graph::application_query::read_execution::ReadInterruption,
    crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
    crate::domain_computation::primary_graph::value_decode::ValueDecodeState,
    crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenial as Denial,
    crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind as Kind,
};

pub(super) enum SourceDenial {
    Admission(worth_relational::facade::mvcc::CompanionPreflightStop),
    Native(worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial),
    Read(Denial),
    Work(crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial),
    Source(super::super::WorthQuerySourceExpectationDenial),
}
impl From<worth_relational::facade::transactions::RelationalExecutionDenialCause> for SourceDenial {
    fn from(cause: worth_relational::facade::transactions::RelationalExecutionDenialCause) -> Self {
        Self::Read(crate::domain_computation::primary_graph::vocabulary::application_query::one_shot::map_authorized_read_denial(
            crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial::Read(
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
impl From<crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial> for SourceDenial {
    fn from(cause: crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial) -> Self {
        Self::Work(cause)
    }
}

pub(super) fn complete_source<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>,
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
        return Err(crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::denial(Kind::ForeignPlan, query, query).into());
    }
    one_shot::plan_admission::validate_one_shot_plan(execution, application, &plan)?;
    refresh_governed_authorization(execution, application, &mut plan)
        .map_err(|cause| crate::domain_computation::primary_graph::vocabulary::application_query::one_shot::map_authorized_read_denial(cause, query))?;
    let binding = plan
        .query
        .binding_observation()
        .cloned()
        .ok_or_else(|| crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::denial(Kind::ForeignPlan, query, query))?;
    let source_identity = *plan.query.identity().as_bytes();
    let mut rebuild_loan = None;
    let mut native_denial = None;
    let (kernel, authorization_work, read_proof) =
        execute_authorized_read(execution, application, &plan, |runtime, layout, plan| {
            if let Some(program) = layout.managed_source(I::NAME) {
                rebuild_loan = Some(runtime.read_truth().prepare_managed_index_rebuild(
                    execution, plan.basis.snapshot_handle(), program, plan.scope.entity_id(),
                ).map_err(|cause| {
                    native_denial = Some(cause);
                    read_execution::relational_read_denial(
                        worth_relational::facade::transactions::RelationalExecutionDenialCause::NestedStopped,
                        query,
                    )
                })?);
            }
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
        .map_err(|cause| match native_denial {
            Some(cause) => SourceDenial::Native(cause),
            None => SourceDenial::Read(crate::domain_computation::primary_graph::vocabulary::application_query::one_shot::map_authorized_read_denial(cause, query)),
        })?;
    let decode = ValueDecodeState::new(application.installed_schema().value_bindings().clone());
    let storage = MaterialCompletion::new(execution, &allocation);
    let failure = |cause| {
        crate::domain_computation::primary_graph::vocabulary::application_query::one_shot::map_authorized_read_denial(
            crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial::Read(
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
    let authorization = crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization::complete(
        execution, &parts.terminal, binding, source_identity, parts.authorization, application.authorization(), &allocation,
    ).map_err(SourceDenial::Admission)?.ok_or_else(|| crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::denial(Kind::ForeignPlan, query, query))?;
    let tree = tree.expect("successful completion moved the genuine disclosed source once");
    let order = super::source_order::canonical(execution, &tree)?;
    let dependencies = super::source_dependencies::capture(
        execution,
        &parts.sources,
        &parts.result_set,
        Arc::clone(&authorization),
        &allocation,
    )?;
    storage
        .arc::<crate::domain_computation::provider_session::WorthQueryCompletedSourceRead>()
        .map_err(failure)?;
    let terminal = Arc::new(parts.terminal);
    storage
        .arc::<super::source_image::SourceMaterial>()
        .map_err(failure)?;
    let material = Arc::new(super::source_image::SourceMaterial {
        tree,
        order,
        _allocation: Arc::clone(&parts.allocation),
    });
    storage.arc::<SourceImage<S, I>>().map_err(failure)?;
    Ok(Arc::new(SourceImage {
        rebuild_loan,
        installed,
        material,
        dependencies,
        authorization,
        _sources: parts.sources,
        _result_set: parts.result_set,
        _affinity: parts.affinity,
        terminal,
        _allocation: parts.allocation,
    }))
}
