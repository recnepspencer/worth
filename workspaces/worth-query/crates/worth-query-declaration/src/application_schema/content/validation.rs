//! Pure declaration closure. A format descriptor supplies no executable authority.
use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::ScalarAspectType;

pub(in crate::application_schema) fn validate(members: &[Member]) -> Result<(), Denial> {
    let mut formats = BTreeMap::new();
    let mut names = BTreeSet::new();
    for member in members {
        if let Member::ContentFormat { declaration } = member
            && (formats.insert(declaration.identity, declaration).is_some()
                || !names.insert((&declaration.name, declaration.version)))
        {
            return Err(Denial::DuplicateContentFormat);
        }
    }
    for member in members {
        if let Member::Field {
            scalar_family,
            content_format: Some(identity),
            canonical,
            ..
        } = member
        {
            if *scalar_family != ScalarAspectType::ContentRef || canonical.is_some() {
                return Err(Denial::InvalidContentField);
            }
            if !formats.contains_key(identity) {
                return Err(Denial::MissingContentFormat);
            }
        }
    }
    Ok(())
}
