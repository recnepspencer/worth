use super::super::scope::{consumption_scope_encoder, seal};
use super::entries::compose_fact_request_entry_digest;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

use super::super::super::declaration::ProjectionConsumptionBindingContext;
use super::super::super::facts::ProjectMaterializedFacts;
use super::super::super::source::ProjectionConsumptionSource;

pub(crate) fn compose_declaration_digest(
    source: &ProjectionConsumptionSource,
    binding: &ProjectionConsumptionBindingContext,
    requested: &ProjectMaterializedFacts,
) -> Result<String, IdentityDerivationDenial> {
    let mut encoder = consumption_scope_encoder("projection_consumption_declaration_v1")
        .field_shape(
            WorthQueryEvidenceTag::new("source_family"),
            source.family().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_identity"),
            source.source_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("result_shape"),
            binding.result_shape_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("authorized_projection"),
            binding.authorized_projection_identity(),
        );
    if let Some(query_digest) = source.query_digest() {
        encoder = encoder.field_shape(WorthQueryEvidenceTag::new("query"), query_digest);
    }
    if let Some(basis_digest) = source.basis_digest() {
        encoder = encoder.field_shape(WorthQueryEvidenceTag::new("basis"), basis_digest);
    }
    if let Some(result_digest) = source.result_digest() {
        encoder = encoder.field_shape(WorthQueryEvidenceTag::new("result"), result_digest);
    }
    let fact_requests = requested
        .requested()
        .map(|request| {
            compose_fact_request_entry_digest(request, requested.native_contract_for(request))
        })
        .collect::<Vec<_>>();
    Ok(seal(encoder.field_value_sequence(
        WorthQueryEvidenceTag::new("requested_fact"),
        fact_requests,
    ))?)
}
