use std::marker::PhantomData;

use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::custody_work::RetainedCustodyWork;
use super::{
    admit_request, denial, validate_authentication_lifetime, validate_basis_lifetime,
    WorthQueryApplicationOneShotDenial, WorthQueryApplicationOneShotDenialKind,
    WorthQueryApplicationOneShotResult,
};
use crate::domain_computation::primary_graph::application_query::{
    access_receipt::{
        WorthQueryApplicationQueryReceiptBasis, WorthQueryApplicationQueryReceiptIdentity,
    },
    read_execution::{
        project_non_live_kernel, OneShotReadWorkObservation, RawNonLiveKernelOutcome,
    },
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationAuthorizationWorkEvidence,
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessReceipt,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(super) fn finalize_one_shot<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
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
    spent: Option<&OneShotReadWorkObservation>,
) -> Result<
    WorthQueryApplicationOneShotResult<Query, QueryResult>,
    WorthQueryApplicationOneShotDenial,
>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    let request_affinity =
        super::super::admitted_result::WorthQueryApplicationQueryRequestAffinity::new(
            plan.principal,
            plan.controls.request_scope(),
        );
    let mut source_footprints = std::mem::take(&mut kernel.raw.source_footprints);
    let result_set_selection = std::sync::Arc::clone(
        kernel
            .raw
            .result_set_source
            .as_ref()
            .expect("one-shot root selection must retain result-set evidence"),
    );
    // A single row can feed an output demand. Its output depends on the
    // complete query selection as well as the row: a newly matching root must
    // invalidate reuse even when this row's local path remains unchanged.
    if let [footprint] = source_footprints.as_mut_slice() {
        footprint.root_selection = Some(std::sync::Arc::clone(&result_set_selection));
    }
    let request = plan.controls.request_scope();
    let basis_identity = plan.basis.identity().clone();
    let basis_version = plan.basis.version_id();
    let parameter_binding_identity = *plan.parameters.identity();
    let scope_locator = plan.scope.identity_locator();
    let scope_value = plan.scope.identity_value();
    let source_slots = source_footprints
        .len()
        .checked_mul(std::mem::size_of::<
            super::super::WorthQueryObservedSource<Query>,
        >())
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
    let scope_bytes =
        super::super::observed_source::WorthQueryObservedScopeSelector::charged_bytes(
            scope_locator,
            scope_value,
        )
        .and_then(|bytes| bytes.checked_add(source_slots))
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
    // Source slots reserve retained Vec backing. Move the admitted parameter
    // basis into shared source custody, so its original backing stays charged
    // without a second canonical clone or serialization pass.
    let parameter_heap = plan
        .parameters
        .retained_owned_capacity_bytes()
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
    let parameter_bytes = parameter_heap
        .checked_add(std::mem::size_of::<
            super::super::observed_source::WorthQueryRetainedObservedParameters,
        >())
        .and_then(|bytes| bytes.checked_add(2 * std::mem::size_of::<usize>()))
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
    let descriptor_bytes = plan
        .query
        .name()
        .len()
        .checked_add(basis_identity.branch_id().0.len())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<String>()))
        .and_then(|bytes| {
            bytes.checked_add(std::mem::size_of::<
                worth_relational::facade::history::BranchId,
            >())
        })
        .and_then(|bytes| {
            bytes.checked_add(std::mem::size_of::<
                super::super::resource_lifecycle::WorthQueryRetainedSourceCharge,
            >())
        })
        .and_then(|bytes| bytes.checked_add(6 * std::mem::size_of::<usize>()))
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
    // Custody copies are retention, not reading: the receipt's work and the
    // Query's declared limit cover only what the read examined.
    let custody = RetainedCustodyWork::of(
        scope_locator,
        scope_value,
        plan.query.name(),
        basis_identity.branch_id(),
    )
    .ok_or_else(|| {
        denial(
            WorthQueryApplicationOneShotDenialKind::WorkLimitExceeded,
            plan.query.name(),
            plan.query.name(),
        )
    })?;
    admit_request(request, plan.query.name())?;
    let scope_charge = kernel
        .result_buffer
        .claim_retained_source(scope_bytes)
        .map_err(|()| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
    let scope_selector = std::sync::Arc::new(
        super::super::observed_source::WorthQueryObservedScopeSelector::new(
            scope_locator,
            scope_value,
            scope_charge,
        ),
    );
    if let Some(spent) = spent {
        spent.scope(custody.scope);
    }
    let parameter_charge = kernel
        .result_buffer
        .claim_retained_source(parameter_bytes)
        .map_err(|()| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
    let descriptor_charge = kernel
        .result_buffer
        .claim_retained_source(descriptor_bytes)
        .map_err(|()| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
    let source_parameters = std::sync::Arc::new(
        super::super::observed_source::WorthQueryRetainedObservedParameters::new(
            plan.parameters,
            parameter_charge,
        ),
    );
    let query_identifier = std::sync::Arc::new(plan.query.name().to_owned());
    let branch = std::sync::Arc::new(basis_identity.branch_id().clone());
    if let Some(spent) = spent {
        spent.descriptor(custody.descriptor);
    }
    let descriptor_charge = std::sync::Arc::new(descriptor_charge);
    let mut observed_sources = Vec::with_capacity(source_footprints.len());
    for footprint in source_footprints {
        admit_request(request, plan.query.name())?;
        let selection = basis_identity.selection().clone();
        let source_meaning = application
            .source_meanings
            .intern(
                plan.query.identity().as_bytes(),
                parameter_binding_identity.bytes(),
                footprint,
                &selection,
            )
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationOneShotDenialKind::SourceIdentityExhausted,
                    plan.query.name(),
                    plan.query.name(),
                )
            })?;
        observed_sources.push(super::super::WorthQueryObservedSource {
            runtime_authority: plan.runtime_authority.as_u64(),
            schema_binding: plan.query.binding_identity().clone(),
            query_identity: plan.query.identity().clone(),
            parameter_binding_identity,
            parameters: std::sync::Arc::clone(&source_parameters),
            query_identifier: std::sync::Arc::clone(&query_identifier),
            branch: std::sync::Arc::clone(&branch),
            selection,
            model_root: plan.scope.entity_id(),
            source_meaning,
            scope_selector: std::sync::Arc::clone(&scope_selector),
            _descriptor_charge: std::sync::Arc::clone(&descriptor_charge),
            _marker: PhantomData,
        });
    }
    let result_set_footprint = super::super::observed_source::WorthQueryObservedSourceFootprint {
        root: plan.scope.entity_id(),
        complete: true,
        entities: Vec::new(),
        aspects: Vec::new(),
        adjacencies: Vec::new(),
        root_selection: Some(result_set_selection),
    };
    let result_set_meaning = application
        .source_meanings
        .intern_result_set(
            plan.query.identity().as_bytes(),
            parameter_binding_identity.bytes(),
            result_set_footprint,
            basis_identity.selection(),
        )
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationOneShotDenialKind::SourceIdentityExhausted,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
    let result_set_observation =
        super::super::WorthQueryObservedResultSet::new(super::super::WorthQueryObservedSource {
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
            _descriptor_charge: descriptor_charge,
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
        ));
    }
    validate_basis_lifetime(&plan.controls, plan.query.name())?;
    admit_request(request, plan.query.name())?;
    validate_authentication_lifetime(application, plan.principal, plan.query.name())?;

    let projected = project_non_live_kernel::<Schema, Query, QueryResult, _>(
        kernel,
        &plan.governance,
        || admit_request(request, plan.query.name()),
        |projection: crate::domain_computation::primary_graph::WorthQueryApplicationProjectionDenial| {
            denial(
                WorthQueryApplicationOneShotDenialKind::Projection(projection.kind()),
                plan.query.name(),
                projection.subject(),
            )
        },
    )?;
    admit_request(request, plan.query.name())?;
    validate_authentication_lifetime(application, plan.principal, plan.query.name())?;
    let (rows, kernel_receipt) = projected.into_parts();
    let read_completion = plan
        .graph_work
        .complete_query_read(
            read_proof,
            kernel_receipt.observed_graph_read_work(),
            basis_release,
        )
        .map_err(|_| {
            denial(
                WorthQueryApplicationOneShotDenialKind::ForeignPlan,
                plan.query.name(),
                plan.query.name(),
            )
        })?;
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
    Ok(WorthQueryApplicationOneShotResult {
        rows,
        observed_sources,
        result_set_observation,
        request_affinity,
        receipt,
    })
}
