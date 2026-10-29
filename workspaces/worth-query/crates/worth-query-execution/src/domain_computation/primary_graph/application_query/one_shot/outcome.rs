use super::{
    admit_request, denial, validate_authentication_lifetime, validate_basis_lifetime,
    WorthQueryApplicationOneShotDenial, WorthQueryApplicationOneShotDenialKind,
    WorthQueryApplicationOneShotResult,
};
use crate::domain_computation::primary_graph::application_query::{
    access_receipt::{
        WorthQueryApplicationQueryReceiptBasis, WorthQueryApplicationQueryReceiptIdentity,
    },
    read_execution::{project_non_live_kernel_with_decode, RawNonLiveKernelOutcome},
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationAuthorizationWorkEvidence,
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessReceipt,
};
use crate::domain_computation::primary_graph::value_decode::ValueDecodeState;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use std::marker::PhantomData;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
mod ordinary;
pub(in crate::domain_computation::primary_graph::application_query) use ordinary::{
    complete_one_shot, complete_one_shot_with_projection, finalize_one_shot,
    finalize_one_shot_with_decode,
};
mod completed;
pub(in crate::domain_computation::primary_graph::application_query) mod material;
pub(in crate::domain_computation::primary_graph::application_query) use completed::CompletedOneShotRead;
mod decode_denial;
mod result_set_footprint;
pub(in crate::domain_computation::primary_graph::application_query) mod source_storage;
pub(in crate::domain_computation::primary_graph::application_query) use decode_denial::value_decode_denial;
use result_set_footprint::ResultSetFootprint;
pub(in crate::domain_computation::primary_graph::application_query) fn complete_one_shot_with_capture<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
    Projected,
    Capture,
    E,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    decode: &ValueDecodeState,
    plan: WorthQueryAdmittedApplicationQueryPlan<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    mut kernel: RawNonLiveKernelOutcome,
    authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
    read_proof: crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    project: impl FnOnce(
        RawNonLiveKernelOutcome,
        &super::super::disclosure::WorthQueryApplicationQueryGovernance,
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        &str,
    ) -> Result<
        super::super::read_execution::ProjectedNonLiveKernelOutcome<Projected>,
        WorthQueryApplicationOneShotDenial,
    >,
    capture: impl FnOnce(
        &super::super::projection::WorthQueryApplicationDisclosedProjectionTree,
        &WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<Capture, E>,
) -> Result<CompletedOneShotRead<Query, Projected, Capture>, E>
where
    Schema: ApplicationSchema,
    E: From<WorthQueryApplicationOneShotDenial>,
{
    if let Some(cause) = decode.denial() {
        return Err(value_decode_denial(cause, plan.query.name()).into());
    }
    // Only the private managed-source kernel supplies this original grant.
    let allocation = kernel
        .raw
        .material_source
        .as_ref()
        .map(std::sync::Arc::clone);
    let material = allocation
        .as_ref()
        .map(|allocation| material::MaterialCompletion::new(execution, allocation));
    let material_failure = |cause| {
        super::map_authorized_read_denial(
            super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial::Read(
                super::super::read_execution::relational_read_denial(cause, plan.query.name()),
            ),
            plan.query.name(),
        )
    };
    if let Some(material) = &material {
        if plan.retain_expression_context {
            return Err(denial(
                WorthQueryApplicationOneShotDenialKind::ExpressionContextUnavailable,
                plan.query.name(),
                plan.query.name(),
            )
            .into());
        }
        material
            .affinity(plan.principal)
            .map_err(&material_failure)?;
    }
    let request_affinity =
        super::super::admitted_result::WorthQueryApplicationQueryRequestAffinity::new(
            plan.principal,
            plan.controls.request_scope(),
        );
    let expression_context = if plan.retain_expression_context {
        Some(std::sync::Arc::new(
            super::super::observed_source::RetainedExpressionContext::retain(
                plan.query.read_graph(),
                plan.query.expression_context_schema(),
                &kernel.raw,
                plan.controls.request_scope(),
            )
            .map_err(|kind| denial(kind, plan.query.name(), plan.query.name()))?,
        ))
    } else {
        None
    };
    let (source_footprints, result_set_selection) = ResultSetFootprint::take_rows(&mut kernel)
        .map_err(|kind| denial(kind, plan.query.name(), plan.query.name()))?;
    let set_footprint = ResultSetFootprint::prepare(
        plan.scope.entity_id(),
        result_set_selection,
        expression_context.is_some(),
        &source_footprints,
    )
    .map_err(|kind| denial(kind, plan.query.name(), plan.query.name()))?;
    let request = plan.controls.request_scope();
    if let Some(material) = &material {
        material
            .basis(plan.basis.identity())
            .map_err(&material_failure)?;
    }
    let basis_identity = plan.basis.identity().clone();
    let basis_version = plan.basis.version_id();
    let parameter_binding_identity = *plan.parameters.identity();
    admit_request(request, plan.query.name())?;
    let capture = capture(&kernel.raw.rows, &plan)?;
    let retained_projection = if plan.retain_projection_for_inspection {
        Some(super::super::projection::retained_projection::retain(
            execution,
            &mut kernel.raw,
            plan.query.read_graph(),
            &plan.governance,
            plan.query.binding_observation().is_some(),
            plan.query.name(),
        )?)
    } else {
        None
    };
    let scope_selector = if material.is_some() {
        material::retain_scope(execution, plan.scope, plan.query.name())?
    } else {
        plan.scope.retain_scope_selector()
    };
    if let Some(material) = &material {
        material
            .original_metadata(plan.query.name(), basis_identity.branch_id())
            .map_err(&material_failure)?;
    }
    let query_identifier = std::sync::Arc::new(plan.query.name().to_owned());
    let result_set_footprint = set_footprint.materialize();
    let branch = std::sync::Arc::new(basis_identity.branch_id().clone());
    let mut source_comparisons = crate::domain_computation::primary_graph::source_read_equivalence::SourceReadEquivalence::default();
    let source_control = || {
        match request.interruption() {
        None => Ok(()),
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled) => Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled),
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded) => Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::DeadlineElapsed),
    }
    };
    let mut compare_source =
        |left: &super::super::observed_source::WorthQueryObservedSourceFootprint,
         right: &super::super::observed_source::WorthQueryObservedSourceFootprint| {
            super::super::observed_source::comparison::equal(
                left,
                right,
                &mut source_comparisons,
                execution,
                &source_control,
            )
            .map_err(&material_failure)
        };
    let mut row_meanings = worth_execution::ExecutionRequestArrayBuilder::allocate(
        *execution,
        source_footprints.len(),
    )
    .map_err(|cause| source_storage::denied(cause, plan.query.name()))?;
    for footprint in source_footprints {
        admit_request(request, plan.query.name())?;
        let selection = basis_identity.selection().clone();
        let source_meaning = if let Some(allocation) = &allocation {
            application.source_meanings.intern_material_compared(
                plan.query.identity().as_bytes(),
                parameter_binding_identity.bytes(),
                footprint,
                &selection,
                &mut compare_source,
                execution,
                allocation,
                &mut |cause| material_failure(cause),
            )
        } else {
            application.source_meanings.intern_compared(
                plan.query.identity().as_bytes(),
                parameter_binding_identity.bytes(),
                footprint,
                &selection,
                &mut compare_source,
            )
        }?
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationOneShotDenialKind::SourceIdentityExhausted,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
        row_meanings
            .push(source_meaning)
            .map_err(|cause| source_storage::denied(cause, plan.query.name()))?;
    }
    let row_meanings = row_meanings
        .seal()
        .map_err(|cause| source_storage::denied(cause, plan.query.name()))?;
    let source_parameters = std::sync::Arc::new(
        super::super::observed_source::WorthQueryRetainedObservedParameters::new(
            plan.parameters,
            plan.query.binding_observation().cloned(),
            row_meanings,
            plan.controls
                .original_settled_producer_for(plan.basis.selected_product().observation()),
            retained_projection,
        ),
    );
    let mut observed_sources = if let Some(material) = &material {
        material
            .source_headers(source_parameters.row_meanings().len())
            .map_err(&material_failure)?
    } else {
        Vec::with_capacity(source_parameters.row_meanings().len())
    };
    for source_meaning in source_parameters.row_meanings() {
        admit_request(request, plan.query.name())?;
        observed_sources.push(super::super::WorthQueryObservedSource {
            expression_context: expression_context.clone(),
            runtime_authority: plan.runtime_authority.as_u64(),
            schema_binding: plan.query.binding_identity().clone(),
            query_identity: plan.query.identity().clone(),
            parameter_binding_identity,
            parameters: std::sync::Arc::clone(&source_parameters),
            query_identifier: std::sync::Arc::clone(&query_identifier),
            branch: std::sync::Arc::clone(&branch),
            selection: basis_identity.selection().clone(),
            model_root: plan.scope.entity_id(),
            source_meaning: std::sync::Arc::clone(source_meaning),
            scope_selector: std::sync::Arc::clone(&scope_selector),

            _marker: PhantomData,
        });
    }
    let result_set_meaning = if let Some(allocation) = &allocation {
        application
            .source_meanings
            .intern_material_result_set_compared(
                plan.query.identity().as_bytes(),
                parameter_binding_identity.bytes(),
                result_set_footprint,
                basis_identity.selection(),
                &mut compare_source,
                execution,
                allocation,
                &mut |cause| material_failure(cause),
            )
    } else {
        application.source_meanings.intern_result_set_compared(
            plan.query.identity().as_bytes(),
            parameter_binding_identity.bytes(),
            result_set_footprint,
            basis_identity.selection(),
            &mut compare_source,
        )
    }?
    .ok_or_else(|| {
        denial(
            WorthQueryApplicationOneShotDenialKind::SourceIdentityExhausted,
            plan.query.name(),
            plan.query.name(),
        )
    })?;
    let result_set_observation =
        super::super::WorthQueryObservedResultSet::new(super::super::WorthQueryObservedSource {
            expression_context,
            runtime_authority: plan.runtime_authority.as_u64(),
            schema_binding: plan.query.binding_identity().clone(),
            query_identity: plan.query.identity().clone(),
            parameter_binding_identity,
            parameters: source_parameters,
            query_identifier,
            branch,
            selection: basis_identity.selection().clone(),
            model_root: plan.scope.entity_id(),
            source_meaning: result_set_meaning,
            scope_selector,

            _marker: PhantomData,
        });
    let basis_release = plan.basis.release();
    // A producer's exact selected wave may retain the authentic native
    // resources. The receipt separately reports physical release; completing
    // this read requires surrendering its own complete local custody.
    let released = basis_release.custody_released();
    if !released {
        return Err(denial(
            WorthQueryApplicationOneShotDenialKind::BasisReleaseFailed,
            plan.query.name(),
            plan.query.name(),
        )
        .into());
    }
    validate_basis_lifetime(&plan.controls, plan.query.name())?;
    admit_request(request, plan.query.name())?;
    validate_authentication_lifetime(application, plan.principal, plan.query.name())?;

    let projected = project(kernel, &plan.governance, request, plan.query.name());
    decode
        .check(execution)
        .map_err(|cause| value_decode_denial(cause, plan.query.name()))?;
    admit_request(request, plan.query.name())?;
    validate_authentication_lifetime(application, plan.principal, plan.query.name())?;
    let (rows, kernel_receipt) = projected?.into_parts();
    let (read_completion, source_context) = completed::complete_query(
        plan.graph_work,
        read_proof,
        kernel_receipt.observed_graph_read_work(),
        basis_release,
        allocation.is_some(),
        plan.query.name(),
    )?;
    let receipt = WorthQueryApplicationQueryAccessReceipt::from_non_live_kernel(
        WorthQueryApplicationQueryReceiptIdentity {
            query_identity: plan.query.identity().clone(),
            parameter_binding_identity,
            graph_authority_identity: plan.graph_authority_identity,
            provider_identity: plan.provider_identity,
        },
        WorthQueryApplicationQueryReceiptBasis {
            identity: basis_identity,
            version: basis_version,
            posture: plan.controls.basis_posture(),
            lane: plan.controls.lane(),
            consistency: plan.controls.consistency(),
            freshness: plan.controls.freshness(),
            released,
        },
        read_completion,
        plan.canonical_work,
        authorization_work,
        plan.governance.receipt(),
        kernel_receipt,
    );
    Ok(CompletedOneShotRead {
        read: WorthQueryApplicationOneShotResult {
            rows,
            observed_sources,
            result_set_observation,
            request_affinity,
            receipt,
        },
        capture,
        material_source: allocation,
        source_context,
        authorization: plan.authorization,
    })
}
