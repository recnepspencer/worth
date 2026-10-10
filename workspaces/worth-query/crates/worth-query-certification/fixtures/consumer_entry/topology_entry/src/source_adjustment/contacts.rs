//! Contact observations over the unchanged ordinary source operation.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use super::PlanarSourceAdjustment;
use worth_query_consumer_values::PositiveLength;

type Contacts = BTreeMap<(String, u64), (usize, usize)>;

fn contacts() -> &'static Mutex<Contacts> {
    static CONTACTS: OnceLock<Mutex<Contacts>> = OnceLock::new();
    CONTACTS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Resets only the actual input whose ordinary handler contacts are observed.
pub fn reset_planar_source_adjustment_contacts(input: &PlanarSourceAdjustment) {
    contacts().lock().unwrap().remove(&(
        input.scope_key.clone(),
        PositiveLength::get(&input.replacement_y),
    ));
}

/// Returns decision and candidate contacts for the exact ordinary input.
pub fn planar_source_adjustment_contacts(input: &PlanarSourceAdjustment) -> (usize, usize) {
    contacts()
        .lock()
        .unwrap()
        .get(&(
            input.scope_key.clone(),
            PositiveLength::get(&input.replacement_y),
        ))
        .copied()
        .unwrap_or_default()
}

pub(crate) fn decision(input: &PlanarSourceAdjustment) {
    contacts()
        .lock()
        .unwrap()
        .entry((
            input.scope_key.clone(),
            PositiveLength::get(&input.replacement_y),
        ))
        .or_default()
        .0 += 1;
}

pub(crate) fn candidate(input: &PlanarSourceAdjustment) {
    contacts()
        .lock()
        .unwrap()
        .entry((
            input.scope_key.clone(),
            PositiveLength::get(&input.replacement_y),
        ))
        .or_default()
        .1 += 1;
}
