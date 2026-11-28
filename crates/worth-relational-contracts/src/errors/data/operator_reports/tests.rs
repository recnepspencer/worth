use super::*;
use crate::errors::data::RelationalError;
use crate::lineage::data::naming_denial::LineageNameDenial;
use crate::publication::data::publication_error::{
    PublicationError, PublicationErrorOwnerPort as _,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::report::{DiagnosticDetail, ReportChainEnd, ReportDetail, ReportProjection};

#[test]
fn report_preserves_typed_causes_without_disclosing_authored_values() {
    fn error(scheme: &str) -> RelationalError {
        TransactionCommitError::publication(PublicationError::from_lineage_name_denial(
            LineageNameDenial::GroupNaming {
                scheme_id: scheme.into(),
                denial: crate::aggregate_identity::transition::group::denial::AggregateGroupNamingDenial::EmptyGroup,
            },
        )).into()
    }

    let left = error("private-scheme-a");
    let right = error("private-scheme-b");
    let report = left.to_operator_report(DiagnosticDetail::Forensic);
    assert_eq!(report, right.to_operator_report(DiagnosticDetail::Forensic));
    assert_eq!(report.denial_class(), DenialClass::InvalidInput);
    assert_eq!(
        report.report_code().as_str(),
        "relational.commit.publication"
    );
    assert_eq!(report.chain_end(), ReportChainEnd::Complete);
    assert_eq!(report.chain().len(), 4);
    assert!(report
        .chain()
        .iter()
        .all(|frame| frame.detail() == &ReportDetail::Unavailable));
    assert!(!report.to_canonical_json().contains("private-scheme"));
    let lineage = left.source().unwrap().source().unwrap().source().unwrap();
    assert!(lineage.is::<LineageNameDenial>());
}

#[test]
fn oversized_canonical_artifact_reports_exhaustion_and_preserves_its_cause() {
    use crate::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Encode;
    use crate::errors::data::{ErrorContext, ErrorOperation, RelationalSubsystem};
    use crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial;
    use crate::transactions::data::commit_log::CommitLog;

    let error = RelationalError::from(TransactionCommitError::PublicationDenied {
        denial: RelationalPublicationDenial::CanonicalArtifactEncoding(Encode::SizeLimit {
            observed: 11,
            maximum: 10,
        }),
        context: ErrorContext::new(RelationalSubsystem::Publication, ErrorOperation::Publish),
        commit_log: CommitLog::new(),
    });
    let report = error.to_operator_report(DiagnosticDetail::Standard);
    assert_eq!(report.denial_class(), DenialClass::Exhausted);
    assert_eq!(report.disposition(), Disposition::CorrectInput);
    assert_eq!(report.chain().len(), 3);
    let encoding = error.source().unwrap().source().unwrap().source().unwrap();
    assert!(matches!(
        encoding.downcast_ref::<Encode>(),
        Some(Encode::SizeLimit {
            observed: 11,
            maximum: 10
        })
    ));
}

#[test]
fn preparation_report_preserves_cold_storage_failure() {
    use crate::history::data::coordinates::BranchId;
    use crate::identity::data::{PartitionId, VersionId};
    use crate::storage::data::storage_unavailable::{
        RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
    };
    use crate::transactions::data::outcomes::commit_preparation::{
        CommitPreparationError, CommitPreparationErrorOwnerPort,
    };

    let cause = RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Region(PartitionId(7)),
        cause: RelationalStorageUnavailableCause::MissingMember,
    };
    let error = TransactionCommitError::preparation(
        <CommitPreparationError as CommitPreparationErrorOwnerPort>::storage_unavailable(
            BranchId("main".into()),
            None,
            VersionId(1),
            cause,
        ),
    );
    assert_eq!(
        error
            .to_operator_report(DiagnosticDetail::Standard)
            .chain()
            .len(),
        2,
    );
    assert_eq!(
        error.source().unwrap().source().unwrap().downcast_ref(),
        Some(&cause),
    );
}

#[test]
fn invariant_conflict_report_preserves_evaluation_failure_and_refusal() {
    use crate::diagnostics::data::DiagnosticCode;
    use crate::transactions::data::outcomes::conflict_class::ConflictClass;
    use crate::validation::data::{
        custom_rule::{
            failure_cause::CustomInvariantFailureCause, refusal::CustomInvariantRefusal,
        },
        results::{
            CustomInvariantFailureIdentity, CustomInvariantFailureKind,
            CustomInvariantFailurePhase, InvariantViolationFields,
        },
        rule_id::{
            CustomInvariantRuleId, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
        },
    };

    let cause = CustomInvariantFailureCause::DeclaredValue(CustomInvariantRefusal::Empty);
    let error = ConflictClass::InvariantViolation {
        code: DiagnosticCode::InvariantViolation,
        detail: String::new(),
        fields: InvariantViolationFields::CustomInvariantFailure {
            identity: CustomInvariantFailureIdentity::new(CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("test.custom_rule"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            }),
            phase: CustomInvariantFailurePhase::Execution,
            failure: CustomInvariantFailureKind::ExecutionError,
            cause: Some(Box::new(cause.clone())),
            detail: String::new(),
        },
    };
    assert_eq!(
        error
            .to_operator_report(DiagnosticDetail::Standard)
            .chain()
            .len(),
        2,
    );
    let source = error.source().unwrap();
    assert_eq!(source.downcast_ref(), Some(&cause));
    assert_eq!(
        source.source().unwrap().downcast_ref(),
        Some(&CustomInvariantRefusal::Empty)
    );
}

#[test]
fn publication_report_preserves_the_canonical_frame_refusal() {
    use crate::durability::canonical_commit_artifact::{
        encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Encode,
        frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal as Frame,
    };
    use crate::errors::data::{ErrorContext, ErrorOperation, RelationalSubsystem};
    use crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial;
    use crate::transactions::data::commit_log::CommitLog;

    let cause = Frame::UnrecognizedMagic;
    let error = TransactionCommitError::PublicationDenied {
        denial: RelationalPublicationDenial::CanonicalArtifactEncoding(Encode::Frame(cause)),
        context: ErrorContext::new(RelationalSubsystem::Publication, ErrorOperation::Publish),
        commit_log: CommitLog::new(),
    };
    assert_eq!(
        error
            .to_operator_report(DiagnosticDetail::Standard)
            .chain()
            .len(),
        3,
    );
    let frame = error.source().unwrap().source().unwrap().source().unwrap();
    assert_eq!(frame.downcast_ref(), Some(&cause));
}
