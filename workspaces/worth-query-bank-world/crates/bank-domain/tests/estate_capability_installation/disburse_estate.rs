use std::collections::BTreeSet;

use bank_domain::schema::DisburseEstateOperation;
use worth_query_host::facade::domain::WorthQueryInstallationRuntimeIdentity;

use super::{
    installed_bank, installed_program_targets, installed_read_targets, InstalledProgramTarget,
    InstalledReadTarget,
};

#[test]
fn disbursement_installs_exact_effect_integrity_reads_and_money_program() {
    let (_index, bank) = installed_bank(WorthQueryInstallationRuntimeIdentity::fresh());
    let operation = bank
        .installed_operation(DisburseEstateOperation::reference())
        .expect("estate disbursement must install one executable accounting program");

    assert_eq!(operation.contracts().projection_work_budget(), 192);
    assert_eq!(
        installed_read_targets(operation.contracts()),
        expected_reads()
    );
    assert_eq!(
        installed_program_targets(operation.contracts()),
        expected_program()
    );
}

fn expected_reads() -> BTreeSet<InstalledReadTarget> {
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

fn expected_program() -> BTreeSet<InstalledProgramTarget> {
    [
        InstalledProgramTarget::Create("JournalEntry".to_owned()),
        InstalledProgramTarget::Create("Posting".to_owned()),
        write("Account", "AccountState", "AccountingRevision"),
        write("JournalEntry", "JournalIdentity", "JournalIdentityField"),
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

fn write(entity: &str, aspect: &str, field: &str) -> InstalledProgramTarget {
    InstalledProgramTarget::Write {
        entity: entity.to_owned(),
        aspect: aspect.to_owned(),
        path: path(field),
    }
}

fn path(field: &str) -> worth_foundational::facade::CanonicalFieldPath {
    worth_foundational::facade::CanonicalFieldPath::single(
        worth_foundational::facade::FieldKey::new(field).unwrap(),
    )
}
