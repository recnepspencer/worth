use super::super::{WorthQueryCurrentOutputDenial, WorthQueryCurrentOutputDenialKind};
use crate::domain_computation::primary_graph::WorthQueryInvariantDecisionPlanDenialKind;

pub(super) fn decision_plan_denial(
    kind: WorthQueryInvariantDecisionPlanDenialKind,
    subject: &str,
) -> WorthQueryCurrentOutputDenial {
    let kind = match kind {
        WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity => {
            WorthQueryCurrentOutputDenialKind::ForeignIdentity
        }
        WorthQueryInvariantDecisionPlanDenialKind::UndeclaredDecisionTarget
        | WorthQueryInvariantDecisionPlanDenialKind::FieldNotInstalled => {
            WorthQueryCurrentOutputDenialKind::UndeclaredDecisionTarget
        }
    };
    WorthQueryCurrentOutputDenial::new(kind, subject)
}
