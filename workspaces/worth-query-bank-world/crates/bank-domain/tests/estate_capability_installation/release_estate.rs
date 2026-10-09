use std::collections::BTreeSet;

use bank_domain::schema::ReleaseEstateOperation;
use worth_query_host::facade::domain::WorthQueryInstallationRuntimeIdentity;

use super::{
    installed_bank, installed_program_targets, installed_read_targets, InstalledProgramTarget,
    InstalledReadTarget,
};

#[test]
fn release_installs_exact_reads_and_effect() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let release = bank
        .installed_operation(ReleaseEstateOperation::reference())
        .expect("release must install one exact executable program");
    assert_eq!(release.contracts().projection_work_budget(), 96);
    assert_eq!(
        installed_read_targets(release.contracts()),
        expected_release_reads()
    );
    assert_eq!(
        installed_program_targets(release.contracts()),
        [InstalledProgramTarget::Write {
            entity: "EstateCase".to_owned(),
            aspect: "EstateCaseRecord".to_owned(),
            path: path("EstateCaseStatusField"),
        }]
        .into_iter()
        .collect()
    );
}

fn expected_release_reads() -> BTreeSet<InstalledReadTarget> {
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

fn field(entity: &str, aspect: &str, field: &str) -> InstalledReadTarget {
    InstalledReadTarget::Field {
        entity: entity.to_owned(),
        aspect: aspect.to_owned(),
        path: path(field),
    }
}

fn relation(relation: &str, from: &str, to: &str) -> InstalledReadTarget {
    InstalledReadTarget::Relation {
        relation: relation.to_owned(),
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

fn path(field: &str) -> worth_foundational::facade::CanonicalFieldPath {
    worth_foundational::facade::CanonicalFieldPath::single(
        worth_foundational::facade::FieldKey::new(field).unwrap(),
    )
}
