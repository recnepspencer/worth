use worth_foundational::canonicalization_api::lower_lane::basis::{
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisValue,
};
use worth_foundational::InternedString;
use worth_store_contracts::PhysicalAuthorityScope;

use crate::StorePhysicalBoundaryWitness;

const ASPECT_BOUNDARY_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("store.aspect.boundary.fact");
const ASPECT_PATCH_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("store.aspect.patch.boundary.fact");
const ASPECT_BOUNDARY_FIELD_KIND: CanonicalBasisEntryKind =
    CanonicalBasisEntryKind::Future("store-aspect-boundary-field");
const ASPECT_PATCH_FIELD_KIND: CanonicalBasisEntryKind =
    CanonicalBasisEntryKind::Future("store-aspect-patch-field");
const PHYSICAL_BOUNDARY_WITNESS_KIND: CanonicalBasisEntryKind =
    CanonicalBasisEntryKind::Future("store-physical-boundary-witness");

pub fn aspect_boundary_entries(
    foundational_entries: &[CanonicalBasisEntry],
    boundary_witness: StorePhysicalBoundaryWitness,
) -> Vec<CanonicalBasisEntry> {
    store_boundary_entries(
        ASPECT_BOUNDARY_DOMAIN,
        ASPECT_BOUNDARY_FIELD_KIND,
        "foundational.aspect-state",
        "foundational-aspect-state",
        foundational_entries,
        boundary_witness,
    )
}

pub fn aspect_patch_entries(
    foundational_entries: &[CanonicalBasisEntry],
    boundary_witness: StorePhysicalBoundaryWitness,
) -> Vec<CanonicalBasisEntry> {
    store_boundary_entries(
        ASPECT_PATCH_DOMAIN,
        ASPECT_PATCH_FIELD_KIND,
        "foundational.aspect-patch",
        "foundational-aspect-patch",
        foundational_entries,
        boundary_witness,
    )
}

fn store_boundary_entries(
    domain: CanonicalBasisDomain,
    store_field_kind: CanonicalBasisEntryKind,
    foundational_locus_prefix: &'static str,
    source_kind: &'static str,
    foundational_entries: &[CanonicalBasisEntry],
    boundary_witness: StorePhysicalBoundaryWitness,
) -> Vec<CanonicalBasisEntry> {
    let mut entries = Vec::with_capacity(foundational_entries.len() + 4);

    entries.push(text_entry(
        domain,
        store_field_kind,
        "source.kind",
        source_kind,
    ));
    entries.extend(
        foundational_entries
            .iter()
            .enumerate()
            .map(|(ordinal, entry)| {
                let locus = namespaced_locus(foundational_locus_prefix, ordinal, entry.locus());
                CanonicalBasisEntry::new(
                    domain,
                    CanonicalBasisLocus::Named(locus),
                    entry.kind(),
                    entry.value().clone(),
                )
            }),
    );
    append_boundary_witness_entries(domain, &mut entries, boundary_witness);

    entries
}

fn append_boundary_witness_entries(
    domain: CanonicalBasisDomain,
    entries: &mut Vec<CanonicalBasisEntry>,
    boundary_witness: StorePhysicalBoundaryWitness,
) {
    entries.push(text_entry(
        domain,
        PHYSICAL_BOUNDARY_WITNESS_KIND,
        "physical.boundary.authority.scope",
        authority_scope_token(boundary_witness.authority().authority_scope()),
    ));
    entries.push(text_entry(
        domain,
        PHYSICAL_BOUNDARY_WITNESS_KIND,
        "physical.boundary.roadmap",
        boundary_witness.authority().roadmap_scope().roadmap(),
    ));
    entries.push(text_entry(
        domain,
        PHYSICAL_BOUNDARY_WITNESS_KIND,
        "physical.boundary.sequence",
        boundary_witness.authority().roadmap_scope().sequence(),
    ));
}

fn namespaced_locus(
    prefix: &'static str,
    ordinal: usize,
    source_locus: &CanonicalBasisLocus,
) -> InternedString {
    format!("{prefix}.{ordinal:04}.{}", locus_token(source_locus)).into()
}

fn locus_token(locus: &CanonicalBasisLocus) -> String {
    match locus {
        CanonicalBasisLocus::Root => "root".to_owned(),
        CanonicalBasisLocus::EntryOrdinal(ordinal) => format!("ordinal.{ordinal}"),
        CanonicalBasisLocus::Aspect(aspect) => format!("aspect.{}", aspect.as_str()),
        CanonicalBasisLocus::AspectField { aspect, path } => format!(
            "aspect-field.{}.{}",
            aspect.as_str(),
            field_path_token(path)
        ),
        CanonicalBasisLocus::Named(name) => interned_token("named", name),
    }
}

fn field_path_token(path: &worth_foundational::CanonicalFieldPath) -> String {
    path.fields()
        .iter()
        .map(|field| field.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

fn interned_token(prefix: &'static str, value: &InternedString) -> String {
    match value {
        InternedString::Raw(raw) => format!("{prefix}.{raw}"),
        InternedString::Stored(raw) => format!("{prefix}.{}", raw.as_str()),
        InternedString::Symbol(symbol) => format!("{prefix}.symbol.{}", symbol.0),
    }
}

fn text_entry(
    domain: CanonicalBasisDomain,
    kind: CanonicalBasisEntryKind,
    locus: impl Into<InternedString>,
    value: impl Into<InternedString>,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        domain,
        CanonicalBasisLocus::Named(locus.into()),
        kind,
        CanonicalBasisValue::ExactText(value.into()),
    )
}

fn authority_scope_token(scope: PhysicalAuthorityScope) -> &'static str {
    match scope {
        PhysicalAuthorityScope::AspectNativeBoundaryVocabulary => "aspect-native-boundary",
        PhysicalAuthorityScope::PhysicalFoundationVocabulary => "physical-foundation",
        PhysicalAuthorityScope::PhysicalEvidenceExport => "physical-evidence-export",
        PhysicalAuthorityScope::PhysicalSubstrateReadiness => "physical-substrate-readiness",
    }
}

#[cfg(test)]
mod textual_basis_tests {
    use super::*;

    #[test]
    fn decoded_stored_text_preserves_the_established_basis_token() {
        let raw = InternedString::Raw("same token".into());
        let restored: InternedString =
            serde_json::from_value(serde_json::to_value(&raw).unwrap()).unwrap();
        assert!(matches!(&restored, InternedString::Stored(_)));
        assert_eq!(interned_token("named", &raw), "named.same token");
        assert_eq!(
            interned_token("named", &restored),
            interned_token("named", &raw)
        );
    }
}
