pub use std::collections::BTreeSet;

pub use bank_domain::schema::DisburseEstateOperation;
pub use worth_query_host::facade::domain::WorthQueryInstallationRuntimeIdentity;

pub use super::{
    installed_bank, installed_program_targets, installed_read_targets, InstalledProgramTarget,
    InstalledReadTarget,
};

pub fn expected_reads() -> BTreeSet<InstalledReadTarget> {
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
        field("Account", "Identity", "AccountIdentity"),
        field("Account", "AccountState", "AccountingRevision"),
        field("Account", "AccountState", "Status"),
        field("Account", "AccountProfile", "Kind"),
        field("Account", "AccountProfile", "AccountDisplayName"),
        field(
            "Institution",
            "InstitutionIdentity",
            "InstitutionIdentityField",
        ),
        field("Business", "BusinessIdentity", "BusinessIdentityField"),
        field("Posting", "PostingValue", "PostingAmount"),
        relation("EstateAccount", "EstateCase", "Account"),
        relation("EstateBeneficiary", "Principal", "EstateCase"),
        relation("EstateJointOwner", "Principal", "Account"),
        relation("LegalAuthorityEstate", "LegalAuthority", "EstateCase"),
        relation("LegalAuthorityHolder", "LegalAuthority", "Principal"),
        relation("EstateExecutor", "Principal", "EstateCase"),
        relation("InstitutionAccount", "Institution", "Account"),
        relation("PersonalOwner", "Principal", "Account"),
        relation("BusinessAccount", "Business", "Account"),
        relation("PostingAccount", "Posting", "Account"),
    ]
    .into_iter()
    .collect()
}

pub fn expected_program() -> BTreeSet<InstalledProgramTarget> {
    [
        InstalledProgramTarget::Create("JournalEntry".to_owned()),
        InstalledProgramTarget::Create("Posting".to_owned()),
        write("Account", "AccountState", "AccountingRevision"),
        write("JournalEntry", "JournalIdentity", "JournalIdentityField"),
        write(
            "JournalEntry",
            "JournalIdentity",
            "JournalInstitutionContext",
        ),
        write("JournalEntry", "JournalState", "JournalPurpose"),
        write("Posting", "PostingIdentity", "PostingIdentityField"),
        write("Posting", "PostingValue", "PostingAmount"),
        write("Posting", "PostingValue", "PostingAccountSequence"),
        write("Posting", "PostingValue", "Purpose"),
        InstalledProgramTarget::Link {
            relation: "JournalPosting".to_owned(),
            from: "JournalEntry".to_owned(),
            to: "Posting".to_owned(),
        },
        InstalledProgramTarget::Link {
            relation: "PostingAccount".to_owned(),
            from: "Posting".to_owned(),
            to: "Account".to_owned(),
        },
        InstalledProgramTarget::Emit("AccountActivityEffect".to_owned()),
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

pub fn write(entity: &str, aspect: &str, field: &str) -> InstalledProgramTarget {
    InstalledProgramTarget::Write {
        entity: entity.to_owned(),
        aspect: aspect.to_owned(),
        path: path(field),
    }
}

pub fn path(field: &str) -> worth_foundational::facade::CanonicalFieldPath {
    worth_foundational::facade::CanonicalFieldPath::single(
        worth_foundational::facade::FieldKey::new(field).unwrap(),
    )
}
