//! Resolve original-source disclosure against native query and mutation inventories.
use std::any::TypeId;

use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBindingDescriptor, DeclaredApplicationSourceDisclosure,
    },
    application_query::{ApplicationQueryBindingDescriptor, ApplicationQueryBindingReference},
};

use super::{
    WorthQueryApplicationOperationInstallationDenial as Denial,
    WorthQueryApplicationOperationInstallationDenialKind as Kind,
};

/// Callback-free association resolved from an actual installed source binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledSourceDisclosureBinding {
    declaration: DeclaredApplicationSourceDisclosure,
    source_type: TypeId,
    query_type: TypeId,
}

impl InstalledSourceDisclosureBinding {
    pub fn declaration(&self) -> &DeclaredApplicationSourceDisclosure {
        &self.declaration
    }

    pub const fn source_type(&self) -> TypeId {
        self.source_type
    }

    pub const fn query_type(&self) -> TypeId {
        self.query_type
    }
}

pub(crate) fn resolve(
    source: &ApplicationMutationBindingDescriptor,
    queries: &[ApplicationQueryBindingDescriptor],
) -> Result<Option<InstalledSourceDisclosureBinding>, Denial> {
    let Some(declaration) = source.inspection_disclosure() else {
        return Ok(None);
    };
    let parts = declaration.parts();
    if !valid_identity(parts.identity.as_str())
        || parts.source_binding.as_str() != source.identity()
    {
        return Err(Denial::new(
            Kind::InvalidSourceDisclosureBinding,
            source.identity(),
        ));
    }
    let query = resolve_query(&parts.query, queries)
        .ok_or_else(|| Denial::new(Kind::SourceDisclosureQueryNotInstalled, source.identity()))?;
    if source.inspection_disclosure_query_type() != Some(query.query_type())
        || query.scope().field() != source.scope().field()
        || query.principal().name() != source.principal().name()
        || query.principal().principal_identity_binding()
            != source.principal().principal_identity_binding()
    {
        return Err(Denial::new(
            Kind::SourceDisclosureMeaningChanged,
            source.identity(),
        ));
    }
    Ok(Some(InstalledSourceDisclosureBinding {
        declaration: declaration.clone(),
        source_type: source.binding_type(),
        query_type: query.query_type(),
    }))
}

pub(crate) fn resolve_query<'a>(
    declaration: &ApplicationQueryBindingReference,
    queries: &'a [ApplicationQueryBindingDescriptor],
) -> Option<&'a ApplicationQueryBindingDescriptor> {
    queries.iter().find(|query| {
        query.identity() == declaration.parts().binding.as_str()
            && declaration.matches_descriptor(query)
    })
}

fn valid_identity(identity: &str) -> bool {
    !identity.is_empty()
        && identity.trim() == identity
        && !identity
            .chars()
            .any(|value| value.is_control() || value.is_whitespace())
}
