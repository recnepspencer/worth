use super::ApplicationRelationFieldDefinition;
use worth_foundational::facade::{AbsenceLaw, AspectContractRevision, AspectIdentity};
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
            || parts.fields.iter().any(|field| {
                super::super::identifier_validation::validate_field_identifier(&field.field)
                    .is_err()
                    || field.value_type.is_empty()
            })
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

fn valid(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control() || ch == '.')
}
