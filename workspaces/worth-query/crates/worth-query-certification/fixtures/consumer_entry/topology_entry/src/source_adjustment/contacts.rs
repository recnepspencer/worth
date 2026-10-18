//! Thread-scoped contacts from the synchronous ordinary source callbacks.

use std::cell::RefCell;
use std::collections::BTreeMap;

use super::PlanarSourceAdjustment;
use worth_query_consumer_values::PositiveLength;

type Contacts = BTreeMap<(String, u64), (usize, usize)>;

thread_local! {
    // Independent courts may use identical inputs. Ordinary decide/candidate
    // callbacks run on their caller's thread; thread exit drops its observations.
    static CONTACTS: RefCell<Contacts> = const { RefCell::new(BTreeMap::new()) };
}

/// Resets this thread's observations for the exact ordinary input.
pub fn reset_planar_source_adjustment_contacts(input: &PlanarSourceAdjustment) {
    CONTACTS.with_borrow_mut(|contacts| {
        contacts.remove(&(
            input.scope_key.clone(),
            PositiveLength::get(&input.replacement_y),
        ));
    });
}

/// Returns this thread's real decision and candidate contacts for the exact input.
pub fn planar_source_adjustment_contacts(input: &PlanarSourceAdjustment) -> (usize, usize) {
    CONTACTS.with_borrow(|contacts| {
        contacts
            .get(&(
                input.scope_key.clone(),
                PositiveLength::get(&input.replacement_y),
            ))
            .copied()
            .unwrap_or_default()
    })
}

pub(crate) fn decision(input: &PlanarSourceAdjustment) {
    CONTACTS.with_borrow_mut(|contacts| {
        contacts
            .entry((
                input.scope_key.clone(),
                PositiveLength::get(&input.replacement_y),
            ))
            .or_default()
            .0 += 1;
    });
}

pub(crate) fn candidate(input: &PlanarSourceAdjustment) {
    CONTACTS.with_borrow_mut(|contacts| {
        contacts
            .entry((
                input.scope_key.clone(),
                PositiveLength::get(&input.replacement_y),
            ))
            .or_default()
            .1 += 1;
    });
}
