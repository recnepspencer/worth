//! A recovered Native effect whose recorded open is missing, conflicting or
//! names another predecessor refuses before any World service exists.
use super::*;

/// Kill after the durable Native effect of the second publication, then damage
/// the recorded open between processes and reopen through real readmission.
fn refusal(change: impl FnOnce(&mut Vec<u8>, u64) -> bool) -> WorldReadmissionDenial {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut fixture = Fixture::journal(&request);
    match fixture.offer(&request, "primary") {
        WorldDurableBootstrapOutcome::Performed(_) => {}
        other => panic!("real bootstrap: {other:?}"),
    }
    let earlier = root(&fixture).selected_commit().ordinal();
    let mut model = Model::default();
    assert!(!model.attempt(&fixture, &request, Step::Acknowledged));
    assert!(model.attempt(&fixture, &request, Step::EffectOnly));
    fixture
        .recorder
        .rewrite_latest(WorldDurableJournal::Custody, |mut bytes| {
            assert_eq!(bytes[52], 3, "the latest custody row is the original open");
            if !change(&mut bytes, earlier) {
                return None;
            }
            super::super::journey::rehash(&mut bytes);
            Some(bytes)
        });
    match fixture.try_reopen(&request) {
        Ok(_) => panic!("a damaged original cannot reach service"),
        Err(denial) => denial,
    }
}

/// Offsets of the open payload's association and expected head fields.
struct Open {
    association_incarnation: usize,
    head_incarnation: usize,
    head_generation: usize,
    head_commit: usize,
}
impl Open {
    fn locate(bytes: &[u8]) -> Self {
        let name = u16::from_le_bytes([bytes[52 + 25], bytes[52 + 26]]) as usize;
        let association_incarnation = 52 + 27 + name + 8;
        let head = association_incarnation + 8 + 8 + 8;
        let head_incarnation = head + 24 + 2 + name;
        Self {
            association_incarnation,
            head_incarnation,
            head_generation: head_incarnation + 8,
            head_commit: head_incarnation + 16,
        }
    }
}
fn write(bytes: &mut [u8], at: usize, value: u64) {
    bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
}
fn read(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

#[test]
fn a_native_effect_without_its_recorded_open_refuses_as_an_orphan() {
    assert!(matches!(
        refusal(|_, _| false),
        WorldReadmissionDenial::OrphanNativeAssociation
    ));
}

/// The open's association and head must agree, so another association names a
/// branch incarnation the recorded lineage never had.
#[test]
fn a_native_effect_whose_open_records_another_association_refuses_before_service() {
    let denial = refusal(|bytes, _| {
        let open = Open::locate(bytes);
        for at in [open.association_incarnation, open.head_incarnation] {
            let actual = read(bytes, at);
            write(bytes, at, actual + 1);
        }
        true
    });
    assert!(
        matches!(denial, WorldReadmissionDenial::MissingPredecessor),
        "{denial:?}"
    );
}

#[test]
fn a_native_effect_admitted_against_another_recorded_predecessor_refuses_at_native() {
    let denial = refusal(|bytes, earlier| {
        let open = Open::locate(bytes);
        assert_eq!(read(bytes, open.head_generation), 1);
        write(bytes, open.head_generation, 0);
        write(bytes, open.head_commit, earlier);
        true
    });
    let WorldReadmissionDenial::NativeCustody { cause, .. } = denial else {
        panic!("Native proves the recorded predecessor: {denial:?}")
    };
    let WorldReadmissionDenial::NativePublication(refused) = *cause else {
        panic!("Native publication refusal: {cause:?}")
    };
    assert!(matches!(
        refused.cause(),
        worth_relational::facade::durability::RelationalRecoveredPublicationAdmissionCause::SubjectMismatch
    ));
}
