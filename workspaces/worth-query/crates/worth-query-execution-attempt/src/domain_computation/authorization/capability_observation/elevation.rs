//! Trusted-time binding for the exact selected elevation.

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_relational::facade::authorization::{
    RelationalAuthorizationFieldComparison, RelationalAuthorizationPathPlan,
    RelationalAuthorizationPredicate,
};

use super::super::capability_registry::{
    WorthQueryCapabilityPathTemplate, WorthQueryInstalledCapabilityPlan,
};
use super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRuntimeTimeSample,
};

pub(super) fn prepare_temporal_path(
    installed: &WorthQueryInstalledCapabilityPlan,
    template: &WorthQueryCapabilityPathTemplate,
    sample: &WorthQueryRuntimeTimeSample,
    path_index: usize,
) -> Result<RelationalAuthorizationPathPlan, WorthQueryOperationAuthorizationDenial> {
    let bindings = installed
        .elevation()
        .as_ref()
        .ok_or_else(|| invalid_policy(installed.contract().name()))?;
    let temporal = &bindings.temporal;
    if sample.timeline() != temporal.timeline {
        return Err(invalid_policy(installed.contract().name()));
    }
    let (field, comparison) = if path_index == temporal.not_before_path_index {
        (
            temporal.not_before.clone(),
            RelationalAuthorizationFieldComparison::AtMost,
        )
    } else if path_index == temporal.not_after_path_index {
        (
            temporal.not_after.clone(),
            RelationalAuthorizationFieldComparison::StrictlyGreater,
        )
    } else {
        return Err(invalid_policy(installed.contract().name()));
    };
    let mut predicates = template.plan.predicates().to_vec();
    predicates.push(RelationalAuthorizationPredicate::compare(
        1,
        bindings.elevation_kind,
        field,
        comparison,
        sample.value().clone(),
    ));
    Ok(template.plan.clone().with_predicates(predicates))
}

fn invalid_policy(subject: &str) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InvalidInstalledPolicy,
        ),
        subject,
    )
}
