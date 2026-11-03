//! Repeated fragment custody must not reconstruct a shared ledger's metadata.
use super::SerialMemoryBudget;
use std::cell::Cell;

thread_local! {
    static SETUPS: Cell<Option<usize>> = const { Cell::new(None) };
}

pub(super) fn setup() {
    SETUPS.update(|count| count.map(|count| count + 1));
}

struct Observation;
impl Observation {
    fn start() -> Self {
        SETUPS.with(|count| assert_eq!(count.replace(Some(0)), None));
        Self
    }
    fn setups(&self) -> usize {
        SETUPS.with(|count| count.get().unwrap())
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        SETUPS.set(None);
    }
}

#[test]
fn repeated_fragment_reservations_do_not_rebuild_shared_ledger_setup() {
    for rows in [8, 64, 512] {
        let budget = SerialMemoryBudget::new(Some(4096));
        let observation = Observation::start();
        for _ in 0..rows {
            let inherited = budget.within_parent(&budget);
            let mut fragment = inherited.reserve(0).unwrap();
            fragment.resize(1024).unwrap();
            drop(fragment);
        }
        assert_eq!(budget.charged_memory_bytes(), 0);
        let setups = observation.setups();
        eprintln!("rows={rows} serial ledger setups={setups}");
        assert!(
            setups <= 4,
            "{rows} rows reconstructed {setups} ledger setups"
        );
    }
}
