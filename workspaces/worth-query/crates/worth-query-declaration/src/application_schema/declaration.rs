use std::marker::PhantomData;

use super::aspect_contract_identity::ApplicationAspectMarkerIdentity;
use super::capabilities::{ApplicationFieldUnit, EqualityPosture, WritePosture};
use super::contribution::{
    ApplicationSchemaContributionIdentity, ApplicationSchemaContributionProvenance,
    AuthoredApplicationSchemaContribution,
};
use super::declaration_denial::ApplicationSchemaDeclarationDenial;
use super::field_reference::ApplicationFieldRef;
use super::member_provenance::ApplicationSchemaMemberProvenance;
use super::principal_binding_reference::ApplicationPrincipalBindingRef;
use super::references::{
    ApplicationAspectRef, ApplicationEffectRef, ApplicationEntityRef, ApplicationOperationRef,
    ApplicationRelationRef, ApplicationUnitRef,
};
use super::schema_identity::ApplicationSchemaIdentity;
use super::schema_member::ApplicationSchemaMember;

mod authorization;
mod finalization;

pub trait ApplicationSchema: Sized + 'static {
    const OWNER: &'static str;
    const NAME: &'static str;
    const MAJOR: u32;
    const MINOR: u32;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErasedApplicationSchemaDeclaration {
    owner: String,
    name: String,
    major: u32,
    minor: u32,
    identity: ApplicationSchemaIdentity,
    members: Vec<ApplicationSchemaMember>,
    contributions: Vec<ApplicationSchemaContributionProvenance>,
}

impl ErasedApplicationSchemaDeclaration {
    pub(super) fn from_fresh_parts(
        owner: String,
        name: String,
        major: u32,
        minor: u32,
        identity: ApplicationSchemaIdentity,
        members: Vec<ApplicationSchemaMember>,
        contributions: Vec<ApplicationSchemaContributionProvenance>,
    ) -> Self {
        Self {
            owner,
            name,
            major,
            minor,
            identity,
            members,
            contributions,
        }
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn major(&self) -> u32 {
        self.major
    }

    pub const fn minor(&self) -> u32 {
        self.minor
    }

    pub fn identity(&self) -> &ApplicationSchemaIdentity {
        &self.identity
    }

    pub fn members(&self) -> &[ApplicationSchemaMember] {
        &self.members
    }

    pub fn contributions(&self) -> &[ApplicationSchemaContributionProvenance] {
        &self.contributions
    }
}

pub struct ApplicationSchemaDeclaration<Schema> {
    erased: ErasedApplicationSchemaDeclaration,
    pub(super) member_provenance: ApplicationSchemaMemberProvenance,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> Clone for ApplicationSchemaDeclaration<Schema> {
    fn clone(&self) -> Self {
        Self {
            erased: self.erased.clone(),
            member_provenance: self.member_provenance.clone(),
            _schema: PhantomData,
        }
    }
}

impl<Schema> std::fmt::Debug for ApplicationSchemaDeclaration<Schema> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationSchemaDeclaration")
            .field("erased", &self.erased)
            .finish_non_exhaustive()
    }
}

impl<Schema> PartialEq for ApplicationSchemaDeclaration<Schema> {
    fn eq(&self, other: &Self) -> bool {
        self.erased == other.erased
    }
}

impl<Schema> Eq for ApplicationSchemaDeclaration<Schema> {}

impl<Schema> ApplicationSchemaDeclaration<Schema> {
    pub fn identity(&self) -> &ApplicationSchemaIdentity {
        self.erased.identity()
    }

    pub fn erased(&self) -> &ErasedApplicationSchemaDeclaration {
        &self.erased
    }

    pub fn into_erased(self) -> ErasedApplicationSchemaDeclaration {
        self.erased
    }

    pub fn contributions(&self) -> &[ApplicationSchemaContributionProvenance] {
        self.erased.contributions()
    }

    #[doc(hidden)]
    pub fn member_provenance(&self) -> &ApplicationSchemaMemberProvenance {
        &self.member_provenance
    }
}

#[derive(Clone, Debug)]
pub struct ApplicationSchemaDeclarationBuilder<Schema> {
    owner: &'static str,
    name: &'static str,
    major: u32,
    minor: u32,
    members: Vec<ApplicationSchemaMember>,
    contributions: Vec<AuthoredApplicationSchemaContribution>,
    pub(super) member_provenance: ApplicationSchemaMemberProvenance,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn invariant<Invariant>(
        mut self,
        definition: super::ApplicationInvariantDefinition<Schema, Invariant>,
    ) -> Self
    where
        Invariant: super::ApplicationInvariantMarkerIdentity<Schema>,
    {
        let reference = definition.reference();
        let operational = definition.operational();
        self.members
            .push(ApplicationSchemaMember::ApplicationInvariant {
                invariant: reference.identifier().to_owned(),
                major: reference.major(),
                minor: reference.minor(),
                execution_point: definition.execution_point(),
                maximum_work_units: definition.work_budget().maximum_work_units(),
                enforcement: operational.enforcement(),
                required_groups: operational.required_groups().to_vec(),
                read_closure: operational.read_closure().to_vec(),
                applicability: operational.applicability().to_vec(),
                provider: operational.provider().to_owned(),
                cost_posture: operational.cost_posture(),
            });
        self
    }

    #[cfg(test)]
    pub(crate) fn from_test_members(members: Vec<ApplicationSchemaMember>) -> Self {
        Self {
            owner: "WORTH.tests",
            name: "raw-member-builder-proof",
            major: 1,
            minor: 0,
            members,
            contributions: Vec::new(),
            member_provenance: ApplicationSchemaMemberProvenance::default(),
            _schema: PhantomData,
        }
    }

    pub(super) fn push_member(mut self, member: ApplicationSchemaMember) -> Self {
        self.members.push(member);
        self
    }

    pub(super) fn push_member_in_place(&mut self, member: ApplicationSchemaMember) {
        self.members.push(member);
    }

    pub(super) fn contribution_member_count(&self) -> usize {
        self.members.len()
    }

    pub(super) fn retain_contribution_closure(
        &mut self,
        identity: ApplicationSchemaContributionIdentity,
        first_member: usize,
    ) {
        let members = self.members[first_member..].to_vec();
        self.contributions
            .push(AuthoredApplicationSchemaContribution::new(
                identity, members,
            ));
    }

    pub fn for_schema() -> Self
    where
        Schema: ApplicationSchema,
    {
        ApplicationSchemaDeclarationBuilder {
            owner: Schema::OWNER,
            name: Schema::NAME,
            major: Schema::MAJOR,
            minor: Schema::MINOR,
            members: Vec::new(),
            contributions: Vec::new(),
            member_provenance: ApplicationSchemaMemberProvenance::default(),
            _schema: PhantomData,
        }
    }

    pub fn entity<Entity>(mut self, reference: ApplicationEntityRef<Schema, Entity>) -> Self {
        self.members.push(ApplicationSchemaMember::Entity {
            entity: reference.name().to_string(),
        });
        self
    }

    pub fn aspect<Entity, Aspect>(
        mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        aspect: ApplicationAspectRef<Schema, Entity, Aspect>,
    ) -> Self
    where
        Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>,
    {
        self.members.push(ApplicationSchemaMember::Aspect {
            entity: entity.name().to_string(),
            aspect: aspect.name().to_string(),
            identity: Aspect::ASPECT_IDENTITY,
            revision: Aspect::CONTRACT_REVISION,
        });
        self
    }

    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Entity: super::ApplicationEntityMarkerIdentity<Schema>,
        Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>,
        Field: super::ApplicationFieldMarkerIdentity<Schema, Entity, Aspect, Value = Value>,
        Write: WritePosture,
        Equality: EqualityPosture,
        Unit: ApplicationFieldUnit,
    {
        let recipe = field.binding_recipe();
        self.member_provenance
            .register_field_binding(recipe.clone());
        self.members.push(ApplicationSchemaMember::Field {
            entity: entity.name().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
            presence: Field::PRESENCE,
            scalar_family: field.scalar_family(),
            value_type: field.value_type_name().to_string(),
            unit: field.unit().map(str::to_string),
            frame: recipe.frame().map(|frame| frame.as_str().to_owned()),
            writable: Write::WRITABLE,
            equality_queryable: Equality::QUERYABLE,
        });
        self
    }

    pub fn relation<Relation, From, To>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: ApplicationEntityRef<Schema, From>,
        to: ApplicationEntityRef<Schema, To>,
    ) -> Self {
        self.members.push(ApplicationSchemaMember::Relation {
            relation: relation.name().to_string(),
            from: from.name().to_string(),
            to: to.name().to_string(),
            integrity: relation.integrity(),
        });
        self
    }

    pub fn principal_binding<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        mut self,
        binding: ApplicationPrincipalBindingRef<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
    ) -> Self {
        self.members
            .push(ApplicationSchemaMember::PrincipalBinding {
                binding: binding.name().to_string(),
                mapping_entity: binding.mapping_entity().to_string(),
                identity_aspect: binding.identity_aspect().to_string(),
                identity_field: binding.identity_field().to_string(),
                status_aspect: binding.status_aspect().to_string(),
                status_field: binding.status_field().to_string(),
                target_relation: binding.target_relation().to_string(),
                principal_entity: binding.principal_entity().to_string(),
                principal_identity_aspect: binding.principal_identity_aspect().to_string(),
                principal_identity_field: binding.principal_identity_field().to_string(),
                principal_identity_scalar_family: binding.principal_identity_scalar_family(),
                principal_identity_value_type: binding.principal_identity_value_type().to_string(),
            });
        self
    }

    pub fn unit<Unit>(mut self, unit: ApplicationUnitRef<Schema, Unit>) -> Self {
        self.members.push(ApplicationSchemaMember::Unit {
            unit: unit.name().to_string(),
        });
        self
    }

    pub fn effect<Effect, Payload>(
        mut self,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
    ) -> Self
    where
        Effect: 'static,
        Payload: 'static,
    {
        self.member_provenance
            .register_effect::<Effect, Payload>(effect.name(), effect.payload_identity());
        self.members.push(ApplicationSchemaMember::Effect {
            effect: effect.name().to_string(),
            payload_type: effect.payload_identity(),
        });
        self
    }
}
