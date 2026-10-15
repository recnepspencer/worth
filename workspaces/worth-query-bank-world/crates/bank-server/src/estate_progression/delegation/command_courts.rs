//! Existing semantic courts consume real owning completion, never resubmit a retained intent.
use crate::{
    BankApplication, BankAuthenticatedPrincipal, BankCommitDenialKind, BankCommitDenialStage,
    BankCommitReceipt, BankEstateProgressionDenial, BankIdentityRuntime, BankMutationCommitOutcome,
};
use bank_domain::{estate::EstateAction, proposals::BankIdempotencyKey, schema::BankSchema};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryApplicationCapabilityDelegationDenial,
        WorthQueryApplicationCapabilityRevocationDenial, WorthQueryCapabilityCommand,
        WorthQueryCapabilityCommandAdvance as Advance,
        WorthQueryCapabilityCommandCompletion as Completion,
        WorthQueryCapabilityCommandDenial as Denial,
    },
};

pub(crate) fn complete_delegation(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    action: EstateAction,
    key: &BankIdempotencyKey,
    request: &WorthQueryRequestScope,
) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
    let command = runtime
        .retain_estate_capability_delegation(principal, action, key.clone(), request)
        .map_err(|(_input, _key, _prepare, denial)| map_delegation_denial(denial))?;
    complete(runtime, principal, request, command, map_delegation_denial)
}

pub(crate) fn complete_revocation(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    action: EstateAction,
    key: &BankIdempotencyKey,
    request: &WorthQueryRequestScope,
) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
    let command = runtime
        .retain_estate_capability_revocation(principal, action, key.clone(), request)
        .map_err(|(_input, _key, _prepare, denial)| map_revocation_denial(denial))?;
    complete(runtime, principal, request, command, map_revocation_denial)
}

fn complete<'runtime, D>(
    runtime: &'runtime BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: &WorthQueryRequestScope,
    mut command: WorthQueryCapabilityCommand<'runtime, BankSchema, BankApplication, D>,
    map: impl Fn(D) -> BankEstateProgressionDenial,
) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
    loop {
        match command.advance(&runtime.request(principal, request)) {
            Advance::Progress(next) => command = next,
            Advance::Completed(done) => {
                let fresh = matches!(done.evidence(), Completion::Fresh { .. });
                let closed = done
                    .close_with_evidence(&runtime.request(principal, request))
                    .unwrap_or_else(|(_owner, _denial)| {
                        panic!("native command close must release actual custody")
                    });
                let receipt = BankCommitReceipt::from_closed_capability_command(closed);
                return Ok(if fresh {
                    BankMutationCommitOutcome::Committed(receipt)
                } else {
                    BankMutationCommitOutcome::AlreadyCommitted(receipt)
                });
            }
            Advance::Refused {
                command: retained,
                denial,
            } => {
                assert!(
                    retained.close(&runtime.request(principal, request)).is_ok(),
                    "these no-effect semantic refusals must lawfully close their original owner"
                );
                return match denial {
                Denial::Native(denial) => {
                    let denial = map(denial);
                    if matches!(denial, BankEstateProgressionDenial::IdempotencyIntentDrift) {
                        Ok(BankMutationCommitOutcome::Denied {
                            kind: BankCommitDenialKind::IdempotencyIntentDrift,
                            stage: BankCommitDenialStage::Idempotency,
                        })
                    } else { Err(denial) }
                }
                Denial::Execution(cause) => Err(BankEstateProgressionDenial::ApplicationEntry(
                    worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause))),
                _ => panic!("semantic court unexpectedly refused command affinity or cleanup"),
                };
            }
            Advance::Terminal(_terminal) => {
                panic!("semantic court unexpectedly reached native terminality")
            }
        }
    }
}

fn map_delegation_denial(
    denial: WorthQueryApplicationCapabilityDelegationDenial<BankEstateProgressionDenial>,
) -> BankEstateProgressionDenial {
    use WorthQueryApplicationCapabilityDelegationDenial as Query;
    match denial {
        Query::ExecutionRequest(cause) => BankEstateProgressionDenial::ApplicationEntry(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
        ),
        Query::Program(denial) => BankEstateProgressionDenial::ProgramAction(denial),
        Query::ProgramMismatch => BankEstateProgressionDenial::ProgramMismatch,
        Query::PrincipalBindingInstallation(denial) => {
            BankEstateProgressionDenial::PrincipalBindingInstallation(denial)
        }
        Query::CapabilityInstallation(denial) => {
            BankEstateProgressionDenial::from_capability_installation(denial)
        }
        Query::OperationInstallation(denial) => {
            BankEstateProgressionDenial::from_operation_installation(denial)
        }
        Query::ProductSelection(denial) => {
            BankEstateProgressionDenial::from_product_selection(denial)
        }
        Query::PrincipalResolution(denial) => {
            BankEstateProgressionDenial::PrincipalResolution(denial)
        }
        Query::IdentityEncoding(denial) => BankEstateProgressionDenial::IdentityEncoding(denial),
        Query::Authorization(denial) => BankEstateProgressionDenial::from_authorization(denial),
        Query::Idempotency(denial) => BankEstateProgressionDenial::from_idempotency(denial),
        Query::IdempotencyIntentDrift => BankEstateProgressionDenial::IdempotencyIntentDrift,
        Query::Preparation(denial) => denial,
        other => panic!("unexpected new native command denial: {other:?}"),
    }
}

fn map_revocation_denial(
    denial: WorthQueryApplicationCapabilityRevocationDenial<BankEstateProgressionDenial>,
) -> BankEstateProgressionDenial {
    use WorthQueryApplicationCapabilityRevocationDenial as Query;
    match denial {
        Query::ExecutionRequest(cause) => BankEstateProgressionDenial::ApplicationEntry(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
        ),
        Query::Program(denial) => BankEstateProgressionDenial::ProgramAction(denial),
        Query::ProgramMismatch => BankEstateProgressionDenial::ProgramMismatch,
        Query::PrincipalBindingInstallation(denial) => {
            BankEstateProgressionDenial::PrincipalBindingInstallation(denial)
        }
        Query::CapabilityInstallation(denial) => {
            BankEstateProgressionDenial::from_capability_installation(denial)
        }
        Query::OperationInstallation(denial) => {
            BankEstateProgressionDenial::from_operation_installation(denial)
        }
        Query::ProductSelection(denial) => {
            BankEstateProgressionDenial::from_product_selection(denial)
        }
        Query::PrincipalResolution(denial) => {
            BankEstateProgressionDenial::PrincipalResolution(denial)
        }
        Query::IdentityEncoding(denial) => BankEstateProgressionDenial::IdentityEncoding(denial),
        Query::Authorization(denial) => BankEstateProgressionDenial::from_authorization(denial),
        Query::Idempotency(denial) => BankEstateProgressionDenial::from_idempotency(denial),
        Query::IdempotencyIntentDrift => BankEstateProgressionDenial::IdempotencyIntentDrift,
        Query::Preparation(denial) => denial,
        other => panic!("unexpected new native command denial: {other:?}"),
    }
}
