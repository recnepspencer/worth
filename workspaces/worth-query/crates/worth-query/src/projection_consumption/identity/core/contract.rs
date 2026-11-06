use super::super::scope::{consumption_scope_encoder, seal};
use super::eligibility::compose_eligibility_warning_kinds_digest;
use super::entries::{
    compose_bound_fact_family_entry_digest, compose_projection_source_reference_entry_digest,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

use super::super::super::contracts::{
    BoundProjectionFactFamily, ProjectionContractSourcePosture, ProjectionContractSupportPosture,
};
use super::super::super::eligibility::AdmittedProjectionConsumption;
use super::super::super::facts::ProjectionFactKind;
use super::super::super::source::{ProjectionConsumptionSource, ProjectionSourceFamily};
use super::super::super::support::ProjectionConsumptionSupportPosture;

pub(crate) fn compose_support_row_digest(
    source_family: ProjectionSourceFamily,
    fact_kind: ProjectionFactKind,
    posture: &ProjectionConsumptionSupportPosture,
    detail_key: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        consumption_scope_encoder("projection_consumption_support_row_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("source_family"),
                source_family.as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("fact_kind"), fact_kind.as_str())
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
            .field_shape(WorthQueryEvidenceTag::new("detail"), detail_key),
    )?)
}

pub(crate) fn compose_materialized_projection_contract_digest(
    admitted: &AdmittedProjectionConsumption,
    source: &ProjectionConsumptionSource,
    binding: &super::super::super::declaration::ProjectionConsumptionBindingContext,
    fact_families: &[BoundProjectionFactFamily],
    support_posture: &ProjectionContractSupportPosture,
    source_posture: ProjectionContractSourcePosture,
) -> Result<String, IdentityDerivationDenial> {
    let mut encoder = consumption_scope_encoder("materialized_projection_contract_v1")
        .field_shape(
            WorthQueryEvidenceTag::new("declaration"),
            admitted.declaration().declaration_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("eligibility"),
            admitted.eligibility_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_family"),
            source.family().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_posture"),
            source_posture.as_str(),
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
            WorthQueryEvidenceTag::new("narrowed_result_shape"),
            binding.narrowed_result_shape_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("authorized_projection"),
            binding.authorized_projection_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("policy"),
            binding.policy_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("tenant_schema"),
            binding.tenant_schema_basis_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("support_posture"),
            support_posture.as_str(),
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
    if let Some(posture) = source.materialized_fact_posture() {
        encoder = encoder.field_shape(
            WorthQueryEvidenceTag::new("materialized_fact_posture"),
            posture.posture_digest(),
        );
    }
    if !support_posture.warning_kinds().is_empty() {
        encoder = encoder.field_shape(
            WorthQueryEvidenceTag::new("warnings"),
            compose_eligibility_warning_kinds_digest(support_posture.warning_kinds())?,
        );
    }
    let source_references = source
        .source_reference_identities()
        .iter()
        .map(compose_projection_source_reference_entry_digest)
        .collect::<Vec<_>>();
    let fact_family_entries = fact_families
        .iter()
        .map(compose_bound_fact_family_entry_digest)
        .collect::<Vec<_>>();
    Ok(seal(
        encoder
            .field_value_sequence(
                WorthQueryEvidenceTag::new("source_reference"),
                source_references,
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("fact_family"),
                fact_family_entries,
            ),
    )?)
}
