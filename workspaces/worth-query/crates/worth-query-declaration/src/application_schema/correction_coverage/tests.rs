//! A half-undo (a source-written field the pre-image or inverse omits) fails
//! closed; full coverage or an explicit typed grant admits.
use super::*;
use crate::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredCorrectionAuthority, DeclaredLoweringCorrespondenceRef,
    DeclaredPreImageDemandDenial, PortableApplicationAftermathContract, PortablePreImageDemand,
    PortablePreImageFieldOutput, PortableRecordedInverse, PreImageOutputFieldCapture,
};
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationOperationBodyProgram,
    ApplicationOperationBodyScopeField,
};
use crate::archive_decode_authority::ARCHIVE_DECODE;
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::ScalarAspectType;

fn assign(field: &str) -> ApplicationOperationBodyScopeField {
    ApplicationOperationBodyScopeField::AssignOptionalInput {
        field: ApplicationFieldBindingLocus::new("Task", "TaskFacts", field),
        binding: WorthQueryPortableTypeIdentity::from_untrusted(ARCHIVE_DECODE, "codec".into()),
        scalar_family: ScalarAspectType::String,
        member: field.to_owned(),
    }
}

fn locus(field: &str) -> PortablePreImageLocus {
    PortablePreImageLocus::from_untrusted_fields(
        ARCHIVE_DECODE,
        "Task".into(),
        "TaskFacts".into(),
        field.into(),
    )
}

fn body(operation: &str, fields: &[&str]) -> ApplicationSchemaMember {
    ApplicationSchemaMember::OperationBody {
        operation: operation.to_owned(),
        body: ApplicationOperationBodyProgram::from_untrusted_in_scope(
            ARCHIVE_DECODE,
            "Task".into(),
            fields.iter().map(|field| assign(field)).collect(),
            None,
            vec![],
        ),
    }
}

/// `EditTask` writes title and due; `RestoreTask` writes `restored`.
fn members(retained: &[&str], restored: &[&str], omitted: &[&str]) -> Vec<ApplicationSchemaMember> {
    let demand = PortablePreImageDemand::from_untrusted_fields(
        ARCHIVE_DECODE,
        retained.iter().map(|field| locus(field)).collect(),
        64,
    )
    .with_untrusted_omitted_reversals(
        ARCHIVE_DECODE,
        omitted.iter().map(|field| locus(field)).collect(),
    );
    with_demand(demand, restored)
}

fn with_demand(demand: PortablePreImageDemand, restored: &[&str]) -> Vec<ApplicationSchemaMember> {
    let inverse = PortableRecordedInverse::from_untrusted_fields(
        ARCHIVE_DECODE,
        "RestoreTask".into(),
        DeclaredLoweringCorrespondenceRef::new("correct_task_edit"),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        demand,
    );
    vec![
        body("EditTask", &["title", "due"]),
        body("RestoreTask", restored),
        ApplicationSchemaMember::OperationAftermath {
            operation: "EditTask".into(),
            contract: PortableApplicationAftermathContract::from_untrusted_fields(
                ARCHIVE_DECODE,
                DeclaredCorrectionAuthority::RuntimeAlone,
                Some(PortableCorrectionMechanism::RecordedInverse(inverse)),
                None,
            ),
        },
    ]
}

fn gap(members: &[ApplicationSchemaMember]) -> Option<(String, ApplicationCorrectionCoverageGap)> {
    match validate(members) {
        Ok(()) => None,
        Err(ApplicationSchemaDeclarationDenial::IncompleteCorrectionCoverage {
            operation,
            field,
            gap,
            ..
        }) => {
            assert_eq!(operation, "EditTask");
            Some((field, gap))
        }
        Err(other) => panic!("unexpected denial {other:?}"),
    }
}

#[test]
fn a_half_undo_fails_closed_unless_covered_or_explicitly_granted() {
    use ApplicationCorrectionCoverageGap as Gap;
    // Daymark's Phase 6 shape: the edit writes `due`, the correction omits it.
    assert_eq!(
        gap(&members(&["title"], &["title"], &[])),
        Some(("due".into(), Gap::PreImage))
    );
    assert_eq!(
        gap(&members(&["title", "due"], &["title"], &[])),
        Some(("due".into(), Gap::InverseWrites))
    );
    // (a) full undo: retained and restored.
    assert_eq!(
        gap(&members(&["title", "due"], &["title", "due"], &[])),
        None
    );
    // (b) an explicit typed grant for exactly the omitted field.
    assert_eq!(gap(&members(&["title"], &["title"], &["due"])), None);
    // A grant must name a field the source writes.
    assert_eq!(
        gap(&members(&["title", "due"], &["title", "due"], &["notes"])),
        Some(("notes".into(), Gap::UnwrittenOmission))
    );
}

fn demand_denial(members: &[ApplicationSchemaMember]) -> DeclaredPreImageDemandDenial {
    match validate(members) {
        Err(ApplicationSchemaDeclarationDenial::InvalidCorrectionDemand { correction, cause }) => {
            assert_eq!(correction, "EditTask");
            cause
        }
        other => panic!("expected an invalid demand, got {other:?}"),
    }
}

#[test]
fn readmitted_omission_grants_are_validated_on_every_recorded_inverse() {
    use DeclaredPreImageDemandDenial as Denial;
    let grant = |loci: Vec<PortablePreImageLocus>| {
        PortablePreImageDemand::from_untrusted_fields(ARCHIVE_DECODE, vec![locus("title")], 64)
            .with_untrusted_omitted_reversals(ARCHIVE_DECODE, loci)
    };
    let readmit = |loci| with_demand(grant(loci), &["title"]);
    assert_eq!(
        demand_denial(&readmit(vec![locus("title")])),
        Denial::ContradictoryOmittedReversal
    );
    assert_eq!(
        demand_denial(&readmit(vec![locus("due"), locus("due")])),
        Denial::DuplicateLocus
    );
    assert_eq!(
        demand_denial(&readmit(vec![locus(" ")])),
        Denial::EmptyLocusAxis
    );
    // A handler source (no body) cannot carry a grant.
    let mut handler = readmit(vec![locus("due")]);
    handler.retain(|member| {
        !matches!(member, ApplicationSchemaMember::OperationBody { operation, .. } if operation == "EditTask")
    });
    assert_eq!(demand_denial(&handler), Denial::UnsupportedOmittedReversal);
    // Nor can an output-role capture.
    let output = grant(vec![locus("due")]).with_untrusted_output_fields(
        PortablePreImageFieldOutput::from_untrusted_fields(
            ARCHIVE_DECODE,
            "task".into(),
            "Task".into(),
            PreImageOutputFieldCapture::ChangedPreserved,
        ),
    );
    assert_eq!(
        demand_denial(&with_demand(output, &["title"])),
        Denial::UnsupportedOmittedReversal
    );
}
