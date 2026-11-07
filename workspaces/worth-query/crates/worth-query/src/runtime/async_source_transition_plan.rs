use worth_runtime_bridge::facade::{
    BridgeMixedCauseAsyncResultCause, BridgeMixedCauseAsyncResultDisposition,
    BridgeMixedCauseAsyncResultTransition, BridgeMixedCauseDeniedKind, BridgeMixedCauseOrdering,
};

use super::async_result_state::WorthQueryRuntimeAsyncResultProjection;
use super::async_source_binding::{
    transition_basis_identity, transition_generation_identity, WorthQueryRuntimeAsyncSourceBinding,
};
use super::{
    WorthQueryAsyncSourceBindingError, WorthQueryAsyncSourceBindingErrorKind,
    WorthQueryRuntimeAsyncResultStateKind,
};
use crate::WorthQueryEvidenceIdentity;

pub(super) struct WorthQueryPlannedAsyncResultTransition {
    projection: WorthQueryRuntimeAsyncResultProjection,
    basis_identity: WorthQueryEvidenceIdentity,
    generation_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryPlannedAsyncResultTransition {
    pub(super) fn into_parts(
        self,
    ) -> (
        WorthQueryRuntimeAsyncResultProjection,
        WorthQueryEvidenceIdentity,
        WorthQueryEvidenceIdentity,
    ) {
        (
            self.projection,
            self.basis_identity,
            self.generation_identity,
        )
    }
}

pub(super) struct WorthQueryPlannedAsyncResultTransitions {
    binding: WorthQueryRuntimeAsyncSourceBinding,
    transitions: Vec<WorthQueryPlannedAsyncResultTransition>,
    suppressed_duplicate_count: usize,
}

impl WorthQueryPlannedAsyncResultTransitions {
    pub(super) fn into_parts(
        self,
    ) -> (
        WorthQueryRuntimeAsyncSourceBinding,
        Vec<WorthQueryPlannedAsyncResultTransition>,
        usize,
    ) {
        (
            self.binding,
            self.transitions,
            self.suppressed_duplicate_count,
        )
    }
}

pub(super) fn plan_async_result_transitions(
    view_name: &str,
    mut binding: WorthQueryRuntimeAsyncSourceBinding,
    mut prior_kind: Option<WorthQueryRuntimeAsyncResultStateKind>,
    ordering: &BridgeMixedCauseOrdering,
) -> Result<WorthQueryPlannedAsyncResultTransitions, WorthQueryAsyncSourceBindingError> {
    let mut transitions = Vec::new();
    let mut suppressed_duplicate_count = 0;
    for transition in ordering.async_result_transitions() {
        if matches!(
            transition.disposition(),
            BridgeMixedCauseAsyncResultDisposition::DuplicateSuppressed
        ) {
            suppressed_duplicate_count += 1;
            continue;
        }
        validate_disposition(transition)?;
        binding.validate_and_advance(transition)?;
        let basis_identity = transition_basis_identity(transition)?;
        let generation_identity = transition_generation_identity(transition)?;
        if matches!(
            transition.cause(),
            BridgeMixedCauseAsyncResultCause::Revalidation(_)
        ) && matches!(
            prior_kind,
            Some(WorthQueryRuntimeAsyncResultStateKind::Current)
        ) {
            let stale = WorthQueryRuntimeAsyncResultProjection::stale_before_bridge_revalidation(
                transition,
            )?;
            prior_kind = Some(stale.kind());
            transitions.push(WorthQueryPlannedAsyncResultTransition {
                projection: stale,
                basis_identity: basis_identity.clone(),
                generation_identity: generation_identity.clone(),
            });
        }
        let projection =
            WorthQueryRuntimeAsyncResultProjection::from_bridge_transition(transition)?;
        let next_kind = projection.kind();
        if !legal_result_transition(prior_kind, next_kind) {
            return Err(WorthQueryAsyncSourceBindingError::new(
                WorthQueryAsyncSourceBindingErrorKind::IllegalResultTransition,
                format!(
                    "live view `{view_name}` cannot advance async result from {:?} to {}",
                    prior_kind, next_kind
                ),
            ));
        }
        prior_kind = Some(next_kind);
        transitions.push(WorthQueryPlannedAsyncResultTransition {
            projection,
            basis_identity,
            generation_identity,
        });
    }
    Ok(WorthQueryPlannedAsyncResultTransitions {
        binding,
        transitions,
        suppressed_duplicate_count,
    })
}

fn validate_disposition(
    transition: &BridgeMixedCauseAsyncResultTransition,
) -> Result<(), WorthQueryAsyncSourceBindingError> {
    let admitted = matches!(
        (transition.cause(), transition.disposition()),
        (
            BridgeMixedCauseAsyncResultCause::Completion(_),
            BridgeMixedCauseAsyncResultDisposition::Ordered { .. }
        ) | (
            BridgeMixedCauseAsyncResultCause::ClassifiedDenied { .. },
            BridgeMixedCauseAsyncResultDisposition::DeliveryDenied(
                BridgeMixedCauseDeniedKind::AsyncStaleCauseRejected
            )
        ) | (
            BridgeMixedCauseAsyncResultCause::Retry(_)
                | BridgeMixedCauseAsyncResultCause::Revalidation(_),
            BridgeMixedCauseAsyncResultDisposition::DeliveryDenied(
                BridgeMixedCauseDeniedKind::AsyncLineageNonDeliverable
            )
        )
    );
    admitted.then_some(()).ok_or_else(|| {
        binding_error(
            WorthQueryAsyncSourceBindingErrorKind::InadmissibleDisposition,
            transition,
        )
    })
}

fn legal_result_transition(
    prior: Option<WorthQueryRuntimeAsyncResultStateKind>,
    next: WorthQueryRuntimeAsyncResultStateKind,
) -> bool {
    use WorthQueryRuntimeAsyncResultStateKind as Kind;
    matches!(
        (prior, next),
        (
            Some(Kind::Pending),
            Kind::Current
                | Kind::Failed
                | Kind::Cancelled
                | Kind::Stale
                | Kind::Superseded
                | Kind::Denied
                | Kind::Unresolved
        ) | (Some(Kind::Current), Kind::Stale | Kind::Superseded)
            | (Some(Kind::Stale), Kind::Revalidating | Kind::Superseded)
            | (
                Some(Kind::Unresolved),
                Kind::Revalidating | Kind::Stale | Kind::Superseded
            )
            | (Some(Kind::Failed | Kind::Cancelled), Kind::Retried)
            | (
                Some(Kind::Retried | Kind::Revalidating),
                Kind::Current
                    | Kind::Failed
                    | Kind::Cancelled
                    | Kind::Stale
                    | Kind::Superseded
                    | Kind::Denied
                    | Kind::Unresolved
            )
    )
}

fn binding_error(
    kind: WorthQueryAsyncSourceBindingErrorKind,
    transition: &BridgeMixedCauseAsyncResultTransition,
) -> WorthQueryAsyncSourceBindingError {
    WorthQueryAsyncSourceBindingError::new(
        kind,
        format!(
            "bridge async transition `{}` for request `{}` did not match the live binding",
            transition.source_identity(),
            transition.request_identity(),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use WorthQueryRuntimeAsyncResultStateKind as Kind;

    #[test]
    fn unresolved_enters_only_from_active_attempts_and_recovers_through_revalidation() {
        assert!(legal_result_transition(
            Some(Kind::Pending),
            Kind::Unresolved
        ));
        assert!(legal_result_transition(
            Some(Kind::Retried),
            Kind::Unresolved
        ));
        assert!(legal_result_transition(
            Some(Kind::Revalidating),
            Kind::Unresolved
        ));
        assert!(legal_result_transition(
            Some(Kind::Unresolved),
            Kind::Revalidating
        ));
        assert!(legal_result_transition(Some(Kind::Unresolved), Kind::Stale));
        assert!(legal_result_transition(
            Some(Kind::Unresolved),
            Kind::Superseded
        ));
        assert!(!legal_result_transition(
            Some(Kind::Unresolved),
            Kind::Current
        ));
        assert!(!legal_result_transition(
            Some(Kind::Current),
            Kind::Unresolved
        ));
    }
}
