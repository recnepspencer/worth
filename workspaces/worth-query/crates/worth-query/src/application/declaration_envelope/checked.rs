use worth_foundational::facade::CanonicalDerivedDigest;

use crate::application::{
    WorthQueryDeclarationInput, WorthQueryDeclarationReceipt,
    WorthQueryDeclarationReceiptDenialCause, WorthQueryDeclarationRoutePlan,
    WorthQueryDeclarationRoutePlanDenialCause, WorthQueryDomainEntryMarker,
};

use super::{
    artifact::WorthQueryDeclarationEnvelope,
    class::{WorthQueryDeclarationEnvelopeClass, WorthQueryDeclarationEnvelopeEvidenceOrigin},
    denial::{
        WorthQueryDeclarationEnvelopeDeferred, WorthQueryDeclarationEnvelopeDenied,
        WorthQueryDeclarationEnvelopeFailed, WorthQueryDeclarationEnvelopeTerminalError,
    },
    digest::derive_envelope_digest,
    explain::WorthQueryDeclarationEnvelopeExplanation,
    input::WorthQueryDeclarationEnvelopeInput,
};

pub enum WorthQueryDeclarationEnvelopeChecked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    Enveloped(WorthQueryDeclarationEnvelope<D, I>),
    Deferred(WorthQueryDeclarationEnvelopeDeferred<D, I>),
    Denied(WorthQueryDeclarationEnvelopeDenied<D, I>),
    Failed(WorthQueryDeclarationEnvelopeFailed<D, I>),
}

pub(crate) fn worth_query_checked_declaration_envelope<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    input: WorthQueryDeclarationEnvelopeInput<D, I>,
) -> WorthQueryDeclarationEnvelopeChecked<D, I> {
    match input {
        WorthQueryDeclarationEnvelopeInput::ReceiptChecked(checked) => match checked {
            crate::application::WorthQueryDeclarationReceiptChecked::Issued(receipt) => {
                worth_query_checked_declaration_envelope(
                    WorthQueryDeclarationEnvelopeInput::issued(receipt),
                )
            }
            crate::application::WorthQueryDeclarationReceiptChecked::Deferred(receipt) => {
                worth_query_checked_declaration_envelope(
                    WorthQueryDeclarationEnvelopeInput::deferred(receipt),
                )
            }
            crate::application::WorthQueryDeclarationReceiptChecked::Denied(receipt) => {
                worth_query_checked_declaration_envelope(
                    WorthQueryDeclarationEnvelopeInput::denied(receipt),
                )
            }
            crate::application::WorthQueryDeclarationReceiptChecked::Failed(receipt) => {
                worth_query_checked_declaration_envelope(
                    WorthQueryDeclarationEnvelopeInput::failed(receipt),
                )
            }
        },
        WorthQueryDeclarationEnvelopeInput::IssuedReceipt(receipt) => {
            let envelope_digest = digest_for_receipt(
                &receipt,
                WorthQueryDeclarationEnvelopeClass::CoveredCrossing,
                None,
                None,
            );
            let explanation = build_explanation(&receipt, None, None);
            let envelope =
                WorthQueryDeclarationEnvelope::from_issued(receipt, envelope_digest, explanation);
            WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope)
        }
        WorthQueryDeclarationEnvelopeInput::DeferredReceipt(receipt) => {
            let reason = receipt.reason();
            let envelope_digest = digest_for_receipt(
                receipt.receipt(),
                WorthQueryDeclarationEnvelopeClass::DeferredCrossing,
                None,
                None,
            );
            let explanation = build_explanation(receipt.receipt(), None, None);
            let envelope =
                WorthQueryDeclarationEnvelope::from_deferred(receipt, envelope_digest, explanation);
            WorthQueryDeclarationEnvelopeChecked::Deferred(
                WorthQueryDeclarationEnvelopeDeferred::new(envelope, reason),
            )
        }
        WorthQueryDeclarationEnvelopeInput::DeniedReceipt(receipt) => {
            let route_cause = receipt.route_cause();
            let receipt_cause = receipt.receipt_cause();
            let reason = receipt.reason();
            let envelope_digest = digest_for_receipt(
                receipt.receipt(),
                WorthQueryDeclarationEnvelopeClass::DeniedCrossing,
                route_cause,
                receipt_cause,
            );
            let explanation = build_explanation(receipt.receipt(), route_cause, receipt_cause);
            let envelope =
                WorthQueryDeclarationEnvelope::from_denied(receipt, envelope_digest, explanation);
            WorthQueryDeclarationEnvelopeChecked::Denied(WorthQueryDeclarationEnvelopeDenied::new(
                envelope,
                route_cause,
                receipt_cause,
                reason,
            ))
        }
        WorthQueryDeclarationEnvelopeInput::FailedReceipt(receipt) => {
            let reason = receipt.reason();
            let envelope_digest = digest_for_receipt(
                receipt.receipt(),
                WorthQueryDeclarationEnvelopeClass::FailedCrossing,
                None,
                None,
            );
            let explanation = build_explanation(receipt.receipt(), None, None);
            let envelope =
                WorthQueryDeclarationEnvelope::from_failed(receipt, envelope_digest, explanation);
            WorthQueryDeclarationEnvelopeChecked::Failed(WorthQueryDeclarationEnvelopeFailed::new(
                envelope, reason,
            ))
        }
    }
}

pub(crate) fn worth_query_declaration_envelope_terminal_from_receipt_terminal<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    error: crate::application::WorthQueryDeclarationReceiptTerminalError<D, I>,
) -> WorthQueryDeclarationEnvelopeTerminalError<D, I> {
    match error {
        crate::application::WorthQueryDeclarationReceiptTerminalError::EvidenceIdentityDerivationDenied(cause) => {
            WorthQueryDeclarationEnvelopeTerminalError::EvidenceIdentityDerivationDenied(cause)
        }
        crate::application::WorthQueryDeclarationReceiptTerminalError::Deferred(receipt) => {
            match worth_query_checked_declaration_envelope(
                WorthQueryDeclarationEnvelopeInput::deferred(receipt),
            ) {
                WorthQueryDeclarationEnvelopeChecked::Deferred(envelope) => {
                    WorthQueryDeclarationEnvelopeTerminalError::Deferred(envelope)
                }
                _ => panic!("deferred receipt should lower into deferred envelope"),
            }
        }
        crate::application::WorthQueryDeclarationReceiptTerminalError::Denied(receipt) => {
            match worth_query_checked_declaration_envelope(
                WorthQueryDeclarationEnvelopeInput::denied(receipt),
            ) {
                WorthQueryDeclarationEnvelopeChecked::Denied(envelope) => {
                    WorthQueryDeclarationEnvelopeTerminalError::Denied(envelope)
                }
                _ => panic!("denied receipt should lower into denied envelope"),
            }
        }
        crate::application::WorthQueryDeclarationReceiptTerminalError::Failed(receipt) => {
            match worth_query_checked_declaration_envelope(
                WorthQueryDeclarationEnvelopeInput::failed(receipt),
            ) {
                WorthQueryDeclarationEnvelopeChecked::Failed(envelope) => {
                    WorthQueryDeclarationEnvelopeTerminalError::Failed(envelope)
                }
                _ => panic!("failed receipt should lower into failed envelope"),
            }
        }
    }
}

fn digest_for_receipt<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    receipt: &WorthQueryDeclarationReceipt<D, I>,
    class: WorthQueryDeclarationEnvelopeClass,
    route_cause: Option<WorthQueryDeclarationRoutePlanDenialCause>,
    receipt_cause: Option<WorthQueryDeclarationReceiptDenialCause>,
) -> CanonicalDerivedDigest {
    let evidence = receipt.foundational_evidence();
    let version = evidence
        .subject()
        .canonical_declaration()
        .version()
        .foundational()
        .clone();
    derive_envelope_digest(
        version,
        receipt.handle_identity_digest(),
        receipt.operating_context_identity_digest(),
        receipt.declaration_family_key(),
        receipt.declaration_digest(),
        receipt.progression_digest(),
        receipt.route_plan_digest(),
        &canonical_digest_token(receipt.receipt_digest()),
        class,
        WorthQueryDeclarationEnvelopeEvidenceOrigin::from_foundational_class(evidence.class()),
        route_cause,
        receipt_cause,
        receipt.aspect_contract(),
        receipt.aspect_publication(),
    )
}

fn build_explanation<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    receipt: &WorthQueryDeclarationReceipt<D, I>,
    route_cause: Option<WorthQueryDeclarationRoutePlanDenialCause>,
    receipt_cause: Option<WorthQueryDeclarationReceiptDenialCause>,
) -> WorthQueryDeclarationEnvelopeExplanation {
    let evidence = receipt.foundational_evidence();
    let evidence_origin =
        WorthQueryDeclarationEnvelopeEvidenceOrigin::from_foundational_class(evidence.class());
    let mut retained_truths = receipt.explain().retained_truths().to_vec();
    retained_truths.push(format!(
        "receipt:{}",
        canonical_digest_token(receipt.receipt_digest())
    ));
    retained_truths.push(format!("evidence-origin:{}", evidence_origin.as_str()));
    retained_truths.push(format!(
        "receipt-aspect-publication:{:?}",
        receipt.aspect_publication()
    ));
    if let Some(route_plan) = receipt.route_plan() {
        retained_truths.push(format!("route-plan:{}", route_plan.route_plan_digest()));
    }

    WorthQueryDeclarationEnvelopeExplanation::new(
        receipt.explain().crossing_posture(),
        evidence_origin,
        receipt.explain().route_reference().map(ToOwned::to_owned),
        retained_truths,
        receipt.route_plan().map(route_reason_from_plan),
        route_cause,
        receipt.explain().governing_reason().to_string(),
        receipt_cause,
    )
}

fn canonical_digest_token(digest: &CanonicalDerivedDigest) -> String {
    let hex = digest
        .value()
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("{}:{hex}", digest.metadata().algorithm().id().as_str())
}

fn route_reason_from_plan<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    route_plan: &WorthQueryDeclarationRoutePlan<D, I>,
) -> String {
    let mut parts = vec![route_plan.explain().route_contract_reason().to_string()];
    parts.extend(route_plan.explain().route_segment_reasons().iter().cloned());
    if let Some(intent_reason) = route_plan.explain().intent_reason() {
        parts.push(intent_reason.to_string());
    }
    parts.join("; ")
}
