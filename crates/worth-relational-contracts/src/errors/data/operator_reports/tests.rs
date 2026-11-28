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
