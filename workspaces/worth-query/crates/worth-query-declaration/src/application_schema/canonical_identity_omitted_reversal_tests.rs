//! Omitted-reversal grants are canonical schema meaning: adding or changing a
//! grant changes the digest, and a schema without grants keeps its digest.
use super::identity;
use crate::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredCorrectionAuthority, DeclaredLoweringCorrespondenceRef,
    PortableApplicationAftermathContract, PortableCorrectionMechanism, PortablePreImageDemand,
    PortablePreImageLocus, PortableRecordedInverse,
};
use crate::application_schema::ApplicationSchemaMember;
use crate::archive_decode_authority::ARCHIVE_DECODE;

fn locus(field: &str) -> PortablePreImageLocus {
    PortablePreImageLocus::from_untrusted_fields(
        ARCHIVE_DECODE,
        "Task".into(),
        "TaskFacts".into(),
        field.into(),
    )
}

fn aftermath(omitted: &[&str]) -> ApplicationSchemaMember {
    let demand =
        PortablePreImageDemand::from_untrusted_fields(ARCHIVE_DECODE, vec![locus("title")], 64)
            .with_untrusted_omitted_reversals(
                ARCHIVE_DECODE,
                omitted.iter().map(|field| locus(field)).collect(),
            );
    ApplicationSchemaMember::OperationAftermath {
        operation: "EditTask".into(),
        contract: PortableApplicationAftermathContract::from_untrusted_fields(
            ARCHIVE_DECODE,
            DeclaredCorrectionAuthority::RuntimeAlone,
            Some(PortableCorrectionMechanism::RecordedInverse(
                PortableRecordedInverse::from_untrusted_fields(
                    ARCHIVE_DECODE,
                    "RestoreTask".into(),
                    DeclaredLoweringCorrespondenceRef::new("task-edit.v1"),
                    DeclaredAftermathPostcondition::ExactPriorTruth,
                    demand,
                ),
            )),
            None,
        ),
    }
}

#[test]
fn an_omitted_reversal_grant_changes_the_canonical_schema_identity() {
    let none = identity(&[aftermath(&[])]);
    let revision = identity(&[aftermath(&["revision"])]);
    let audit = identity(&[aftermath(&["audit"])]);
    assert_ne!(none, revision, "adding a grant changes identity");
    assert_ne!(revision, audit, "changing a grant changes identity");
    assert_eq!(
        none,
        identity(&[aftermath(&[])]),
        "no grant keeps identity stable"
    );
}
