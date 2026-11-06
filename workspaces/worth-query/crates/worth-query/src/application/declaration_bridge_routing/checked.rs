use crate::application::{
    WorthQueryDeclarationAspectFit, WorthQueryDeclarationEnvelope,
    WorthQueryDeclarationEnvelopeChecked, WorthQueryDeclarationEnvelopeDeferred,
    WorthQueryDeclarationEnvelopeDenied, WorthQueryDeclarationEnvelopeFailed,
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
    WorthQueryLowerAuthorityRouteFamily,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::{
    artifact::{WorthQueryDeclarationBridgeRouting, WorthQueryDeclarationBridgeRoutingClass},
    aspect_gate::BridgeAuthorityAspectGate,
    checked_input::{deny_non_success_mismatch, lower_checked_input},
    contract::WorthQueryDeclarationBridgeRoutingSupportStatus,
    denial::{
        WorthQueryDeclarationBridgeRoutingDeferred, WorthQueryDeclarationBridgeRoutingDenialCause,
        WorthQueryDeclarationBridgeRoutingDenied, WorthQueryDeclarationBridgeRoutingFailed,
    },
    digest::derive_bridge_routing_digest,
    explain::WorthQueryDeclarationBridgeRoutingExplanation,
    handle_gate::{envelope_matches_handle, subject_matches_handle},
    lower::worth_query_lower_bridge_binding,
};

pub enum WorthQueryDeclarationBridgeRoutingInput<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    Enveloped(WorthQueryDeclarationEnvelope<D, I>),
    Deferred(WorthQueryDeclarationEnvelopeDeferred<D, I>),
    Denied(WorthQueryDeclarationEnvelopeDenied<D, I>),
    Failed(WorthQueryDeclarationEnvelopeFailed<D, I>),
    EnvelopeChecked(WorthQueryDeclarationEnvelopeChecked<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationBridgeRoutingInput<D, I>
{
    pub fn enveloped(envelope: WorthQueryDeclarationEnvelope<D, I>) -> Self {
        Self::Enveloped(envelope)
    }

    pub fn deferred(envelope: WorthQueryDeclarationEnvelopeDeferred<D, I>) -> Self {
        Self::Deferred(envelope)
    }

    pub fn denied(envelope: WorthQueryDeclarationEnvelopeDenied<D, I>) -> Self {
        Self::Denied(envelope)
    }

    pub fn failed(envelope: WorthQueryDeclarationEnvelopeFailed<D, I>) -> Self {
        Self::Failed(envelope)
    }

    pub fn envelope_checked(checked: WorthQueryDeclarationEnvelopeChecked<D, I>) -> Self {
        Self::EnvelopeChecked(checked)
    }
}

pub enum WorthQueryDeclarationBridgeRoutingChecked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    Routed(WorthQueryDeclarationBridgeRouting<D, I>),
    Deferred(WorthQueryDeclarationBridgeRoutingDeferred<D, I>),
    Denied(WorthQueryDeclarationBridgeRoutingDenied<D, I>),
    Failed(WorthQueryDeclarationBridgeRoutingFailed<D, I>),
}

pub(crate) fn worth_query_checked_declaration_bridge_routing<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    input: WorthQueryDeclarationBridgeRoutingInput<D, I>,
) -> Result<WorthQueryDeclarationBridgeRoutingChecked<D, I>, IdentityDerivationDenial> {
    match input {
        WorthQueryDeclarationBridgeRoutingInput::EnvelopeChecked(checked) => {
            worth_query_checked_declaration_bridge_routing(lower_checked_input(checked))?
        }
        WorthQueryDeclarationBridgeRoutingInput::Enveloped(envelope) => route_enveloped(envelope)?,
        WorthQueryDeclarationBridgeRoutingInput::Deferred(envelope) => {
            let reason = envelope.reason();
            WorthQueryDeclarationBridgeRoutingChecked::Deferred(
                WorthQueryDeclarationBridgeRoutingDeferred::new(
                    envelope.into_envelope(),
                    reason,
                ),
            )
        }
        WorthQueryDeclarationBridgeRoutingInput::Denied(envelope) => {
            WorthQueryDeclarationBridgeRoutingChecked::Denied(
                WorthQueryDeclarationBridgeRoutingDenied::new(
                    envelope.into_envelope(),
                    I::Family::bridge_continuation_contract().map(|contract| contract.request()),
                    I::Family::bridge_continuation_contract().map(|contract| contract.family()),
                    WorthQueryDeclarationBridgeRoutingDenialCause::EnvelopeNotCoveredForBridgeRouting,
                ),
            )
        }
        WorthQueryDeclarationBridgeRoutingInput::Failed(envelope) => {
            let reason = envelope.reason();
            WorthQueryDeclarationBridgeRoutingChecked::Failed(
                WorthQueryDeclarationBridgeRoutingFailed::new(
                    envelope.into_envelope(),
                    reason,
                ),
            )
        }
    }

    Ok(())
}

pub(crate) fn worth_query_checked_declaration_bridge_routing_on_handle<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    handle_identity_digest: &str,
    operating_context_identity_digest: &str,
    support_status: WorthQueryDeclarationBridgeRoutingSupportStatus,
    input: WorthQueryDeclarationBridgeRoutingInput<D, I>,
) -> Result<WorthQueryDeclarationBridgeRoutingChecked<D, I>, IdentityDerivationDenial> {
    match input {
        WorthQueryDeclarationBridgeRoutingInput::EnvelopeChecked(checked) => {
            worth_query_checked_declaration_bridge_routing_on_handle(
                handle_identity_digest,
                operating_context_identity_digest,
                support_status,
                lower_checked_input(checked),
            )?
        }
        WorthQueryDeclarationBridgeRoutingInput::Enveloped(envelope) => {
            if !envelope_matches_handle(
                handle_identity_digest,
                operating_context_identity_digest,
                &envelope,
            ) {
                return Ok(WorthQueryDeclarationBridgeRoutingChecked::Denied(
                    WorthQueryDeclarationBridgeRoutingDenied::new(
                        envelope,
                        I::Family::bridge_continuation_contract()
                            .map(|contract| contract.request()),
                        I::Family::bridge_continuation_contract().map(|contract| contract.family()),
                        WorthQueryDeclarationBridgeRoutingDenialCause::BridgeEnvelopeMismatch,
                    ),
                ));
            }
            if support_status != WorthQueryDeclarationBridgeRoutingSupportStatus::Admitted {
                return Ok(WorthQueryDeclarationBridgeRoutingChecked::Denied(
                    WorthQueryDeclarationBridgeRoutingDenied::new(
                        envelope,
                        I::Family::bridge_continuation_contract()
                            .map(|contract| contract.request()),
                        I::Family::bridge_continuation_contract().map(|contract| contract.family()),
                        WorthQueryDeclarationBridgeRoutingDenialCause::BridgeAuthorityUnavailable,
                    ),
                ));
            }
            route_enveloped(envelope)?
        }
        other => {
            if !subject_matches_handle(
                handle_identity_digest,
                operating_context_identity_digest,
                &other,
            ) {
                return Ok(deny_non_success_mismatch(other));
            }
            worth_query_checked_declaration_bridge_routing(other)?
        }
    }

    Ok(())
}

fn route_enveloped<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    envelope: WorthQueryDeclarationEnvelope<D, I>,
) -> Result<WorthQueryDeclarationBridgeRoutingChecked<D, I>, IdentityDerivationDenial> {
    let Some(contract) = I::Family::bridge_continuation_contract() else {
        return Ok(WorthQueryDeclarationBridgeRoutingChecked::Denied(
            WorthQueryDeclarationBridgeRoutingDenied::new(
                envelope,
                None,
                None,
                WorthQueryDeclarationBridgeRoutingDenialCause::UnsupportedContinuationMode,
            ),
        ));
    };
    let denied_request = Some(contract.request());
    let denied_family = Some(contract.family());
    let Some(route_plan) = envelope.route_plan() else {
        return Ok(WorthQueryDeclarationBridgeRoutingChecked::Denied(
            WorthQueryDeclarationBridgeRoutingDenied::new(
                envelope,
                denied_request,
                denied_family,
                WorthQueryDeclarationBridgeRoutingDenialCause::BridgeEnvelopeMismatch,
            ),
        ));
    };
    if !route_plan
        .route_families()
        .contains(&WorthQueryLowerAuthorityRouteFamily::Bridge)
    {
        return Ok(WorthQueryDeclarationBridgeRoutingChecked::Denied(
            WorthQueryDeclarationBridgeRoutingDenied::new(
                envelope,
                denied_request,
                denied_family,
                WorthQueryDeclarationBridgeRoutingDenialCause::NonBridgeRoutePlan,
            ),
        ));
    }

    let authority_aspects =
        BridgeAuthorityAspectGate::from_envelope(&envelope, contract.required_aspects());
    match authority_aspects.fit() {
        WorthQueryDeclarationAspectFit::Conflict => {
            return Ok(WorthQueryDeclarationBridgeRoutingChecked::Denied(
                WorthQueryDeclarationBridgeRoutingDenied::new(
                    envelope,
                    denied_request,
                    denied_family,
                    WorthQueryDeclarationBridgeRoutingDenialCause::AspectConflict,
                ),
            ));
        }
        WorthQueryDeclarationAspectFit::MissingRequired => {
            return Ok(WorthQueryDeclarationBridgeRoutingChecked::Denied(
                WorthQueryDeclarationBridgeRoutingDenied::new(
                    envelope,
                    denied_request,
                    denied_family,
                    WorthQueryDeclarationBridgeRoutingDenialCause::MissingRequiredAspect,
                ),
            ));
        }
        WorthQueryDeclarationAspectFit::Partial => {
            return Ok(WorthQueryDeclarationBridgeRoutingChecked::Denied(
                WorthQueryDeclarationBridgeRoutingDenied::new(
                    envelope,
                    denied_request,
                    denied_family,
                    WorthQueryDeclarationBridgeRoutingDenialCause::AuthorityAspectGap,
                ),
            ));
        }
        WorthQueryDeclarationAspectFit::Exact
        | WorthQueryDeclarationAspectFit::CompatibleSuperset => {}
    }
    if matches!(
        authority_aspects.mapping_fit(),
        WorthQueryDeclarationAspectFit::Partial
    ) {
        return Ok(WorthQueryDeclarationBridgeRoutingChecked::Denied(
            WorthQueryDeclarationBridgeRoutingDenied::new(
                envelope,
                denied_request,
                denied_family,
                WorthQueryDeclarationBridgeRoutingDenialCause::AuthorityAspectGap,
            ),
        ));
    }

    let mixed_origin = route_plan.route_count() > 1;
    let future_projection = route_plan.future_projection().clone();
    let basis_lifecycle_support_digest = route_plan
        .progressed_declaration()
        .retained_world_basis()
        .basis_lifecycle_support_for_reporting()
        .to_string();
    let class = if mixed_origin {
        WorthQueryDeclarationBridgeRoutingClass::MixedAuthorityBridgeContinuation
    } else {
        WorthQueryDeclarationBridgeRoutingClass::ExclusiveBridgeContinuation
    };
    let (continuation_request, continuation_family, binding) =
        worth_query_lower_bridge_binding(&envelope, contract)?;
    let digest = derive_bridge_routing_digest(
        &envelope,
        class,
        continuation_request,
        continuation_family,
        binding.surface(),
        authority_aspects.contract(),
        authority_aspects.coverage(),
        authority_aspects.coverage_basis(),
        authority_aspects.fit(),
        authority_aspects.mapped_aspects(),
        authority_aspects.mapping_fit(),
        envelope.route_denial_cause(),
        envelope.receipt_denial_cause(),
        &future_projection,
        &basis_lifecycle_support_digest,
    )?;
    let mut retained_truths = envelope.explain().retained_truths().to_vec();
    retained_truths.push(format!(
        "bridge-continuation-mode:{}",
        continuation_request.mode().as_str()
    ));
    retained_truths.push(format!(
        "bridge-truth-context:{}",
        continuation_request.truth_context().as_str()
    ));
    retained_truths.push(format!(
        "bridge-continuation-family:{}",
        continuation_family.as_str()
    ));
    retained_truths.push(format!("bridge-aspect-fit:{:?}", authority_aspects.fit()));
    retained_truths.push(format!(
        "bridge-mapping-fit:{:?}",
        authority_aspects.mapping_fit()
    ));
    retained_truths.extend(future_projection.retained_facts());
    retained_truths.push(format!(
        "basis-lifecycle-support:{}",
        basis_lifecycle_support_digest
    ));
    let explanation = WorthQueryDeclarationBridgeRoutingExplanation::new(
        envelope.explain().crossing_posture(),
        continuation_request.mode(),
        continuation_request.truth_context(),
        continuation_family,
        binding.surface(),
        retained_truths,
        envelope
            .explain()
            .route_governing_reason()
            .map(ToOwned::to_owned),
        envelope.route_denial_cause(),
        envelope.explain().receipt_governing_reason().to_string(),
        envelope.receipt_denial_cause(),
        envelope.evidence_origin(),
        mixed_origin,
    );
    Ok(WorthQueryDeclarationBridgeRoutingChecked::Routed(
        WorthQueryDeclarationBridgeRouting::new(
            class,
            continuation_request,
            continuation_family,
            binding,
            authority_aspects.contract().clone(),
            authority_aspects.coverage().clone(),
            authority_aspects.coverage_basis(),
            authority_aspects.fit(),
            authority_aspects.mapped_aspects().clone(),
            authority_aspects.mapping_fit(),
            future_projection,
            basis_lifecycle_support_digest,
            envelope,
            digest,
            explanation,
        ),
    ))
}
