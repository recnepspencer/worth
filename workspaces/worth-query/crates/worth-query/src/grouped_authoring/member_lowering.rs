use crate::application::{
    WorthQueryDeclarationEntryOrchestrationDeferred, WorthQueryDeclarationEntryOrchestrationDenied,
    WorthQueryDeclarationEntryOrchestrationFailed, WorthQueryDeclarationEntryOrchestrationOutcome,
    WorthQueryDeclarationEntryOrchestrationStage, WorthQueryDeclarationEnvelopeChecked,
    WorthQueryDeclarationEnvelopeOrchestrationTranscript, WorthQueryDeclarationInput,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use worth_foundational::facade::CanonicalDerivedDigest;

use super::artifact::WorthQueryGroupedDeclarationArtifact;
use super::orchestration::{
    WorthQueryGroupedEnvelopeMember, WorthQueryGroupedMemberOrchestrationStop,
    WorthQueryGroupedOrchestration, WorthQueryGroupedOrchestrationAlignmentStop,
    WorthQueryGroupedOrchestrationChecked,
};

pub(crate) struct WorthQueryGroupedMemberLowering<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    checked: WorthQueryGroupedOrchestrationChecked<D, I>,
    member_transcripts: Vec<WorthQueryDeclarationEnvelopeOrchestrationTranscript<D, I>>,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryGroupedMemberLowering<D, I>
{
    pub(crate) fn checked(self) -> WorthQueryGroupedOrchestrationChecked<D, I> {
        self.checked
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        WorthQueryGroupedOrchestrationChecked<D, I>,
        Vec<WorthQueryDeclarationEnvelopeOrchestrationTranscript<D, I>>,
    ) {
        (self.checked, self.member_transcripts)
    }
}

pub(crate) fn lower_grouped_members_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    capture_proof: bool,
) -> WorthQueryGroupedMemberLowering<D, I> {
    if declaration.operating_context_identity_digest() != handle.operating_context_identity_digest()
    {
        return WorthQueryGroupedMemberLowering {
            checked: WorthQueryGroupedOrchestrationChecked::WrongWorld(
                WorthQueryGroupedOrchestrationAlignmentStop::new(
                    declaration,
                    "the grouped declaration was admitted in a different operating context",
                ),
            ),
            member_transcripts: Vec::new(),
        };
    }
    if declaration.handle_identity_digest() != handle.handle_identity_digest() {
        return WorthQueryGroupedMemberLowering {
            checked: WorthQueryGroupedOrchestrationChecked::WrongHandle(
                WorthQueryGroupedOrchestrationAlignmentStop::new(
                    declaration,
                    "the grouped declaration was admitted on a different configured domain handle",
                ),
            ),
            member_transcripts: Vec::new(),
        };
    }

    let mut member_envelopes = Vec::with_capacity(declaration.members().len());
    let mut member_transcripts = Vec::with_capacity(declaration.members().len());
    for member in declaration.members() {
        if capture_proof {
            let transcript =
                handle.orchestrate_envelope_from_progressed_proof(member.progression().clone());
            let terminal_outcome = match transcript.outcome() {
                WorthQueryDeclarationEnvelopeChecked::Enveloped(_) => None,
                terminal => Some(declaration_entry_outcome_from_envelope_checked_ref(
                    terminal,
                )),
            };
            member_transcripts.push(transcript);
            if let Some(outcome) = terminal_outcome {
                return WorthQueryGroupedMemberLowering {
                    checked: WorthQueryGroupedOrchestrationChecked::MemberStopped(
                        WorthQueryGroupedMemberOrchestrationStop::new(
                            declaration.clone(),
                            member.member_index(),
                            member.role(),
                            member.aspect_record().clone(),
                            outcome,
                        ),
                    ),
                    member_transcripts,
                };
            }

            let checked =
                handle.orchestrate_envelope_from_progressed_checked(member.progression().clone());
            if let WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope) = checked {
                member_envelopes.push(WorthQueryGroupedEnvelopeMember::new(
                    member.member_index(),
                    member.role(),
                    member.aspect_record().clone(),
                    envelope,
                ));
            }
            continue;
        }

        let checked =
            handle.orchestrate_envelope_from_progressed_checked(member.progression().clone());
        match checked {
            WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope) => {
                member_envelopes.push(WorthQueryGroupedEnvelopeMember::new(
                    member.member_index(),
                    member.role(),
                    member.aspect_record().clone(),
                    envelope,
                ));
            }
            terminal => {
                return WorthQueryGroupedMemberLowering {
                    checked: WorthQueryGroupedOrchestrationChecked::MemberStopped(
                        WorthQueryGroupedMemberOrchestrationStop::new(
                            declaration.clone(),
                            member.member_index(),
                            member.role(),
                            member.aspect_record().clone(),
                            declaration_entry_outcome_from_envelope_checked(terminal),
                        ),
                    ),
                    member_transcripts,
                };
            }
        }
    }

    WorthQueryGroupedMemberLowering {
        checked: WorthQueryGroupedOrchestrationChecked::Bound(WorthQueryGroupedOrchestration::new(
            declaration,
            member_envelopes,
        )),
        member_transcripts,
    }
}

fn declaration_entry_outcome_from_envelope_checked_ref<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    checked: &WorthQueryDeclarationEnvelopeChecked<D, I>,
) -> WorthQueryDeclarationEntryOrchestrationOutcome<D, I> {
    match checked {
        WorthQueryDeclarationEnvelopeChecked::Enveloped(_) => {
            panic!("terminal conversion should not receive an enveloped outcome")
        }
        WorthQueryDeclarationEnvelopeChecked::Deferred(value) => {
            WorthQueryDeclarationEntryOrchestrationOutcome::Deferred(
                WorthQueryDeclarationEntryOrchestrationDeferred::new(
                    value.envelope().declaration_family_key(),
                    WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
                    value.reason(),
                    Some(canonical_digest_token(value.envelope().envelope_digest())),
                ),
            )
        }
        WorthQueryDeclarationEnvelopeChecked::Denied(value) => {
            WorthQueryDeclarationEntryOrchestrationOutcome::Denied(
                WorthQueryDeclarationEntryOrchestrationDenied::new(
                    value.envelope().declaration_family_key(),
                    WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
                    value.reason(),
                    Some(canonical_digest_token(value.envelope().envelope_digest())),
                ),
            )
        }
        WorthQueryDeclarationEnvelopeChecked::Failed(value) => {
            WorthQueryDeclarationEntryOrchestrationOutcome::Failed(
                WorthQueryDeclarationEntryOrchestrationFailed::new(
                    value.envelope().declaration_family_key(),
                    WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
                    value.reason(),
                    Some(canonical_digest_token(value.envelope().envelope_digest())),
                ),
            )
        }
    }
}

fn declaration_entry_outcome_from_envelope_checked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    checked: WorthQueryDeclarationEnvelopeChecked<D, I>,
) -> WorthQueryDeclarationEntryOrchestrationOutcome<D, I> {
    match checked {
        WorthQueryDeclarationEnvelopeChecked::Enveloped(envelope) => {
            WorthQueryDeclarationEntryOrchestrationOutcome::Enveloped(envelope)
        }
        WorthQueryDeclarationEnvelopeChecked::Deferred(value) => {
            WorthQueryDeclarationEntryOrchestrationOutcome::Deferred(
                WorthQueryDeclarationEntryOrchestrationDeferred::new(
                    value.envelope().declaration_family_key(),
                    WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
                    value.reason(),
                    Some(canonical_digest_token(value.envelope().envelope_digest())),
                ),
            )
        }
        WorthQueryDeclarationEnvelopeChecked::Denied(value) => {
            WorthQueryDeclarationEntryOrchestrationOutcome::Denied(
                WorthQueryDeclarationEntryOrchestrationDenied::new(
                    value.envelope().declaration_family_key(),
                    WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
                    value.reason(),
                    Some(canonical_digest_token(value.envelope().envelope_digest())),
                ),
            )
        }
        WorthQueryDeclarationEnvelopeChecked::Failed(value) => {
            WorthQueryDeclarationEntryOrchestrationOutcome::Failed(
                WorthQueryDeclarationEntryOrchestrationFailed::new(
                    value.envelope().declaration_family_key(),
                    WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
                    value.reason(),
                    Some(canonical_digest_token(value.envelope().envelope_digest())),
                ),
            )
        }
    }
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
