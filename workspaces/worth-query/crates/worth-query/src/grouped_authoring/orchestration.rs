use crate::application::{
    WorthQueryDeclarationEnvelope, WorthQueryDeclarationEnvelopeOrchestrationTranscript,
    WorthQueryDeclarationInput, WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::hash_parts;

use super::artifact::{
    WorthQueryGroupedDeclarationArtifact, WorthQueryGroupedDeclarationAspectRecord,
};
use super::member_lowering::lower_grouped_members_on_handle;
use super::posture::WorthQueryGroupedMemberRole;

mod ordinary;

pub(crate) use ordinary::ordinary_outcome_from_grouped_orchestration_checked;

pub struct WorthQueryGroupedEnvelopeMember<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    member_index: usize,
    role: WorthQueryGroupedMemberRole,
    aspect_record: WorthQueryGroupedDeclarationAspectRecord,
    envelope: WorthQueryDeclarationEnvelope<D, I>,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryGroupedEnvelopeMember<D, I>
{
    pub(crate) fn new(
        member_index: usize,
        role: WorthQueryGroupedMemberRole,
        aspect_record: WorthQueryGroupedDeclarationAspectRecord,
        envelope: WorthQueryDeclarationEnvelope<D, I>,
    ) -> Self {
        Self {
            member_index,
            role,
            aspect_record,
            envelope,
        }
    }

    pub fn member_index(&self) -> usize {
        self.member_index
    }

    pub fn role(&self) -> WorthQueryGroupedMemberRole {
        self.role
    }

    pub fn aspect_record(&self) -> &WorthQueryGroupedDeclarationAspectRecord {
        &self.aspect_record
    }

    pub fn envelope(&self) -> &WorthQueryDeclarationEnvelope<D, I> {
        &self.envelope
    }
}

pub struct WorthQueryGroupedOrchestration<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    member_envelopes: Vec<WorthQueryGroupedEnvelopeMember<D, I>>,
    orchestration_digest: String,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryGroupedOrchestration<D, I>
{
    pub(crate) fn new(
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
        member_envelopes: Vec<WorthQueryGroupedEnvelopeMember<D, I>>,
    ) -> Self {
        let orchestration_digest = hash_parts(&[
            format!("group:{}", declaration.group_digest()),
            format!(
                "member_envelopes:{}",
                member_envelopes
                    .iter()
                    .map(|member| format!(
                        "{}:{:?}:{:?}:{:?}",
                        member.member_index(),
                        member.role(),
                        member.aspect_record().coverage_basis(),
                        member.envelope().envelope_digest()
                    ))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]);
        Self {
            declaration,
            member_envelopes,
            orchestration_digest,
        }
    }

    pub fn declaration(&self) -> &WorthQueryGroupedDeclarationArtifact<D, I> {
        &self.declaration
    }

    pub fn member_envelopes(&self) -> &[WorthQueryGroupedEnvelopeMember<D, I>] {
        &self.member_envelopes
    }

    pub fn orchestration_digest(&self) -> &str {
        &self.orchestration_digest
    }
}

pub struct WorthQueryGroupedMemberOrchestrationStop<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    member_index: usize,
    member_role: WorthQueryGroupedMemberRole,
    member_aspect_record: WorthQueryGroupedDeclarationAspectRecord,
    member_outcome: crate::application::WorthQueryDeclarationEntryOrchestrationOutcome<D, I>,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryGroupedMemberOrchestrationStop<D, I>
{
    pub(crate) fn new(
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
        member_index: usize,
        member_role: WorthQueryGroupedMemberRole,
        member_aspect_record: WorthQueryGroupedDeclarationAspectRecord,
        member_outcome: crate::application::WorthQueryDeclarationEntryOrchestrationOutcome<D, I>,
    ) -> Self {
        Self {
            declaration,
            member_index,
            member_role,
            member_aspect_record,
            member_outcome,
        }
    }

    pub fn declaration(&self) -> &WorthQueryGroupedDeclarationArtifact<D, I> {
        &self.declaration
    }

    pub fn member_index(&self) -> usize {
        self.member_index
    }

    pub fn member_role(&self) -> WorthQueryGroupedMemberRole {
        self.member_role
    }

    pub fn member_aspect_record(&self) -> &WorthQueryGroupedDeclarationAspectRecord {
        &self.member_aspect_record
    }

    pub fn member_outcome(
        &self,
    ) -> &crate::application::WorthQueryDeclarationEntryOrchestrationOutcome<D, I> {
        &self.member_outcome
    }
}

pub struct WorthQueryGroupedOrchestrationAlignmentStop<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    reason: String,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryGroupedOrchestrationAlignmentStop<D, I>
{
    pub(crate) fn new(
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            declaration,
            reason: reason.into(),
        }
    }

    pub fn declaration(&self) -> &WorthQueryGroupedDeclarationArtifact<D, I> {
        &self.declaration
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

pub enum WorthQueryGroupedOrchestrationStop<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    MemberStopped(WorthQueryGroupedMemberOrchestrationStop<D, I>),
    WrongWorld(WorthQueryGroupedOrchestrationAlignmentStop<D, I>),
    WrongHandle(WorthQueryGroupedOrchestrationAlignmentStop<D, I>),
}

pub enum WorthQueryGroupedOrchestrationChecked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    Bound(WorthQueryGroupedOrchestration<D, I>),
    MemberStopped(WorthQueryGroupedMemberOrchestrationStop<D, I>),
    WrongWorld(WorthQueryGroupedOrchestrationAlignmentStop<D, I>),
    WrongHandle(WorthQueryGroupedOrchestrationAlignmentStop<D, I>),
}

pub struct WorthQueryGroupedOrchestrationProof<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    member_transcripts: Vec<WorthQueryDeclarationEnvelopeOrchestrationTranscript<D, I>>,
    outcome: WorthQueryGroupedOrchestrationChecked<D, I>,
    orchestration_digest: String,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryGroupedOrchestrationProof<D, I>
{
    pub(crate) fn new(
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
        member_transcripts: Vec<WorthQueryDeclarationEnvelopeOrchestrationTranscript<D, I>>,
        outcome: WorthQueryGroupedOrchestrationChecked<D, I>,
    ) -> Self {
        let orchestration_digest = hash_parts(&[
            format!("group:{}", declaration.group_digest()),
            format!(
                "member_proofs:{}",
                member_transcripts
                    .iter()
                    .map(|member| member.orchestration_digest().to_string())
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]);
        Self {
            declaration,
            member_transcripts,
            outcome,
            orchestration_digest,
        }
    }

    pub fn declaration(&self) -> &WorthQueryGroupedDeclarationArtifact<D, I> {
        &self.declaration
    }

    pub fn member_transcripts(
        &self,
    ) -> &[WorthQueryDeclarationEnvelopeOrchestrationTranscript<D, I>] {
        &self.member_transcripts
    }

    pub fn outcome(&self) -> &WorthQueryGroupedOrchestrationChecked<D, I> {
        &self.outcome
    }

    pub fn orchestration_digest(&self) -> &str {
        &self.orchestration_digest
    }

    pub fn into_checked(self) -> WorthQueryGroupedOrchestrationChecked<D, I> {
        self.outcome
    }
}

pub type WorthQueryGroupedOrchestrationTranscript<D, I> = WorthQueryGroupedOrchestrationProof<D, I>;

pub(crate) fn worth_query_grouped_orchestration_checked_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
) -> Result<WorthQueryGroupedOrchestrationChecked<D, I>, IdentityDerivationDenial> {
    Ok(lower_grouped_members_on_handle(handle, declaration, false)?.checked())
}

pub(crate) fn worth_query_grouped_orchestration_proof_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
) -> Result<WorthQueryGroupedOrchestrationTranscript<D, I>, IdentityDerivationDenial> {
    let (checked, member_transcripts) =
        lower_grouped_members_on_handle(handle, declaration.clone(), true)?.into_parts();
    Ok(WorthQueryGroupedOrchestrationProof::new(
        declaration,
        member_transcripts,
        checked,
    ))
}
