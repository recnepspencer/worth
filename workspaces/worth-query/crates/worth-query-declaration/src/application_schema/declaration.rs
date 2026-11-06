use std::marker::PhantomData;

use super::contribution::{
    ApplicationSchemaContributionIdentity, ApplicationSchemaContributionProvenance,
    AuthoredApplicationSchemaContribution,
};
use super::declaration_denial::ApplicationSchemaDeclarationDenial;
use super::member_provenance::ApplicationSchemaMemberProvenance;
use super::principal_binding_reference::ApplicationPrincipalBindingRef;
use super::references::{ApplicationEffectRef, ApplicationOperationRef, ApplicationUnitRef};
use super::schema_identity::ApplicationSchemaIdentity;
use super::schema_member::ApplicationSchemaMember;

mod authorization;
mod components;
mod finalization;
mod selected_aspect;

pub trait ApplicationSchema: Sized + 'static {
    const OWNER: &'static str = "WORTH.application";
    const NAME: &'static str;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial>;

    fn visit_correction<
        Visitor: crate::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        _source_binding: std::any::TypeId,
        visitor: Visitor,
    ) -> Visitor::Output {
        visitor.unavailable()
    }
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
        self.members
            .extend(definition.managed_index_sources().map(|requirement| {
                ApplicationSchemaMember::InvariantManagedIndexSource {
                    invariant: reference.identifier().to_owned(),
                    execution_point: definition.execution_point(),
                    requirement,
                }
            }));
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
        first_access: usize,
    ) {
        let members = self.members[first_member..].to_vec();
        let access = self.member_provenance.fragments.access[first_access..].to_vec();
        self.contributions
            .push(AuthoredApplicationSchemaContribution::new(
                identity, members, access,
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
        for field in binding.mapping_field_bindings() {
            self.member_provenance.fragments.expect_field(field.clone());
        }
        self.member_provenance
            .fragments
            .expect_field(binding.principal_identity_binding_recipe().clone());
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
