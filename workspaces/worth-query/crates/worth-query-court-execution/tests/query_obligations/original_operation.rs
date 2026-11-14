//! Authority-free original DATA cannot replace an acknowledged native identity.
use std::num::NonZeroU64;
use worth_foundational::facade::{CanonicalDigestId, HomeIdentity};
use worth_query_execution::facade::query_obligations::*;

fn append(
    ledger: &mut QueryObligationLedger,
    generations: &mut QueryObligationPublicationGenerations,
    record: impl Into<QueryObligationRecord>,
) -> Vec<u8> {
    let frame = ledger.prepare(record, generations).unwrap();
    let bytes = frame.bytes().to_vec();
    ledger
        .readmit(QueryObligationFrame::readmit(&bytes).unwrap())
        .unwrap();
    bytes
}

#[test]
fn original_operation_data_preserves_identity_and_resume_position() {
    let home = HomeIdentity::from_observed_bytes([31; 16]).unwrap();
    let revision = CanonicalDigestId::new([71; 32]);
    let mut live = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&live);
    let install = append(
        &mut live,
        &mut generations,
        QueryProgramSupportRecord::Install { revision },
    );
    let identity =
        ProgramSupportObligationIdentity::from_recorded(home, NonZeroU64::new(2).unwrap());
    let row = QueryOriginalOperationObligation {
        identity,
        data: QueryOriginalOperationData {
            revision: *revision.bytes(),
            program: "installed-program".into(),
            action: "edit".into(),
            binding: "document-edit".into(),
            branch: "main".into(),
            description: b"captured input, key, preconditions and exact source".to_vec(),
        },
        position: QueryOriginalOperationPosition::Initial,
    };
    let retained = append(
        &mut live,
        &mut generations,
        QueryOriginalOperationRecord::Retain(row.clone()),
    );
    let advanced = append(
        &mut live,
        &mut generations,
        QueryOriginalOperationRecord::Advance {
            identity,
            position: QueryOriginalOperationPosition::Resolving,
        },
    );
    let mut cold = QueryObligationLedger::empty(home);
    for bytes in [install, retained, advanced] {
        cold.readmit(QueryObligationFrame::readmit(&bytes).unwrap())
            .unwrap();
    }
    let recovered = cold.original_operations().next().unwrap();
    assert_eq!(recovered.identity, identity);
    assert_eq!(recovered.data, row.data);
    assert_eq!(
        recovered.position,
        QueryOriginalOperationPosition::Resolving
    );
    assert_eq!(
        cold.counts(&revision),
        (0, 1),
        "original custody pins its installed interpretation"
    );
    let mut generations = QueryObligationPublicationGenerations::from_journal(&cold);
    let mut substituted = row.clone();
    substituted.data.description = b"substituted operation".to_vec();
    assert_eq!(
        cold.prepare(
            QueryOriginalOperationRecord::Retain(substituted),
            &mut generations
        ),
        Err(QueryObligationDenial::ObligationSubjectMismatch)
    );
    let missing =
        ProgramSupportObligationIdentity::from_recorded(home, NonZeroU64::new(99).unwrap());
    assert_eq!(
        cold.prepare(
            QueryOriginalOperationRecord::Advance {
                identity: missing,
                position: QueryOriginalOperationPosition::Resolving,
            },
            &mut generations
        ),
        Err(QueryObligationDenial::UnknownObligation)
    );
    let foreign = ProgramSupportObligationIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([32; 16]).unwrap(),
        identity.acquire_position(),
    );
    assert_eq!(
        cold.prepare(
            QueryOriginalOperationRecord::Discharge { identity: foreign },
            &mut generations
        ),
        Err(QueryObligationDenial::ForeignHome)
    );
    append(
        &mut cold,
        &mut generations,
        QueryOriginalOperationRecord::Discharge { identity },
    );
    assert!(cold.original_operations().next().is_none());
    assert_eq!(cold.counts(&revision), (0, 0));
    assert_eq!(
        cold.prepare(
            QueryOriginalOperationRecord::Discharge { identity },
            &mut generations
        ),
        Err(QueryObligationDenial::UnknownObligation)
    );
}
