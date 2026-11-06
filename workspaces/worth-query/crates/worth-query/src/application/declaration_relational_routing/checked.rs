use crate::application::{
    WorthQueryDeclarationAspectFit, WorthQueryDeclarationEnvelopeChecked,
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
    WorthQueryLowerAuthorityRouteFamily,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::{
    artifact::{
        WorthQueryDeclarationRelationalRouting, WorthQueryDeclarationRelationalRoutingClass,
    },
    aspect_gate::RelationalAuthorityAspectGate,
    contract::WorthQueryDeclarationRelationalTruthRoutingSupportStatus,
    denial::{
        WorthQueryDeclarationRelationalRoutingDeferred,
        WorthQueryDeclarationRelationalRoutingDenialCause,
        WorthQueryDeclarationRelationalRoutingDenied, WorthQueryDeclarationRelationalRoutingFailed,
    },
    digest::derive_relational_routing_digest,
    explain::WorthQueryDeclarationRelationalRoutingExplanation,
    handle_gate::{envelope_matches_handle, subject_matches_handle},
    input::WorthQueryDeclarationRelationalRoutingInput,
    lower::worth_query_lower_relational_binding,
};

pub enum WorthQueryDeclarationRelationalRoutingChecked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    Routed(WorthQueryDeclarationRelationalRouting<D, I>),
    Deferred(WorthQueryDeclarationRelationalRoutingDeferred<D, I>),
    Denied(WorthQueryDeclarationRelationalRoutingDenied<D, I>),
    Failed(WorthQueryDeclarationRelationalRoutingFailed<D, I>),
}

pub(crate) fn worth_query_checked_declaration_relational_routing<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    input: WorthQueryDeclarationRelationalRoutingInput<D, I>,
) -> Result<WorthQueryDeclarationRelationalRoutingChecked<D, I>, IdentityDerivationDenial> {
    match input {
        WorthQueryDeclarationRelationalRoutingInput::EnvelopeChecked(checked) => {
            worth_query_checked_declaration_relational_routing(lower_checked_input(checked))?
        }
        WorthQueryDeclarationRelationalRoutingInput::Enveloped(envelope) => route_enveloped(envelope)?,
        WorthQueryDeclarationRelationalRoutingInput::Deferred(envelope) => {
            let reason = envelope.reason();
            WorthQueryDeclarationRelationalRoutingChecked::Deferred(
                WorthQueryDeclarationRelationalRoutingDeferred::new(
                    envelope.into_envelope(),
                    reason,
                ),
            )
        }
        WorthQueryDeclarationRelationalRoutingInput::Denied(envelope) => {
            WorthQueryDeclarationRelationalRoutingChecked::Denied(
                WorthQueryDeclarationRelationalRoutingDenied::new(
                    envelope.into_envelope(),
                    I::Family::relational_truth_contract().map(|contract| contract.truth_claim()),
                    I::Family::relational_truth_contract()
                        .map(|contract| contract.authority_family()),
                    WorthQueryDeclarationRelationalRoutingDenialCause::EnvelopeNotCoveredForRelationalRouting,
                ),
            )
        }
        WorthQueryDeclarationRelationalRoutingInput::Failed(envelope) => {
            let reason = envelope.reason();
            WorthQueryDeclarationRelationalRoutingChecked::Failed(
                WorthQueryDeclarationRelationalRoutingFailed::new(
                    envelope.into_envelope(),
                    reason,
                ),
            )
        }
    }

    Ok(())
}

pub(crate) fn worth_query_checked_declaration_relational_routing_on_handle<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    handle_identity_digest: &str,
    operating_context_identity_digest: &str,
    support_status: WorthQueryDeclarationRelationalTruthRoutingSupportStatus,
    input: WorthQueryDeclarationRelationalRoutingInput<D, I>,
) -> Result<WorthQueryDeclarationRelationalRoutingChecked<D, I>, IdentityDerivationDenial> {
    match input {
        WorthQueryDeclarationRelationalRoutingInput::EnvelopeChecked(checked) => {
            worth_query_checked_declaration_relational_routing_on_handle(
                handle_identity_digest,
                operating_context_identity_digest,
                support_status,
                lower_checked_input(checked),
            )?
        }
        WorthQueryDeclarationRelationalRoutingInput::Enveloped(envelope) => {
            if !envelope_matches_handle(
                handle_identity_digest,
                operating_context_identity_digest,
                &envelope,
            ) {
                return Ok(WorthQueryDeclarationRelationalRoutingChecked::Denied(
                    WorthQueryDeclarationRelationalRoutingDenied::new(
                        envelope,
                        I::Family::relational_truth_contract().map(|contract| contract.truth_claim()),
                        I::Family::relational_truth_contract()
                            .map(|contract| contract.authority_family()),
                        WorthQueryDeclarationRelationalRoutingDenialCause::RelationalEnvelopeMismatch,
                    ),
                ));
            }
            if support_status != WorthQueryDeclarationRelationalTruthRoutingSupportStatus::Admitted
            {
                return Ok(WorthQueryDeclarationRelationalRoutingChecked::Denied(
                    WorthQueryDeclarationRelationalRoutingDenied::new(
                        envelope,
                        I::Family::relational_truth_contract().map(|contract| contract.truth_claim()),
                        I::Family::relational_truth_contract()
                            .map(|contract| contract.authority_family()),
                        WorthQueryDeclarationRelationalRoutingDenialCause::RelationalAuthorityUnavailable,
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
            worth_query_checked_declaration_relational_routing(other)?
        }
    }

    Ok(())
}

fn route_enveloped<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    envelope: crate::application::WorthQueryDeclarationEnvelope<D, I>,
) -> WorthQueryDeclarationRelationalRoutingChecked<D, I> {
    let Some(contract) = I::Family::relational_truth_contract() else {
        return WorthQueryDeclarationRelationalRoutingChecked::Denied(
            WorthQueryDeclarationRelationalRoutingDenied::new(
                envelope,
                None,
                None,
                WorthQueryDeclarationRelationalRoutingDenialCause::UnsupportedRelationalTruthClaim,
            ),
        );
    };
    let denied_truth_claim = Some(contract.truth_claim());
    let denied_authority_family = Some(contract.authority_family());
    let Some(route_plan) = envelope.route_plan() else {
        return WorthQueryDeclarationRelationalRoutingChecked::Denied(
            WorthQueryDeclarationRelationalRoutingDenied::new(
                envelope,
                denied_truth_claim,
                denied_authority_family,
                WorthQueryDeclarationRelationalRoutingDenialCause::RelationalEnvelopeMismatch,
            ),
        );
    };
    if !route_plan
        .route_families()
        .contains(&WorthQueryLowerAuthorityRouteFamily::Relational)
    {
        return WorthQueryDeclarationRelationalRoutingChecked::Denied(
            WorthQueryDeclarationRelationalRoutingDenied::new(
                envelope,
                denied_truth_claim,
                denied_authority_family,
                WorthQueryDeclarationRelationalRoutingDenialCause::NonRelationalRoutePlan,
            ),
        );
    }

    let authority_aspects =
        RelationalAuthorityAspectGate::from_envelope(&envelope, contract.required_aspects());
    match authority_aspects.fit() {
        WorthQueryDeclarationAspectFit::Conflict => {
            return WorthQueryDeclarationRelationalRoutingChecked::Denied(
                WorthQueryDeclarationRelationalRoutingDenied::new(
                    envelope,
                    denied_truth_claim,
                    denied_authority_family,
                    WorthQueryDeclarationRelationalRoutingDenialCause::AspectConflict,
                ),
            );
        }
        WorthQueryDeclarationAspectFit::MissingRequired => {
            return WorthQueryDeclarationRelationalRoutingChecked::Denied(
                WorthQueryDeclarationRelationalRoutingDenied::new(
                    envelope,
                    denied_truth_claim,
                    denied_authority_family,
                    WorthQueryDeclarationRelationalRoutingDenialCause::MissingRequiredAspect,
                ),
            );
        }
        WorthQueryDeclarationAspectFit::Partial => {
            return WorthQueryDeclarationRelationalRoutingChecked::Denied(
                WorthQueryDeclarationRelationalRoutingDenied::new(
                    envelope,
                    denied_truth_claim,
                    denied_authority_family,
                    WorthQueryDeclarationRelationalRoutingDenialCause::RelationalAspectGap,
                ),
            );
        }
        WorthQueryDeclarationAspectFit::Exact
        | WorthQueryDeclarationAspectFit::CompatibleSuperset => {}
    }

    let mixed_origin = route_plan.route_count() > 1;
    let class = if mixed_origin {
        WorthQueryDeclarationRelationalRoutingClass::MixedAuthorityRelationalTruth
    } else {
        WorthQueryDeclarationRelationalRoutingClass::ExclusiveRelationalTruth
    };
    let (truth_claim, authority_family, binding) =
        worth_query_lower_relational_binding(&envelope, contract);
    let digest = derive_relational_routing_digest(
        &envelope,
        class,
        truth_claim,
        authority_family,
        binding.surface(),
        authority_aspects.contract(),
        authority_aspects.coverage(),
        authority_aspects.coverage_basis(),
        authority_aspects.fit(),
        envelope.route_denial_cause(),
        envelope.receipt_denial_cause(),
    );
    let mut retained_truths = envelope.explain().retained_truths().to_vec();
    retained_truths.push(format!("relational-truth-claim:{}", truth_claim.as_str()));
    retained_truths.push(format!(
        "relational-authority-family:{}",
        authority_family.as_str()
    ));
    retained_truths.push(format!(
        "relational-aspect-fit:{:?}",
        authority_aspects.fit()
    ));
    retained_truths.push(format!(
        "relational-aspect-coverage-basis:{:?}",
        authority_aspects.coverage_basis()
    ));
    let explanation = WorthQueryDeclarationRelationalRoutingExplanation::new(
        envelope.explain().crossing_posture(),
        truth_claim,
        authority_family,
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
    WorthQueryDeclarationRelationalRoutingChecked::Routed(
        WorthQueryDeclarationRelationalRouting::new(
            class,
            truth_claim,
            authority_family,
            binding,
            authority_aspects.contract().clone(),
            authority_aspects.coverage().clone(),
            authority_aspects.coverage_basis(),
            authority_aspects.fit(),
            envelope,
            digest,
            explanation,
        ),
    )
}

fn lower_checked_input<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    checked: WorthQueryDeclarationEnvelopeChecked<D, I>,
) -> WorthQueryDeclarationRelationalRoutingInput<D, I> {
    match checked {
        WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope) => {
            WorthQueryDeclarationRelationalRoutingInput::enveloped(envelope)
        }
        WorthQueryDeclarationEnvelopeChecked::Deferred(envelope) => {
            WorthQueryDeclarationRelationalRoutingInput::deferred(envelope)
        }
        WorthQueryDeclarationEnvelopeChecked::Denied(envelope) => {
            WorthQueryDeclarationRelationalRoutingInput::denied(envelope)
        }
        WorthQueryDeclarationEnvelopeChecked::Failed(envelope) => {
            WorthQueryDeclarationRelationalRoutingInput::failed(envelope)
        }
    }
}

fn deny_non_success_mismatch<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    input: WorthQueryDeclarationRelationalRoutingInput<D, I>,
) -> WorthQueryDeclarationRelationalRoutingChecked<D, I> {
    match input {
        WorthQueryDeclarationRelationalRoutingInput::Deferred(envelope) => {
            WorthQueryDeclarationRelationalRoutingChecked::Denied(
                WorthQueryDeclarationRelationalRoutingDenied::new(
                    envelope.into_envelope(),
                    I::Family::relational_truth_contract().map(|contract| contract.truth_claim()),
                    I::Family::relational_truth_contract()
                        .map(|contract| contract.authority_family()),
                    WorthQueryDeclarationRelationalRoutingDenialCause::RelationalEnvelopeMismatch,
                ),
            )
        }
        WorthQueryDeclarationRelationalRoutingInput::Denied(envelope) => {
            WorthQueryDeclarationRelationalRoutingChecked::Denied(
                WorthQueryDeclarationRelationalRoutingDenied::new(
                    envelope.into_envelope(),
                    I::Family::relational_truth_contract().map(|contract| contract.truth_claim()),
                    I::Family::relational_truth_contract()
                        .map(|contract| contract.authority_family()),
                    WorthQueryDeclarationRelationalRoutingDenialCause::RelationalEnvelopeMismatch,
                ),
            )
        }
        WorthQueryDeclarationRelationalRoutingInput::Failed(envelope) => {
            WorthQueryDeclarationRelationalRoutingChecked::Denied(
                WorthQueryDeclarationRelationalRoutingDenied::new(
                    envelope.into_envelope(),
                    I::Family::relational_truth_contract().map(|contract| contract.truth_claim()),
                    I::Family::relational_truth_contract()
                        .map(|contract| contract.authority_family()),
                    WorthQueryDeclarationRelationalRoutingDenialCause::RelationalEnvelopeMismatch,
                ),
            )
        }
        WorthQueryDeclarationRelationalRoutingInput::Enveloped(_)
        | WorthQueryDeclarationRelationalRoutingInput::EnvelopeChecked(_) => {
            unreachable!("covered envelopes use the covered-handle path")
        }
    }
}
