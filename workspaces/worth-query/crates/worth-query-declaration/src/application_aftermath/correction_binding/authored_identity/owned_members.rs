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
    /// Restore this one member under its original live owner, by its family key.
    /// The relation must have exactly one target per live member and be retained
    /// by the recorded inverse's relation demand.
    pub fn owned_member_of<Owner, Member, Anchor>(
        mut self,
        owner: ApplicationRelationRef<S, Owner, Member, Anchor>,
    ) -> Self {
        let owner = PortablePreImageRelationLocus::from_untrusted_fields(
            owner.name().to_owned(),
            owner.from().to_owned(),
            owner.to().to_owned(),
        );
        let previous = self.compose;
        self.compose = Box::new(move || Ok(previous()?.with_live_member_owner(owner.clone())));
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
        const { <Source as WorthQueryApplicationDeclaredOutputRoleFamily>::DECLARED };
        const { <Inverse as WorthQueryApplicationDeclaredOutputRoleFamily>::DECLARED };
        let owner = PortablePreImageRelationLocus::from_untrusted_fields(
            owner.name().to_owned(),
            owner.from().to_owned(),
            owner.to().to_owned(),
        );
        let member = Self::from_family::<Family>(
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
        );
        self.co_recipes.push(member.binding_recipe().clone());
        let previous = self.compose;
        self.compose = Box::new(move || {
            let anchor = previous()?;
            let member = (member.compose)()?;
            Ok(anchor.with_co_anchor(member.with_member_owner(owner.clone()), false))
        });
        self
    }
}
