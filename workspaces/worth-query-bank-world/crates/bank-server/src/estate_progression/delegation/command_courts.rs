//! Existing semantic courts consume real owning completion, never resubmit a retained intent.
use crate::{
    BankApplication, BankAuthenticatedPrincipal, BankClosedCapabilityCommand, BankCommitDenialKind,
    BankCommitDenialStage, BankEstateProgressionDenial, BankIdentityRuntime,
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

/// A semantic court retains the native close owner, including every outstanding obligation.
#[derive(Debug)]
pub(crate) enum CommandCourtOutcome {
    Committed(BankClosedCapabilityCommand),
    RecoveredCommitted(BankClosedCapabilityCommand),
    Denied {
        kind: BankCommitDenialKind,
        stage: BankCommitDenialStage,
    },
}

pub(crate) fn complete_delegation(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    action: EstateAction,
    key: &BankIdempotencyKey,
    request: &WorthQueryRequestScope,
) -> Result<CommandCourtOutcome, BankEstateProgressionDenial> {
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
) -> Result<CommandCourtOutcome, BankEstateProgressionDenial> {
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
) -> Result<CommandCourtOutcome, BankEstateProgressionDenial> {
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
                let receipt = BankClosedCapabilityCommand::from_query(closed);
                return Ok(if fresh {
                    CommandCourtOutcome::Committed(receipt)
                } else {
                    CommandCourtOutcome::RecoveredCommitted(receipt)
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
                        Ok(CommandCourtOutcome::Denied {
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

/// Recorded retry agrees with the independently held original performed command.
pub(crate) fn assert_original_receipt(
    original: &BankClosedCapabilityCommand,
    retry: &BankClosedCapabilityCommand,
) {
    let live = original
        .completion()
        .receipt()
        .expect("original command performed");
    let data = retry
        .recorded_receipt()
        .expect("retry retains durable receipt data");
    assert!(
        retry.live_receipt().is_none(),
        "recorded data is not Bank live authority"
    );
    assert_eq!(live.commit_reference(), data.commit_reference());
    assert_eq!(live.outcome_identity(), Some(data.outcome_identity()));
    assert_eq!(live.changed_record_count(), data.changed_record_count());
    assert_eq!(live.emitted_effect_count(), data.emitted_effect_count());
    assert_eq!(
        live.idempotency_binding().key_identity(),
        data.idempotency_key_identity()
    );
    let (before, after) = (live.basis_descriptor(), data.basis_descriptor());
    assert_eq!(before.descriptor_version(), after.descriptor_version());
    assert_eq!(before.branch_id(), after.branch_id());
    assert_eq!(before.posture(), after.posture());
    assert_eq!(
        before.reference().generation(),
        after.reference().generation()
    );
    match (
        before.reference().target().as_basis(),
        after.reference().target().as_basis(),
    ) {
        (Some(left), Some(right)) => {
            assert_eq!(left.selected_commit_id(), right.selected_commit_id());
            assert_eq!(left.version_id(), right.version_id());
            assert_eq!(left.parent_commit_ids(), right.parent_commit_ids());
            assert_eq!(left.roots(), right.roots());
        }
        (None, None) => {}
        (Some(_), None) | (None, Some(_)) => panic!("original target kind changed"),
    }
    assert_eq!(
        live.basis_descriptor().truth_version(),
        data.basis_descriptor().truth_version()
    );
    assert_eq!(
        live.basis_descriptor().schema_commitment(),
        data.basis_descriptor().schema_commitment()
    );
    assert_eq!(
        live.basis_descriptor().visibility_commitment(),
        data.basis_descriptor().visibility_commitment()
    );
    assert_eq!(
        live.recorded_output_settlement(),
        data.exact_output_settlement()
    );
}
