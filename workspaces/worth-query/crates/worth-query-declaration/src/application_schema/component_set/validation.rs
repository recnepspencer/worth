//! Both authored and portable schemas validate the actual declared source graph.
use super::ApplicationComponentSetMeaning;
use crate::application_schema::ApplicationSchemaDeclarationDenial as Denial;
use std::collections::BTreeSet;

pub(in crate::application_schema) fn validate_identifiers(
    meaning: &ApplicationComponentSetMeaning,
) -> Result<(), Denial> {
    for name in std::iter::once(meaning.name())
        .chain(std::iter::once(meaning.member()))
        .chain(meaning.relations().iter().map(String::as_str))
    {
        super::super::identifier_validation::validate_simple_identifier(name)?;
    }
    Ok(())
}

pub(in crate::application_schema) fn validate_closure(
    definition: &ApplicationComponentSetMeaning,
    entities: &BTreeSet<&str>,
    relations: &BTreeSet<(&str, &str, &str)>,
) -> Result<(), Denial> {
    if definition.relations().is_empty() {
        return Err(Denial::InvalidComponentSet);
    }
    if !entities.contains(definition.member()) {
        return Err(Denial::MissingEntity);
    }
    for source in definition.relations() {
        if !relations.contains(&(source.as_str(), definition.member(), definition.member())) {
            return Err(Denial::InvalidComponentSet);
        }
    }
    Ok(())
}
