//! Variable authored membership is declared by its existing relation and output families.
use super::*;
use crate::application_aftermath::{
    ApplicationCorrectionBinding, ApplicationInverseBinding, PortablePreImageRelationLocus,
};
use crate::application_operation::{
    ApplicationMutationBinding, WorthQueryApplicationDeclaredOutputRoleFamily,
    WorthQueryApplicationOutputRoleFamily,
};
use crate::application_schema::ApplicationRelationRef;

impl<S: ApplicationSchema> ApplicationAuthoredCorrection<S> {
    /// Zero to many members of one family leave or join an owner that stays
    /// live. The recorded inverse returns each one through the inverse family
    /// role with the same suffix; the family's logical key alone decides which
    /// fresh member answers which original. The owner relation must have exactly
    /// one target per live member and be retained by the relation demand.
    pub fn live_owner_members<C, Family, Source, Inverse, Owner, Anchor>(
        owner: ApplicationRelationRef<S, Owner, Family::Entity, Anchor>,
    ) -> Self
    where
        C: ApplicationCorrectionBinding<S>,
        Family: ApplicationAuthoredCorrectionFamily<S>,
        Source: WorthQueryApplicationOutputRoleFamily<
            Schema = S,
            Entity = Family::Entity,
            Contract = <C::Source as ApplicationMutationBinding<S>>::Output,
        >,
        Inverse: WorthQueryApplicationOutputRoleFamily<
            Schema = S,
            Entity = Family::Entity,
            Contract = <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Output,
        >,
    {
        let owner = owner_locus(&owner);
        let mut members = Self::member_family::<C, Family, Source, Inverse>();
        let family = members.compose;
        members.compose = Box::new(move || Ok(family()?.with_live_member_owner(owner.clone())));
        members
    }

    /// A further member family of the same live-owner inverse. Each family
    /// names its own owner relation and its own pair of role prefixes.
    pub fn and_live_owner_members<C, Family, Source, Inverse, Owner, Anchor>(
        mut self,
        owner: ApplicationRelationRef<S, Owner, Family::Entity, Anchor>,
    ) -> Self
    where
        C: ApplicationCorrectionBinding<S>,
        Family: ApplicationAuthoredCorrectionFamily<S>,
        Source: WorthQueryApplicationOutputRoleFamily<
            Schema = S,
            Entity = Family::Entity,
            Contract = <C::Source as ApplicationMutationBinding<S>>::Output,
        >,
        Inverse: WorthQueryApplicationOutputRoleFamily<
            Schema = S,
            Entity = Family::Entity,
            Contract = <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Output,
        >,
    {
        let owner = owner_locus(&owner);
        let member = Self::member_family::<C, Family, Source, Inverse>();
        self.co_recipes.push(member.binding_recipe().clone());
        let previous = self.compose;
        self.compose = Box::new(move || {
            let first = previous()?;
            let member = (member.compose)()?;
            Ok(first.with_co_anchor(member.with_live_member_owner(owner.clone()), false))
        });
        self
    }

    /// Join a variable-size member kind to this anchor's lifetime inverse.
    /// Source and inverse output families must have opposite lifecycle postures.
    /// Installation requires an exactly-one member-to-anchor owner relation.
    pub fn owned_members<C, Family, Source, Inverse, Owner, Anchor>(
        mut self,
        owner: ApplicationRelationRef<S, Owner, Family::Entity, Anchor>,
    ) -> Self
    where
        C: ApplicationCorrectionBinding<S>,
        Family: ApplicationAuthoredCorrectionFamily<S>,
        Source: WorthQueryApplicationOutputRoleFamily<
            Schema = S,
            Entity = Family::Entity,
            Contract = <C::Source as ApplicationMutationBinding<S>>::Output,
        >,
        Inverse: WorthQueryApplicationOutputRoleFamily<
            Schema = S,
            Entity = Family::Entity,
            Contract = <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Output,
        >,
    {
        let owner = owner_locus(&owner);
        let member = Self::member_family::<C, Family, Source, Inverse>();
        self.co_recipes.push(member.binding_recipe().clone());
        let previous = self.compose;
        self.compose = Box::new(move || {
            let anchor = previous()?;
            let member = (member.compose)()?;
            Ok(anchor.with_co_anchor(member.with_member_owner(owner.clone()), false))
        });
        self
    }

    /// The source family's single posture fixes the correspondence direction.
    fn member_family<C, Family, Source, Inverse>() -> Self
    where
        C: ApplicationCorrectionBinding<S>,
        Family: ApplicationAuthoredCorrectionFamily<S>,
        Source: WorthQueryApplicationOutputRoleFamily<
            Schema = S,
            Entity = Family::Entity,
            Contract = <C::Source as ApplicationMutationBinding<S>>::Output,
        >,
        Inverse: WorthQueryApplicationOutputRoleFamily<
            Schema = S,
            Entity = Family::Entity,
            Contract = <ApplicationInverseBinding<S, C> as ApplicationMutationBinding<S>>::Output,
        >,
    {
        const { <Source as WorthQueryApplicationDeclaredOutputRoleFamily>::DECLARED };
        const { <Inverse as WorthQueryApplicationDeclaredOutputRoleFamily>::DECLARED };
        Self::from_family::<Family>(
            Source::PREFIX,
            Inverse::PREFIX,
            if Source::POSTURES
                == crate::application_operation::ApplicationMutationOutputPostureSet::CREATE
            {
                ApplicationMutationOutputPosture::Create
            } else if Source::POSTURES
                == crate::application_operation::ApplicationMutationOutputPostureSet::RETIRE
            {
                ApplicationMutationOutputPosture::Retire
            } else {
                ApplicationMutationOutputPosture::Preserve
            },
        )
    }
}

fn owner_locus<S, Owner, Member, Anchor>(
    owner: &ApplicationRelationRef<S, Owner, Member, Anchor>,
) -> PortablePreImageRelationLocus {
    PortablePreImageRelationLocus::from_untrusted_fields(
        owner.name().to_owned(),
        owner.from().to_owned(),
        owner.to().to_owned(),
    )
}
