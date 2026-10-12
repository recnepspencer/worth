use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};

use super::super::{
    WorthQueryApplicationEntityIdentity, WorthQueryEntityResolutionDenial,
    WorthQueryEntityResolutionDenialKind, WorthQueryPrincipalResolutionMode,
};
use super::WorthQueryEntityResolutionTruth;

pub(super) fn validate<Schema, Entity>(
    truth: &WorthQueryEntityResolutionTruth<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
) -> Result<(), WorthQueryEntityResolutionDenial> {
    if identity.runtime_authority() != truth.installed.runtime_authority
        || identity.binding_identity() != &truth.installed.binding_identity
    {
        return Err(denial(
            WorthQueryEntityResolutionDenialKind::ForeignResolutionTruth,
            identity.entity_name(),
        ));
    }
    let request = BoundedEntityFieldLookupRequest::new(
        truth.snapshot.clone(),
        identity.identity_index_id(),
        identity.entity_kind(),
        identity.identity_locator().clone(),
        identity.identity_value().clone(),
        2,
    )
    .map_err(|rejected| map_index_denial(rejected.kind(), identity.entity_name()))?;
    let lookup = truth
        .relational
        .index_access()
        .execute_bounded_entity_field_lookup(request, parity(identity.resolution_mode()), execution)
        .map_err(|rejected| map_index_denial(rejected.kind(), identity.entity_name()))?;
    if lookup.overflowed()
        || lookup.candidate_entity_ids().len() != 1
        || lookup.candidate_entity_ids()[0] != identity.entity_id()
    {
        return Err(denial(
            WorthQueryEntityResolutionDenialKind::UnknownEntity,
            identity.entity_name(),
        ));
    }
    Ok(())
}

fn parity(mode: WorthQueryPrincipalResolutionMode) -> BoundedIndexParityMode {
    match mode {
        WorthQueryPrincipalResolutionMode::Ordinary => BoundedIndexParityMode::Production,
        WorthQueryPrincipalResolutionMode::Certification => BoundedIndexParityMode::Certification,
    }
}

fn map_index_denial(
    kind: BoundedEntityFieldLookupDenialKind,
    subject: &str,
) -> WorthQueryEntityResolutionDenial {
    let kind = match kind {
        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
        | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => {
            WorthQueryEntityResolutionDenialKind::CorruptIdentityIndex
        }
        _ => WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
    };
    denial(kind, subject)
}

fn denial(
    kind: WorthQueryEntityResolutionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryEntityResolutionDenial {
    WorthQueryEntityResolutionDenial::new(kind, subject)
}
