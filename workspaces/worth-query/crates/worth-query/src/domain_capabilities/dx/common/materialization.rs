use crate::domain_capabilities::denials::WorthQueryDomainCapabilityTransitionOutcome;
use crate::domain_capabilities::dx::checked::WorthQueryCheckedDomainCapabilityOutcome;
use crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

pub(super) fn materialize_common_lane<Request, Eligible, Admitted, Ready, Success>(
    category: &'static str,
    target_kind: WorthQueryDomainCapabilityTargetKind,
    semantic_posture: &'static str,
    requested: Request,
    evaluate: impl FnOnce(
        Request,
    ) -> Result<
        WorthQueryDomainCapabilityTransitionOutcome<Eligible>,
        IdentityDerivationDenial,
    >,
    admit: impl FnOnce(
        Eligible,
    ) -> Result<
        WorthQueryDomainCapabilityTransitionOutcome<Admitted>,
        IdentityDerivationDenial,
    >,
    prepare: impl FnOnce(
        Admitted,
    ) -> Result<
        WorthQueryDomainCapabilityTransitionOutcome<Ready>,
        IdentityDerivationDenial,
    >,
    materialize: impl FnOnce(
        Ready,
    ) -> Result<
        WorthQueryDomainCapabilityTransitionOutcome<Success>,
        IdentityDerivationDenial,
    >,
) -> Result<WorthQueryCheckedDomainCapabilityOutcome<Success>, IdentityDerivationDenial> {
    let outcome = match evaluate(requested)? {
        worth_proof::TransitionOutcome::Success(eligible) => match admit(eligible)? {
            worth_proof::TransitionOutcome::Success(admitted) => match prepare(admitted)? {
                worth_proof::TransitionOutcome::Success(ready) => materialize(ready)?,
                worth_proof::TransitionOutcome::Denied(denial) => {
                    worth_proof::TransitionOutcome::Denied(denial)
                }
                worth_proof::TransitionOutcome::Stale(stale) => {
                    worth_proof::TransitionOutcome::Stale(stale)
                }
                worth_proof::TransitionOutcome::RebindRequired(rebind) => {
                    worth_proof::TransitionOutcome::RebindRequired(rebind)
                }
                worth_proof::TransitionOutcome::Failed(failure) => {
                    worth_proof::TransitionOutcome::Failed(failure)
                }
                worth_proof::TransitionOutcome::Deferred(never) => match never {},
            },
            worth_proof::TransitionOutcome::Denied(denial) => {
                worth_proof::TransitionOutcome::Denied(denial)
            }
            worth_proof::TransitionOutcome::Stale(stale) => {
                worth_proof::TransitionOutcome::Stale(stale)
            }
            worth_proof::TransitionOutcome::RebindRequired(rebind) => {
                worth_proof::TransitionOutcome::RebindRequired(rebind)
            }
            worth_proof::TransitionOutcome::Failed(failure) => {
                worth_proof::TransitionOutcome::Failed(failure)
            }
            worth_proof::TransitionOutcome::Deferred(never) => match never {},
        },
        worth_proof::TransitionOutcome::Denied(denial) => {
            worth_proof::TransitionOutcome::Denied(denial)
        }
        worth_proof::TransitionOutcome::Stale(stale) => {
            worth_proof::TransitionOutcome::Stale(stale)
        }
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            worth_proof::TransitionOutcome::RebindRequired(rebind)
        }
        worth_proof::TransitionOutcome::Failed(failure) => {
            worth_proof::TransitionOutcome::Failed(failure)
        }
        worth_proof::TransitionOutcome::Deferred(never) => match never {},
    };

    Ok(
        WorthQueryCheckedDomainCapabilityOutcome::from_transition_outcome(
            category,
            target_kind,
            semantic_posture,
            outcome,
        ),
    )
}
