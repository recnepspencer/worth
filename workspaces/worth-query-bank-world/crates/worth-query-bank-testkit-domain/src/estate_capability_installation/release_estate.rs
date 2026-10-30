pub use std::collections::BTreeSet;

pub use bank_domain::schema::ReleaseEstateOperation;
pub use worth_query_host::facade::domain::WorthQueryInstallationRuntimeIdentity;

pub use super::{
    installed_bank, installed_program_targets, installed_read_targets, InstalledProgramTarget,
    InstalledReadTarget,
};

pub fn expected_release_reads() -> BTreeSet<InstalledReadTarget> {
    [
        field("EstateCase", "EstateCaseRecord", "EstateCaseIdentityField"),
        field("EstateCase", "EstateCaseRecord", "EstateCaseStatusField"),
        field("Principal", "PrincipalIdentity", "PrincipalIdentityField"),
        field(
            "LegalAuthority",
            "LegalAuthorityRecord",
            "LegalAuthorityIdentityField",
        ),
        field(
            "LegalAuthority",
            "LegalAuthorityRecord",
            "LegalAuthorityRecognizedField",
        ),
        field(
            "MandatoryReview",
            "MandatoryReviewRecord",
            "MandatoryReviewIdentityField",
        ),
        field(
            "MandatoryReview",
            "MandatoryReviewRecord",
            "MandatoryReviewKindField",
        ),
        field(
            "MandatoryReview",
            "MandatoryReviewRecord",
            "MandatoryReviewStatusField",
        ),
        relation("EstateExecutor", "Principal", "EstateCase"),
        relation("LegalAuthorityEstate", "LegalAuthority", "EstateCase"),
        relation("LegalAuthorityHolder", "LegalAuthority", "Principal"),
        relation("ReviewEstate", "MandatoryReview", "EstateCase"),
        relation("ReviewPrincipal", "Principal", "MandatoryReview"),
    ]
    .into_iter()
    .collect()
}

pub fn field(entity: &str, aspect: &str, field: &str) -> InstalledReadTarget {
    InstalledReadTarget::Field {
        entity: entity.to_owned(),
        aspect: aspect.to_owned(),
        path: path(field),
    }
}

pub fn relation(relation: &str, from: &str, to: &str) -> InstalledReadTarget {
    InstalledReadTarget::Relation {
        relation: relation.to_owned(),
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

pub fn path(field: &str) -> worth_foundational::facade::CanonicalFieldPath {
    worth_foundational::facade::CanonicalFieldPath::single(
        worth_foundational::facade::FieldKey::new(field).unwrap(),
    )
}
