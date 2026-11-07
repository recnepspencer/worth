use crate::application::{WorthQueryDeclarationInput, WorthQueryDomainEntryMarker};
use crate::contribution_composed_orchestration::{
    orchestrate_progressed_declaration_with_contributions_checked_on_handle,
    WorthQueryContributionComposedMaterializationPolicy,
    WorthQueryContributionComposedOrchestration,
    WorthQueryContributionComposedOrchestrationChecked, WorthQueryContributionIntent,
};
use worth_foundational::FoundationalProfileSet;

use super::artifact::{
    WorthQueryGroupedDeclarationArtifact, WorthQueryGroupedDeclarationAspectRecord,
};
use super::contribution_input::{
    WorthQueryGroupedContributionAssignment, WorthQueryGroupedContributionInput,
    WorthQueryGroupedContributionSource,
};
use super::declaration::{
    worth_query_grouped_declaration_checked_on_handle, WorthQueryGroupedDeclarationChecked,
};
use super::declaration_stop::WorthQueryGroupedDeclarationStop;
use super::orchestration::WorthQueryGroupedOrchestrationAlignmentStop;
use super::posture::WorthQueryGroupedMemberRole;

#[derive(Clone)]
pub struct WorthQueryGroupedContributionMemberContext {
    member_index: usize,
    role: WorthQueryGroupedMemberRole,
    aspect_record: WorthQueryGroupedDeclarationAspectRecord,
    shared_contribution_count: usize,
    member_contribution_count: usize,
}

impl WorthQueryGroupedContributionMemberContext {
    fn new(
        member_index: usize,
        role: WorthQueryGroupedMemberRole,
        aspect_record: WorthQueryGroupedDeclarationAspectRecord,
        shared_contribution_count: usize,
        member_contribution_count: usize,
    ) -> Self {
        Self {
            member_index,
            role,
            aspect_record,
            shared_contribution_count,
            member_contribution_count,
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

    pub fn shared_contribution_count(&self) -> usize {
        self.shared_contribution_count
    }

    pub fn member_contribution_count(&self) -> usize {
        self.member_contribution_count
    }
}

pub struct WorthQueryGroupedContributionComposition<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    members: Vec<(
        WorthQueryGroupedContributionMemberContext,
        WorthQueryContributionComposedOrchestration<D, I>,
    )>,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryGroupedContributionComposition<D, I>
{
    fn new(
        declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
        members: Vec<(
            WorthQueryGroupedContributionMemberContext,
            WorthQueryContributionComposedOrchestration<D, I>,
        )>,
    ) -> Self {
        Self {
            declaration,
            members,
        }
    }

    pub fn declaration(&self) -> &WorthQueryGroupedDeclarationArtifact<D, I> {
        &self.declaration
    }

    pub fn members(
        &self,
    ) -> &[(
        WorthQueryGroupedContributionMemberContext,
        WorthQueryContributionComposedOrchestration<D, I>,
    )] {
        &self.members
    }
}

pub enum WorthQueryGroupedContributionStop<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    DeclarationStopped(WorthQueryGroupedDeclarationStop),
    EvidenceIdentityDerivationDenied(
        crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    ),
    WrongWorld(WorthQueryGroupedOrchestrationAlignmentStop<D, I>),
    WrongHandle(WorthQueryGroupedOrchestrationAlignmentStop<D, I>),
    MemberStopped(
        WorthQueryGroupedContributionMemberContext,
        WorthQueryContributionComposedOrchestrationChecked<D, I>,
    ),
}

pub(crate) fn worth_query_grouped_contribution_checked_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: crate::application::WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<D, C>,
    input: WorthQueryGroupedContributionInput<D, I>,
) -> Result<WorthQueryGroupedContributionComposition<D, I>, WorthQueryGroupedContributionStop<D, I>>
{
    let (source, shared_contributions, member_contributions, materialization_profile) =
        input.into_parts();
    let declaration = match source {
        WorthQueryGroupedContributionSource::DeclarationInput(declaration_input) => {
            match worth_query_grouped_declaration_checked_on_handle(handle, declaration_input) {
                WorthQueryGroupedDeclarationChecked::Bound(value) => value,
                WorthQueryGroupedDeclarationChecked::MemberStopped(stop) => {
                    return Err(WorthQueryGroupedContributionStop::DeclarationStopped(stop));
                }
            }
        }
        WorthQueryGroupedContributionSource::DeclarationArtifact(declaration) => declaration,
    };
    if declaration.operating_context_identity_digest() != handle.operating_context_identity_digest()
    {
        return Err(WorthQueryGroupedContributionStop::WrongWorld(
            WorthQueryGroupedOrchestrationAlignmentStop::new(
                declaration,
                "the grouped declaration was admitted in a different operating context",
            ),
        ));
    }
    if declaration.handle_identity_digest() != handle.handle_identity_digest() {
        return Err(WorthQueryGroupedContributionStop::WrongHandle(
            WorthQueryGroupedOrchestrationAlignmentStop::new(
                declaration,
                "the grouped declaration was admitted on a different configured domain handle",
            ),
        ));
    }
    lower_grouped_contributions_on_handle(
        handle,
        declaration,
        shared_contributions,
        member_contributions,
        materialization_profile,
    )
}

fn lower_grouped_contributions_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: crate::application::WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D> + Clone,
>(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<D, C>,
    declaration: WorthQueryGroupedDeclarationArtifact<D, I>,
    shared_contributions: Vec<WorthQueryContributionIntent>,
    member_contributions: Vec<WorthQueryGroupedContributionAssignment>,
    materialization_profile: Option<FoundationalProfileSet>,
) -> Result<WorthQueryGroupedContributionComposition<D, I>, WorthQueryGroupedContributionStop<D, I>>
{
    let mut members = Vec::with_capacity(declaration.members().len());
    for member in declaration.members() {
        let member_specific = member_contributions
            .iter()
            .filter(|value| value.member_index() == member.member_index())
            .map(|value| value.contribution().clone())
            .collect::<Vec<_>>();
        let checked = orchestrate_progressed_declaration_with_contributions_checked_on_handle(
            handle,
            member.progression().clone(),
            member_contributions_for_member(&shared_contributions, &member_specific),
            materialization_policy(materialization_profile),
        )
        .map_err(WorthQueryGroupedContributionStop::EvidenceIdentityDerivationDenied)?;
        let context = WorthQueryGroupedContributionMemberContext::new(
            member.member_index(),
            member.role(),
            member.aspect_record().clone(),
            shared_contributions.len(),
            member_specific.len(),
        );
        match checked {
            WorthQueryContributionComposedOrchestrationChecked::Bound(bound) => {
                members.push((context, bound));
            }
            stop => {
                return Err(WorthQueryGroupedContributionStop::MemberStopped(
                    context, stop,
                ))
            }
        }
    }
    Ok(WorthQueryGroupedContributionComposition::new(
        declaration,
        members,
    ))
}

fn member_contributions_for_member(
    shared: &[WorthQueryContributionIntent],
    member_specific: &[WorthQueryContributionIntent],
) -> Vec<WorthQueryContributionIntent> {
    shared
        .iter()
        .cloned()
        .chain(member_specific.iter().cloned())
        .collect()
}

fn materialization_policy(
    profile: Option<FoundationalProfileSet>,
) -> WorthQueryContributionComposedMaterializationPolicy {
    match profile {
        Some(value) => WorthQueryContributionComposedMaterializationPolicy::Summary(value),
        None => WorthQueryContributionComposedMaterializationPolicy::None,
    }
}
