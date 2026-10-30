use std::cmp::Ordering;

use super::normalized_transition::CanonicalSchemaDiffDetail;

pub(super) fn compare_detail_canonically(
    left: &CanonicalSchemaDiffDetail<'_>,
    right: &CanonicalSchemaDiffDetail<'_>,
) -> Ordering {
    detail_sort_key(left)
        .cmp(&detail_sort_key(right))
        .then_with(|| compare_detail_terms(left, right))
}

pub(super) fn detail_sort_key(detail: &CanonicalSchemaDiffDetail<'_>) -> u8 {
    match detail {
        CanonicalSchemaDiffDetail::AddedField { .. } => 1,
        CanonicalSchemaDiffDetail::RemovedField { .. } => 2,
        CanonicalSchemaDiffDetail::TypeChanged { .. } => 3,
        CanonicalSchemaDiffDetail::EnumDomainExpanded { .. } => 4,
        CanonicalSchemaDiffDetail::InvariantContractChanged { .. } => 5,
        CanonicalSchemaDiffDetail::ProjectionContractChanged { .. } => 6,
        CanonicalSchemaDiffDetail::SubscriberContractChanged { .. } => 7,
        CanonicalSchemaDiffDetail::FreeText { .. } => 8,
    }
}

fn compare_detail_terms(
    left: &CanonicalSchemaDiffDetail<'_>,
    right: &CanonicalSchemaDiffDetail<'_>,
) -> Ordering {
    match (left, right) {
        (
            CanonicalSchemaDiffDetail::AddedField {
                field: lf,
                required: lr,
                default_expression: ld,
            },
            CanonicalSchemaDiffDetail::AddedField {
                field: rf,
                required: rr,
                default_expression: rd,
            },
        ) => lf.cmp(rf).then_with(|| lr.cmp(rr)).then_with(|| ld.cmp(rd)),
        (
            CanonicalSchemaDiffDetail::RemovedField { field: lf },
            CanonicalSchemaDiffDetail::RemovedField { field: rf },
        ) => lf.cmp(rf),
        (
            CanonicalSchemaDiffDetail::TypeChanged {
                field: lf,
                from_type: lfrom,
                to_type: lto,
            },
            CanonicalSchemaDiffDetail::TypeChanged {
                field: rf,
                from_type: rfrom,
                to_type: rto,
            },
        ) => lf
            .cmp(rf)
            .then_with(|| lfrom.cmp(rfrom))
            .then_with(|| lto.cmp(rto)),
        (
            CanonicalSchemaDiffDetail::EnumDomainExpanded {
                field: lf,
                added_variants: lv,
            },
            CanonicalSchemaDiffDetail::EnumDomainExpanded {
                field: rf,
                added_variants: rv,
            },
        ) => lf.cmp(rf).then_with(|| lv.cmp(rv)),
        (
            CanonicalSchemaDiffDetail::InvariantContractChanged { contract_name: lf },
            CanonicalSchemaDiffDetail::InvariantContractChanged { contract_name: rf },
        ) => lf.cmp(rf),
        (
            CanonicalSchemaDiffDetail::ProjectionContractChanged {
                projection_name: lf,
            },
            CanonicalSchemaDiffDetail::ProjectionContractChanged {
                projection_name: rf,
            },
        ) => lf.cmp(rf),
        (
            CanonicalSchemaDiffDetail::SubscriberContractChanged { contract_name: lf },
            CanonicalSchemaDiffDetail::SubscriberContractChanged { contract_name: rf },
        ) => lf.cmp(rf),
        (
            CanonicalSchemaDiffDetail::FreeText {
                detail: lf,
                declared_intent: li,
            },
            CanonicalSchemaDiffDetail::FreeText {
                detail: rf,
                declared_intent: ri,
            },
        ) => lf.cmp(rf).then_with(|| li.cmp(ri)),
        _ => Ordering::Equal,
    }
}
