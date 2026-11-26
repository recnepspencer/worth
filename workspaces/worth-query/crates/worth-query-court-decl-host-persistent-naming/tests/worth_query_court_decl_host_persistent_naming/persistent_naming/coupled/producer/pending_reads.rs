//! #426: the producer's Initial, for an edit value registered here, reads
//! the name and birth of a member an earlier stage of the same envelope
//! created and named, on the envelope's unpublished candidate. Every other
//! Initial is unchanged.
use super::*;
use std::collections::BTreeMap;
use std::sync::Mutex;
use worth_query_decl::facade::application_naming::RecordedNameBirth;

/// What the Initial read: the member's name and that name's birth.
pub type Read = Result<(WorthQueryName<Member>, RecordedNameBirth<Member>), String>;

static PLANS: Mutex<BTreeMap<u64, String>> = Mutex::new(BTreeMap::new());
static OBSERVED: Mutex<BTreeMap<u64, Read>> = Mutex::new(BTreeMap::new());

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Has the Initial for an edit to `value` read member `key`'s name.
pub fn read_pending(value: u64, key: &str) {
    lock(&PLANS).insert(value, key.to_owned());
}

/// What the last Initial for an edit to `value` read, or its refusal.
pub fn observed(value: u64) -> Option<Read> {
    lock(&OBSERVED).remove(&value)
}

pub(super) fn decide(
    reader: &mut DecisionReader<'_, '_, '_, Schema, initial::Binding>,
    value: u64,
) -> Result<(), HandlerExecutionDenial> {
    let Some(key) = lock(&PLANS).get(&value).cloned() else {
        return Ok(());
    };
    let member = reader.resolve_entity(MemberKey::at("MemberKey"), key)?;
    let names = reader.reader();
    let read = names
        .name_of(&member)
        .and_then(|name| Ok((name, names.birth_of(&name)?)));
    let recorded = read.clone().map_err(|denial| format!("{denial:?}"));
    lock(&OBSERVED).insert(value, recorded);
    read.map(|_| ()).map_err(HandlerExecutionDenial::new)
}
