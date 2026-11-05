//! Actual historical candidate matching; exact rejection never discards influence.
use super::super::source::service::ManagedSourceRoot;
use super::CapturedIndexInfluence;
use crate::domain_computation::primary_graph::access_strategy::realized_footprint::ManagedHistoryRelevanceDenial as Denial;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationQueryDisclosurePosture},
    application_schema::ApplicationSchema,
};
use worth_relational::facade::history::RelationalAuthoredManagedSourceImpact as Impact;
use {
    crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial,
    crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop,
};

pub(super) fn relevant<S: ApplicationSchema + 'static, I: ApplicationManagedIndexBinding<S>>(
    capture: &CapturedIndexInfluence<S, I>,
    execution: &ExecutionRequest<'_, '_>,
    impact: Impact,
) -> Result<bool, Denial> {
    // This first historical matcher qualifies Public Parent consistency only.
    // Governed influence keeps the existing conservative candidate posture.
    if capture
        .parameters
        .parent()
        .access_contract()
        .meaning()
        .parent_disclosure_posture()
        != ApplicationQueryDisclosurePosture::Public
    {
        return Ok(true);
    }
    let material = match impact {
        Impact::Unaffected => return Ok(false),
        Impact::Conservative => return Ok(true),
        Impact::Retained(material) => material,
    };
    let foreign = || {
        Denial::Provider(WorthQueryManagedIndexWorkDenial::Controlled(
            Stop::ForeignCoverage,
        ))
    };
    // Strong shares stay in this outside-gate frame, including every error and
    // unwind. Expiration is conservative and never an empty historical Source.
    let Some(payloads) = material
        .acquire_payloads(*execution)
        .map_err(Denial::Native)?
    else {
        return Ok(true);
    };
    let before = payloads
        .before()
        .downcast_ref::<ManagedSourceRoot<S, I>>()
        .ok_or_else(foreign)?;
    let after = payloads
        .after()
        .downcast_ref::<ManagedSourceRoot<S, I>>()
        .ok_or_else(foreign)?;
    if material.scope() != capture.scope {
        return Err(foreign());
    }
    // Validate BOTH associations before an early positive predicate result.
    for root in [before, after] {
        if root.rows.program() != material.program()
            || root.rows.scope() != capture.scope
            || !root
                .installed
                .same_installation(capture.adapter.installed())
            || !Arc::ptr_eq(&root.adapter, &capture.adapter)
        {
            return Err(foreign());
        }
    }
    capture
        .adapter
        .historical_candidate_relevant(
            execution,
            before.rows.view(&before.bindings, None),
            after.rows.view(&after.bindings, None),
            capture.parameters.raw(),
            &material,
        )
        .map_err(Denial::Provider)
}
