//! Explicit derived component meaning lowers to the ordinary Native coordinator.
use super::{
    ApplicationComponentSetRef, ApplicationDerivedRelationRef, ApplicationGraphMemberMeaning,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial as Denial,
    ApplicationSchemaMember,
};

/// One homogeneous derived source, distinct from the legacy physical roster.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationDerivedComponentSetMeaning {
    pub name: String,
    pub member: ApplicationGraphMemberMeaning,
    pub relation: String,
}

pub struct ApplicationDerivedComponentSetDefinition<Schema, Set, Member> {
    meaning: ApplicationDerivedComponentSetMeaning,
    marker: std::marker::PhantomData<fn() -> (Schema, Set, Member)>,
}
impl<S, Set, M> ApplicationDerivedComponentSetDefinition<S, Set, M> {
    pub fn new<R>(
        reference: ApplicationComponentSetRef<S, Set, M>,
        source: ApplicationDerivedRelationRef<S, R, M, M>,
    ) -> Self {
        Self {
            meaning: ApplicationDerivedComponentSetMeaning {
                name: reference.name().to_owned(),
                member: reference.member_selector().into_meaning(),
                relation: source.name().to_owned(),
            },
            marker: std::marker::PhantomData,
        }
    }
}
impl<S> ApplicationSchemaDeclarationBuilder<S> {
    pub fn derived_components<Set, M>(
        self,
        definition: ApplicationDerivedComponentSetDefinition<S, Set, M>,
    ) -> Self {
        self.push_member(ApplicationSchemaMember::DerivedComponentSet {
            definition: definition.meaning,
        })
    }
}

impl ApplicationDerivedComponentSetMeaning {
    /// Checks local descriptive syntax; completed schema closure is independent.
    pub fn validate(&self) -> Result<(), Denial> {
        self.validate_identifiers()
    }
    pub(crate) fn validate_identifiers(&self) -> Result<(), Denial> {
        for name in [
            self.name.as_str(),
            self.member.name(),
            self.relation.as_str(),
        ] {
            super::identifier_validation::validate_simple_identifier(name)?;
        }
        Ok(())
    }
    pub(crate) fn validate_closure(
        &self,
        members: &[ApplicationSchemaMember],
    ) -> Result<(), Denial> {
        self.validate_identifiers()?;
        if !members.iter().any(|member| matches!(member, ApplicationSchemaMember::DerivedRelation { definition }
            if definition.parts().relation == self.relation && definition.parts().from == self.member && definition.parts().to == self.member)) {
            return Err(Denial::InvalidComponentSet);
        }
        Ok(())
    }
}
