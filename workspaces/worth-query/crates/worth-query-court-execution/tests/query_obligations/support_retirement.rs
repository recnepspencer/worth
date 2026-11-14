//! Value admission for an existing original grant while its support retires.
use std::num::NonZeroU64;
use worth_foundational::facade::{
    CanonicalDigestId, DurableCompositePublicationAttemptIdentity, HomeIdentity,
    ProgramSupportObligationIdentity, ProgramSupportObligationKind, ProgramSupportStatus,
};
use worth_foundational::facade::{DurableWorldOwnerIdentity, WorldOwnerSlotIdentity};
use worth_query_execution::facade::query_obligations::*;
#[test]
fn retiring_support_accepts_existing_original_transfers_but_not_new_adoptions() {
    let home = HomeIdentity::from_observed_bytes([1; 16]).unwrap();
    let revision = CanonicalDigestId::new([2; 32]);
    let mut ledger = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
    for record in [
        QueryProgramSupportRecord::Install { revision },
        QueryProgramSupportRecord::Status {
            revision,
            before: ProgramSupportStatus::Active,
            after: ProgramSupportStatus::Retiring,
        },
    ] {
        let frame = ledger.prepare(record, &mut generations).unwrap();
        ledger.readmit(frame).unwrap();
    }
    let row = QueryProgramSupportObligation {
        identity: ProgramSupportObligationIdentity::from_recorded(
            home,
            NonZeroU64::new(3).unwrap(),
        ),
        source: revision,
        target: revision,
        kind: ProgramSupportObligationKind::OriginalInterpretation,
        original: DurableCompositePublicationAttemptIdentity::from_recorded(
            DurableWorldOwnerIdentity::from_recorded(
                home,
                WorldOwnerSlotIdentity::from_recorded(0),
            ),
            1,
        ),
    };
    let frame = ledger
        .prepare(
            QueryProgramSupportRecord::Acquire(row.clone()),
            &mut generations,
        )
        .unwrap();
    let mut adoption = row;
    adoption.kind = ProgramSupportObligationKind::BranchAdoption;
    assert_eq!(
        ledger.prepare(
            QueryProgramSupportRecord::Acquire(adoption),
            &mut generations
        ),
        Err(QueryObligationDenial::SupportUnavailable)
    );
    ledger.readmit(frame).unwrap();
    assert_eq!(ledger.counts(&revision), (1, 0));
    assert_eq!(
        ledger.prepare(
            QueryProgramSupportRecord::Status {
                revision,
                before: ProgramSupportStatus::Retiring,
                after: ProgramSupportStatus::Retired
            },
            &mut generations
        ),
        Err(QueryObligationDenial::SupportUnavailable)
    );
}
