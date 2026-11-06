//! Contribution membership retains actual authored ownership through deferred expansion.
use super::{Builder, Denial, FragmentMember, FragmentMembers, Lowering};
use crate::application_schema::{ApplicationSchema, ApplicationSchemaContribution};
use std::marker::PhantomData;
#[doc(hidden)]
pub struct ContributionWitness<W>(PhantomData<W>);
impl<S, Set, C, W> FragmentMember<S, Set, ContributionWitness<W>> for C
where
    S: ApplicationSchema,
    C: ApplicationSchemaContribution<S>,
    C::Members: FragmentMembers<S, Set, W>,
{
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        let identity = C::IDENTITY;
        if !identity.is_valid() {
            return Err(Denial::InvalidContributionIdentity);
        }
        if !state.contributions.insert(identity.clone()) {
            return Err(Denial::DuplicateContributionIdentity);
        }
        let path = format!("{prefix}.{}", identity.as_str());
        state.names.insert(&path)?;
        let first_member = builder.contribution_member_count();
        let first_access = builder.member_provenance.fragments.access.len();
        let mut builder =
            <C::Members as FragmentMembers<S, Set, W>>::lower(builder, prefix, state)?;
        builder.retain_contribution_closure(identity, first_member, first_access);
        Ok(builder)
    }
    fn visit_correction<Visitor>(
        source: std::any::TypeId,
        visitor: Visitor,
    ) -> Result<Visitor::Output, Visitor>
    where
        S: ApplicationSchema,
        Visitor: crate::application_aftermath::ApplicationCorrectionVisitor<S>,
    {
        <C::Members as FragmentMembers<S, Set, W>>::visit_correction(source, visitor)
    }
}
