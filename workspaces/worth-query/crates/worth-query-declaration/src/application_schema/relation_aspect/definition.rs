use std::marker::PhantomData;

use worth_foundational::facade::{AbsenceLaw, AspectContractRevision, AspectIdentity};

use super::{ApplicationRelationFieldDefinition, ApplicationRelationFieldRef};
use crate::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationAspectRef, ApplicationFieldBindingRecipe,
    ApplicationFieldMarkerIdentity, ApplicationFieldUnit, ApplicationRelationRef,
    ApplicationScalarValueBinding, WritePosture,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryPortableRelationAspectParts {
    pub relation: String,
    pub from: String,
    pub to: String,
    pub aspect: String,
    pub identity: AspectIdentity,
    pub revision: AspectContractRevision,
    pub absence: AbsenceLaw,
    pub fields: Vec<ApplicationRelationFieldDefinition>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ErasedApplicationRelationAspectDefinition {
    parts: WorthQueryPortableRelationAspectParts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationRelationAspectDefinitionDenial {
    InvalidIdentifier,
    RevisionZero,
    EmptyFields,
    DuplicateField,
    NonCanonicalFields,
}

impl ErasedApplicationRelationAspectDefinition {
    pub fn from_untrusted_parts(parts: WorthQueryPortableRelationAspectParts) -> Self {
        Self { parts }
    }
    pub fn parts(&self) -> &WorthQueryPortableRelationAspectParts {
        &self.parts
    }

    pub fn validate(&self) -> Result<(), ApplicationRelationAspectDefinitionDenial> {
        use ApplicationRelationAspectDefinitionDenial as Denial;
        let parts = &self.parts;
        if [&parts.relation, &parts.from, &parts.to, &parts.aspect]
            .into_iter()
            .any(|value| !valid(value))
            || parts
                .fields
                .iter()
                .any(|field| !valid(&field.field) || field.value_type.is_empty())
        {
            return Err(Denial::InvalidIdentifier);
        }
        if parts.revision.0 == 0 {
            return Err(Denial::RevisionZero);
        }
        if parts.fields.is_empty() {
            return Err(Denial::EmptyFields);
        }
        for pair in parts.fields.windows(2) {
            match pair[0].field.cmp(&pair[1].field) {
                std::cmp::Ordering::Equal => return Err(Denial::DuplicateField),
                std::cmp::Ordering::Greater => return Err(Denial::NonCanonicalFields),
                std::cmp::Ordering::Less => {}
            }
        }
        Ok(())
    }
}

/// Typed relation-aspect declaration plus compiler-local scalar binding recipes.
pub struct ApplicationRelationAspectDefinition<Schema, Relation, From, To, Aspect> {
    pub(super) meaning: ErasedApplicationRelationAspectDefinition,
    pub(super) recipes: Vec<ApplicationFieldBindingRecipe>,
    _relation: PhantomData<fn() -> (Schema, Relation, From, To, Aspect)>,
}

pub struct ApplicationRelationAspectDefinitionBuilder<Schema, Relation, From, To, Aspect> {
    definition: ApplicationRelationAspectDefinition<Schema, Relation, From, To, Aspect>,
}

impl<Schema, Relation, From, To, Aspect>
    ApplicationRelationAspectDefinition<Schema, Relation, From, To, Aspect>
where
    Aspect: ApplicationAspectMarkerIdentity<Schema, Relation>,
{
    pub fn new(
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        aspect: ApplicationAspectRef<Schema, Relation, Aspect>,
        absence: AbsenceLaw,
    ) -> ApplicationRelationAspectDefinitionBuilder<Schema, Relation, From, To, Aspect> {
        ApplicationRelationAspectDefinitionBuilder {
            definition: Self {
                meaning: ErasedApplicationRelationAspectDefinition::from_untrusted_parts(
                    WorthQueryPortableRelationAspectParts {
                        relation: relation.name().to_owned(),
                        from: relation.from().to_owned(),
                        to: relation.to().to_owned(),
                        aspect: aspect.name().to_owned(),
                        identity: aspect.identity(),
                        revision: aspect.revision(),
                        absence,
                        fields: Vec::new(),
                    },
                ),
                recipes: Vec::new(),
                _relation: PhantomData,
            },
        }
    }
}

impl<Schema, Relation, From, To, Aspect>
    ApplicationRelationAspectDefinitionBuilder<Schema, Relation, From, To, Aspect>
where
    Aspect: ApplicationAspectMarkerIdentity<Schema, Relation>,
{
    pub fn field<Field, Write, Unit>(
        mut self,
        field: ApplicationRelationFieldRef<Schema, Relation, From, To, Aspect, Field, Write, Unit>,
    ) -> Self
    where
        Field: ApplicationFieldMarkerIdentity<Schema, Relation, Aspect>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let recipe = field.binding_recipe();
        self.definition
            .meaning
            .parts
            .fields
            .push(ApplicationRelationFieldDefinition {
                field: field.field().to_owned(),
                presence: Field::PRESENCE,
                scalar_family: Field::Binding::SCALAR_FAMILY,
                value_type: Field::Binding::IDENTITY_NAME.to_owned(),
                canonical: recipe.canonical_contract().cloned(),
                unit: field.unit().map(str::to_owned),
                frame: recipe.frame().map(|frame| frame.as_str().to_owned()),
                writable: Write::WRITABLE,
            });
        self.definition.recipes.push(recipe);
        self
    }

    pub fn finish(
        mut self,
    ) -> Result<
        ApplicationRelationAspectDefinition<Schema, Relation, From, To, Aspect>,
        ApplicationRelationAspectDefinitionDenial,
    > {
        self.definition
            .meaning
            .parts
            .fields
            .sort_by(|left, right| left.field.cmp(&right.field));
        self.definition.meaning.validate()?;
        Ok(self.definition)
    }
}

fn valid(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control() || ch == '.')
}
