use std::cmp::Ordering;

use worth_foundational::FieldKey;

use crate::schema::data::{SchemaDiffAtom, SchemaDiffDetail, SchemaStratum};

use super::detail_ordering::compare_detail_canonically;

#[derive(Debug)]
pub(super) struct NormalizedTransitionView<'a> {
    pub(super) canonical_atoms: Vec<CanonicalSchemaDiffAtom<'a>>,
}

#[derive(Debug, Clone)]
pub(super) struct CanonicalSchemaDiffAtom<'a> {
    pub(super) atom: &'a SchemaDiffAtom,
    pub(super) element_name_sort_key: u64,
    pub(super) normalized_strata: Vec<SchemaStratum>,
    pub(super) normalized_detail: CanonicalSchemaDiffDetail<'a>,
}

#[derive(Debug, Clone)]
pub(super) enum CanonicalSchemaDiffDetail<'a> {
    AddedField {
        field: &'a FieldKey,
        required: bool,
        default_expression: Option<&'a str>,
    },
    RemovedField {
        field: &'a FieldKey,
    },
    TypeChanged {
        field: &'a FieldKey,
        from_type: &'a str,
        to_type: &'a str,
    },
    EnumDomainExpanded {
        field: &'a FieldKey,
        added_variants: Vec<&'a str>,
    },
    InvariantContractChanged {
        contract_name: &'a str,
    },
    ProjectionContractChanged {
        projection_name: &'a str,
    },
    SubscriberContractChanged {
        contract_name: &'a str,
    },
    FreeText {
        detail: &'a str,
        declared_intent: crate::schema::data::FreeFormSchemaDiffIntent,
    },
}

pub(super) fn normalize_transition(diff_atoms: &[SchemaDiffAtom]) -> NormalizedTransitionView<'_> {
    let mut canonical_atoms = diff_atoms
        .iter()
        .map(CanonicalSchemaDiffAtom::new)
        .collect::<Vec<_>>();
    canonical_atoms.sort_unstable_by(compare_atoms_canonically);
    NormalizedTransitionView { canonical_atoms }
}

impl<'a> CanonicalSchemaDiffAtom<'a> {
    fn new(atom: &'a SchemaDiffAtom) -> Self {
        let mut normalized_strata = atom.strata.clone();
        normalized_strata.sort_unstable();
        normalized_strata.dedup();
        Self {
            atom,
            element_name_sort_key: non_authority_sort_key(atom.element.element_name.as_bytes()),
            normalized_detail: CanonicalSchemaDiffDetail::new(&atom.detail),
            normalized_strata,
        }
    }
}

impl<'a> CanonicalSchemaDiffDetail<'a> {
    fn new(detail: &'a SchemaDiffDetail) -> Self {
        match detail {
            SchemaDiffDetail::AddedField {
                field,
                required,
                default_expression,
            } => Self::AddedField {
                field,
                required: *required,
                default_expression: default_expression.as_deref(),
            },
            SchemaDiffDetail::RemovedField { field } => Self::RemovedField { field },
            SchemaDiffDetail::TypeChanged {
                field,
                from_type,
                to_type,
            } => Self::TypeChanged {
                field,
                from_type: from_type.as_ref(),
                to_type: to_type.as_ref(),
            },
            SchemaDiffDetail::EnumDomainExpanded {
                field,
                added_variants,
            } => {
                let mut normalized_variants = added_variants
                    .iter()
                    .map(|variant| variant.as_ref())
                    .collect::<Vec<_>>();
                normalized_variants.sort_unstable();
                normalized_variants.dedup();
                Self::EnumDomainExpanded {
                    field,
                    added_variants: normalized_variants,
                }
            }
            SchemaDiffDetail::InvariantContractChanged { contract_name } => {
                Self::InvariantContractChanged {
                    contract_name: contract_name.as_ref(),
                }
            }
            SchemaDiffDetail::ProjectionContractChanged { projection_name } => {
                Self::ProjectionContractChanged {
                    projection_name: projection_name.as_ref(),
                }
            }
            SchemaDiffDetail::SubscriberContractChanged { contract_name } => {
                Self::SubscriberContractChanged {
                    contract_name: contract_name.as_ref(),
                }
            }
            SchemaDiffDetail::FreeText {
                detail,
                declared_intent,
            } => Self::FreeText {
                detail: detail.as_ref(),
                declared_intent: *declared_intent,
            },
        }
    }
}

fn compare_atoms_canonically(
    left: &CanonicalSchemaDiffAtom<'_>,
    right: &CanonicalSchemaDiffAtom<'_>,
) -> Ordering {
    left.atom
        .element
        .schema_id
        .0
        .cmp(&right.atom.element.schema_id.0)
        .then_with(|| {
            left.atom
                .element
                .schema_version_id
                .cmp(&right.atom.element.schema_version_id)
        })
        .then_with(|| left.atom.element.kind.cmp(&right.atom.element.kind))
        .then_with(|| left.atom.element.kind_id.cmp(&right.atom.element.kind_id))
        .then_with(|| left.element_name_sort_key.cmp(&right.element_name_sort_key))
        .then_with(|| {
            left.atom
                .element
                .element_name
                .cmp(&right.atom.element.element_name)
        })
        .then_with(|| left.normalized_strata.cmp(&right.normalized_strata))
        .then_with(|| {
            left.atom
                .publication_impact
                .cmp(&right.atom.publication_impact)
        })
        .then_with(|| {
            left.atom
                .subscriber_impact
                .cmp(&right.atom.subscriber_impact)
        })
        .then_with(|| {
            left.atom
                .historical_interpretation
                .cmp(&right.atom.historical_interpretation)
        })
        .then_with(|| compare_detail_canonically(&left.normalized_detail, &right.normalized_detail))
}

fn non_authority_sort_key(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
