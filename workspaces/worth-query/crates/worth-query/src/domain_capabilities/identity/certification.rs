use super::scope::{
    compose_certification_sequence_digest, domain_capability_certification_scope_encoder, seal,
};
use crate::domain_capabilities::WorthQueryDomainCapabilityCategory;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

pub(crate) fn compose_certified_surface_row_digest(
    category: &str,
    ordinary_lane: &str,
    inspectable_lane: &str,
    proof_lane: &str,
    raw_lane: &str,
    implementation_path: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        domain_capability_certification_scope_encoder("domain_capability_certified_surface_row_v1")
            .field_shape(WorthQueryEvidenceTag::new("category"), category)
            .field_shape(WorthQueryEvidenceTag::new("ordinary_lane"), ordinary_lane)
            .field_shape(
                WorthQueryEvidenceTag::new("inspectable_lane"),
                inspectable_lane,
            )
            .field_shape(WorthQueryEvidenceTag::new("proof_lane"), proof_lane)
            .field_shape(WorthQueryEvidenceTag::new("raw_lane"), raw_lane)
            .field_shape(
                WorthQueryEvidenceTag::new("implementation_path"),
                implementation_path,
            ),
    )?)
}

pub(crate) fn compose_public_surface_digest(
    row_digests: impl IntoIterator<Item = String>,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "domain_capability_public_surface_v1",
        "row",
        row_digests,
    )?)
}

pub(crate) fn compose_certification_surface_digest(
    public_surface_digest: &str,
    category_count: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        domain_capability_certification_scope_encoder(
            "worth_query_domain_capability_certification_surface_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("public_surface"),
            public_surface_digest,
        )
        .field_usize(WorthQueryEvidenceTag::new("category_count"), category_count),
    )?)
}

pub(crate) fn compose_scaled_contribution_digest(
    digests: &[String],
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "domain_capability_scaled_contribution_v1",
        "contribution",
        digests,
    )?)
}

pub(crate) fn compose_scaled_trace_digest(
    digests: &[String],
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "domain_capability_scaled_trace_v1",
        "trace",
        digests,
    )?)
}

pub(crate) fn compose_scaled_support_digest(
    digests: &[String],
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "domain_capability_scaled_support_v1",
        "support",
        digests,
    )?)
}

pub(crate) fn compose_scaled_category_digest(
    categories: &[WorthQueryDomainCapabilityCategory],
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "domain_capability_scaled_category_v1",
        "category",
        categories.iter().map(|category| category.as_str()),
    )?)
}

pub(crate) fn compose_canonical_runtime_materialization_digest(
    digests: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "domain_capability_canonical_runtime_materialization_v1",
        "artifact",
        digests,
    )?)
}

pub(crate) fn compose_counter_snapshot_digest(
    contribution_width: usize,
    trace_width: usize,
    category_width: usize,
    support_width: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        domain_capability_certification_scope_encoder(
            "domain_capability_certification_counter_snapshot_v1",
        )
        .field_usize(
            WorthQueryEvidenceTag::new("contribution_width"),
            contribution_width,
        )
        .field_usize(WorthQueryEvidenceTag::new("trace_width"), trace_width)
        .field_usize(WorthQueryEvidenceTag::new("category_width"), category_width)
        .field_usize(WorthQueryEvidenceTag::new("support_width"), support_width),
    )?)
}

pub(crate) fn compose_slope_scale_entry_digest(
    label: &str,
    scale: usize,
    width: usize,
    digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        domain_capability_certification_scope_encoder("domain_capability_slope_scale_v1")
            .field_shape(WorthQueryEvidenceTag::new("label"), label)
            .field_usize(WorthQueryEvidenceTag::new("scale"), scale)
            .field_usize(WorthQueryEvidenceTag::new("width"), width)
            .field_shape(WorthQueryEvidenceTag::new("digest"), digest),
    )?)
}

pub(crate) fn compose_slope_digest(
    scale_entries: impl IntoIterator<Item = String>,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "domain_capability_slope_v1",
        "scale",
        scale_entries,
    )?)
}

fn compose_certification_bundle_output_entry_digest(
    name: &'static str,
    value: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        domain_capability_certification_scope_encoder(
            "domain_capability_certification_bundle_output_entry_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("name"), name)
        .field_shape(WorthQueryEvidenceTag::new("value"), value),
    )?)
}

pub(crate) fn compose_certification_bundle_digest(
    outputs: impl IntoIterator<Item = (&'static str, impl AsRef<str>)>,
) -> Result<String, IdentityDerivationDenial> {
    let output_entries = outputs
        .into_iter()
        .map(|(name, value)| compose_certification_bundle_output_entry_digest(name, value.as_ref()))
        .collect::<Vec<_>>();
    Ok(compose_certification_sequence_digest(
        "domain_capability_certification_bundle_v1",
        "output",
        output_entries,
    )?)
}
