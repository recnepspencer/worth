//! Exact declared fields of an original mutation's created output.
use crate::{
    application_operation::{
        ApplicationMutationBinding, WorthQueryApplicationDeclaredOutputRole,
        WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
    },
    application_schema::{
        ApplicationAspectMarkerIdentity, ApplicationCanonicalValueContract,
        ApplicationEntityMarkerIdentity, ApplicationFieldBindingRecipe,
        ApplicationFieldMarkerIdentity, ApplicationFieldPresence, ApplicationFieldRef,
        ApplicationFieldUnit, ApplicationSchemaMember,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};
use std::{any::TypeId, marker::PhantomData};
use worth_foundational::facade::ScalarAspectType;

/// Portable meaning of one declared historical field. Untrusted reconstruction
/// still requires native source, role and field validation at installation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredCorrectionOutputField {
    parts: DeclaredCorrectionOutputFieldParts,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredCorrectionOutputFieldParts {
    pub source_binding: WorthQueryPortableTypeIdentity,
    pub role: String,
    pub entity: String,
    pub aspect: String,
    pub field: String,
    pub presence: ApplicationFieldPresence,
    pub scalar_family: ScalarAspectType,
    pub value_binding: WorthQueryPortableTypeIdentity,
    pub unit: Option<String>,
    pub binding_unit: Option<String>,
    pub frame: Option<String>,
    pub canonical: Option<ApplicationCanonicalValueContract>,
}

impl DeclaredCorrectionOutputField {
    pub fn from_untrusted_parts(parts: DeclaredCorrectionOutputFieldParts) -> Self {
        Self { parts }
    }
    pub fn parts(&self) -> &DeclaredCorrectionOutputFieldParts {
        &self.parts
    }
    /// Read-only value shape for the existing canonical and archive field codecs.
    pub fn projected_value_member(&self) -> ApplicationSchemaMember {
        let parts = &self.parts;
        ApplicationSchemaMember::Field {
            entity: parts.entity.clone(),
            aspect: parts.aspect.clone(),
            field: parts.field.clone(),
            presence: parts.presence,
            scalar_family: parts.scalar_family,
            value_type: parts.value_binding.as_str().to_owned(),
            canonical: parts.canonical.clone(),
            unit: parts.unit.clone(),
            frame: parts.frame.clone(),
            writable: false,
            equality_queryable: false,
            unique: false,
        }
    }

    /// Compares complete declared value meaning, not name-only field equality.
    pub fn matches_schema_field(&self, member: &ApplicationSchemaMember) -> bool {
        let ApplicationSchemaMember::Field {
            entity,
            aspect,
            field,
            presence,
            scalar_family,
            value_type,
            canonical,
            unit,
            frame,
            ..
        } = member
        else {
            return false;
        };
        let expected = &self.parts;
        expected.entity == *entity
            && expected.aspect == *aspect
            && expected.field == *field
            && expected.presence == *presence
            && expected.scalar_family == *scalar_family
            && expected.value_binding.as_str() == value_type
            && expected.unit == *unit
            && expected.binding_unit == *unit
            && expected.frame == *frame
            && expected.canonical == *canonical
    }
}

/// Schema-affine witness made from a concrete source, declared role and field.
/// Native identities qualify the portable declaration; they grant no read authority.
#[derive(Clone, Debug)]
pub struct ApplicationCorrectionOutputField<Schema> {
    declaration: DeclaredCorrectionOutputField,
    source: TypeId,
    output_contract: TypeId,
    entity: TypeId,
    entity_name: &'static str,
    recipe: ApplicationFieldBindingRecipe,
    schema: PhantomData<fn() -> Schema>,
}

impl<S> ApplicationCorrectionOutputField<S> {
    pub fn declaration(&self) -> &DeclaredCorrectionOutputField {
        &self.declaration
    }
    pub fn source_binding_type(&self) -> TypeId {
        self.source
    }
    pub fn output_contract_type(&self) -> TypeId {
        self.output_contract
    }
    pub fn entity_type(&self) -> TypeId {
        self.entity
    }
    pub fn entity_name(&self) -> &'static str {
        self.entity_name
    }
    pub fn binding_recipe(&self) -> &ApplicationFieldBindingRecipe {
        &self.recipe
    }
    pub fn into_declaration(self) -> DeclaredCorrectionOutputField {
        self.declaration
    }
}

/// Typed construction context for one source mutation's exactly-one created role.
/// The field's entity and schema are fixed by that role.
pub struct ApplicationCorrectionOutput<Source, Role>(PhantomData<fn() -> (Source, Role)>);
impl<Source, Role> ApplicationCorrectionOutput<Source, Role>
where
    Role: WorthQueryApplicationOutputRole<
        Action = WorthQueryCreateOutput,
        Cardinality = WorthQueryExactlyOneOutput,
    >,
    Source: ApplicationMutationBinding<Role::Schema, Output = Role::Contract>,
{
    pub fn field<Aspect, Field, Value, Write, Equality, Unit>(
        field: ApplicationFieldRef<
            Role::Schema,
            Role::Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> ApplicationCorrectionOutputField<Role::Schema>
    where
        Aspect: ApplicationAspectMarkerIdentity<Role::Schema, Role::Entity>,
        Field: ApplicationFieldMarkerIdentity<Role::Schema, Role::Entity, Aspect, Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        const { <Role as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        let recipe = field.binding_recipe();
        ApplicationCorrectionOutputField {
            declaration: DeclaredCorrectionOutputField::from_untrusted_parts(
                DeclaredCorrectionOutputFieldParts {
                    source_binding: Source::descriptor()
                        .description()
                        .binding_identity()
                        .clone(),
                    role: Role::NAME.to_owned(),
                    entity:
                        <Role::Entity as ApplicationEntityMarkerIdentity<Role::Schema>>::IDENTIFIER
                            .to_owned(),
                    aspect: field.aspect().to_owned(),
                    field: field.field().to_owned(),
                    presence: Field::PRESENCE,
                    scalar_family: recipe.scalar_family(),
                    value_binding: recipe.binding_identity().clone(),
                    unit: field.unit().map(str::to_owned),
                    binding_unit: recipe.unit().map(|unit| unit.as_str().to_owned()),
                    frame: recipe.frame().map(|frame| frame.as_str().to_owned()),
                    canonical: recipe.canonical_contract().cloned(),
                },
            ),
            source: TypeId::of::<Source>(),
            output_contract: TypeId::of::<Role::Contract>(),
            entity: TypeId::of::<Role::Entity>(),
            entity_name:
                <Role::Entity as ApplicationEntityMarkerIdentity<Role::Schema>>::IDENTIFIER,
            recipe,
            schema: PhantomData,
        }
    }
}
