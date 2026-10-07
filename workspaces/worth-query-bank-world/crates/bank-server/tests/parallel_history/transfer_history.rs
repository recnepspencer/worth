//! Each request is checked against signed postings computed from inputs.
use super::{
    journal_model::Journal,
    observation::{Posture, SerialObserver},
    seeded_world::{World, SEED},
};
#[test]
fn seeded_serial_transfers_match_journal_conservation_authorization_and_retries() {
    let world = World::new(SEED);
    let mut model = Journal::new(&[(11, 1), (12, 2)], 100, 11, world.funding);
    let mut observer = SerialObserver::new(&world);
    for (place, input) in world.requests(SEED).iter().enumerate() {
        let expected = model.apply(input);
        let actual = observer.transfer(input, Posture::Serial);
        assert_eq!(
            actual.as_ref().ok(), Some(&expected),
            "request {place}: declared commit, authorization, funds and retry outcome; native {actual:?}"
        );
        let (postings, report) = observer.journal();
        assert_eq!(
            postings,
            model.canonical_postings(),
            "request {place}: exact committed postings"
        );
        assert_eq!(
            postings.iter().map(|posting| posting.amount).sum::<i64>(),
            0,
            "journal including cash conserves signed units"
        );
        assert_eq!(
            observer.balances(),
            model.balances,
            "request {place}: public account summary balances"
        );
        assert!(report.observed_read_work > 0);
    }
}
