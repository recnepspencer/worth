//! Exact declared fields of an original mutation's created output.
use crate::{
    application_operation::{
        ApplicationMutationBinding, WorthQueryApplicationDeclaredOutputRole,
        WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
    },
    application_schema::{
        ApplicationCanonicalValueContract, ApplicationEntityMarkerIdentity,
        ApplicationFieldBindingRecipe, ApplicationFieldPresence, ApplicationSchemaMember,
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
    pub content_format: Option<[u8; 32]>,
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
            content_format: parts.content_format,
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
            content_format,
            unit,
            frame,
            ..
        } = member
        else {
            return false;
        };
        let expected = &self.parts;
        expected.content_format == *content_format
            && expected.entity == *entity
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

/// An unchecked schema-affine selection of an original output field.
#[derive(Clone, Debug)]
pub struct ApplicationCorrectionOutputField<Schema> {
    metadata: std::sync::Arc<OutputFieldMetadata>,
    source: TypeId,
    output_contract: TypeId,
    entity: TypeId,
    entity_name: &'static str,
    recipe: ApplicationFieldBindingRecipe,
    schema: PhantomData<fn() -> Schema>,
}
struct OutputFieldMetadata {
    resolve: Box<dyn Fn() -> Result<ResolvedOutputField, Denial> + Send + Sync>,
    value: std::sync::OnceLock<Result<ResolvedOutputField, Denial>>,
}
struct ResolvedOutputField {
    declaration: DeclaredCorrectionOutputField,
    content: Option<crate::application_schema::ApplicationContentBindingRecipe>,
}
use crate::application_schema::ApplicationSchemaDeclarationDenial as Denial;
impl std::fmt::Debug for OutputFieldMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputFieldMetadata")
            .finish_non_exhaustive()
    }
}
impl<S> ApplicationCorrectionOutputField<S> {
    fn checked(&self) -> Result<&ResolvedOutputField, Denial> {
        self.metadata
            .value
            .get_or_init(|| (self.metadata.resolve)())
            .as_ref()
            .map_err(Clone::clone)
    }
    pub fn declaration(&self) -> Result<&DeclaredCorrectionOutputField, Denial> {
        Ok(&self.checked()?.declaration)
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
    pub fn content_recipe(
        &self,
    ) -> Result<Option<&crate::application_schema::ApplicationContentBindingRecipe>, Denial> {
        Ok(self.checked()?.content.as_ref())
    }
    pub fn binding_recipe(&self) -> &ApplicationFieldBindingRecipe {
        &self.recipe
    }
    pub fn into_declaration(self) -> Result<DeclaredCorrectionOutputField, Denial> {
        self.declaration().cloned()
    }
}

/// Typed construction context for one source mutation's exactly-one created role.
pub struct ApplicationCorrectionOutput<Source, Role>(PhantomData<fn() -> (Source, Role)>);
impl<Source, Role> ApplicationCorrectionOutput<Source, Role>
where
    Role: WorthQueryApplicationOutputRole<
        Action = WorthQueryCreateOutput,
        Cardinality = WorthQueryExactlyOneOutput,
    >,
    Source: ApplicationMutationBinding<Role::Schema, Output = Role::Contract>,
{
    pub fn content_field<Field, C>(
        field: crate::application_schema::ApplicationContentFieldRef<Field, C>,
    ) -> ApplicationCorrectionOutputField<Role::Schema>
    where
        Field::Binding: crate::application_schema::ApplicationContentReferenceBinding,
        Field: crate::application_schema::fragment::RecordSelection<Entity = Role::Entity>,
        C: crate::application_schema::ApplicationContentBinding,
    {
        Self::selection(
            field.field(),
            Some(crate::application_schema::ApplicationContentBindingRecipe::of::<C>()),
        )
    }
    pub fn field<Field>(field: Field) -> ApplicationCorrectionOutputField<Role::Schema>
    where
        Field: crate::application_schema::fragment::RecordSelection<Entity = Role::Entity>,
    {
        Self::selection(field, None)
    }
    fn selection<Field>(
        field: Field,
        requested: Option<crate::application_schema::ApplicationContentBindingRecipe>,
    ) -> ApplicationCorrectionOutputField<Role::Schema>
    where
        Field: crate::application_schema::fragment::RecordSelection<Entity = Role::Entity>,
    {
        const { <Role as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        ApplicationCorrectionOutputField {
            metadata: std::sync::Arc::new(OutputFieldMetadata {
                value: std::sync::OnceLock::new(),
                resolve: Box::new(move || {
                    let checked = field.checked()?;
                    if requested
                        .as_ref()
                        .is_some_and(|requested| checked.content_format() != Some(requested))
                    {
                        return Err(Denial::InvalidRecordField {
                            field: field.expected_recipe().locus().clone(),
                        });
                    }
                    let recipe = checked.recipe();
                    Ok(ResolvedOutputField {
                        declaration: DeclaredCorrectionOutputField::from_untrusted_parts(
                            DeclaredCorrectionOutputFieldParts {
                                source_binding: Source::descriptor()
                                    .description()
                                    .binding_identity()
                                    .clone(),
                                role: Role::NAME.to_owned(),
                                entity: <Role::Entity as ApplicationEntityMarkerIdentity<
                                    Role::Schema,
                                >>::IDENTIFIER
                                    .to_owned(),
                                aspect: field.aspect().to_owned(),
                                field: field.field().to_owned(),
                                presence: checked.presence(),
                                scalar_family: recipe.scalar_family(),
                                value_binding: recipe.binding_identity().clone(),
                                unit: recipe.unit().map(|unit| unit.as_str().to_owned()),
                                binding_unit: recipe.unit().map(|unit| unit.as_str().to_owned()),
                                frame: recipe.frame().map(|frame| frame.as_str().to_owned()),
                                canonical: recipe.canonical_contract().cloned(),
                                content_format: checked
                                    .content_format()
                                    .map(|recipe| recipe.declaration().identity),
                            },
                        ),
                        content: checked.content_format().cloned(),
                    })
                }),
            }),
            source: TypeId::of::<Source>(),
            output_contract: TypeId::of::<Role::Contract>(),
            entity: TypeId::of::<Role::Entity>(),
            entity_name:
                <Role::Entity as ApplicationEntityMarkerIdentity<Role::Schema>>::IDENTIFIER,
            recipe: field.expected_recipe(),
            schema: PhantomData,
        }
    }
}
