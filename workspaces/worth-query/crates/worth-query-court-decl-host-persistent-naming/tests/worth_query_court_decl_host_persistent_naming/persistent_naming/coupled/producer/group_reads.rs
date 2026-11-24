//! #361: the producer's Initial, deciding against the envelope's unpublished
//! candidate, reads a member's loop (`group_name_of`) and resolves a held
//! loop (`resolve_group`), and its build carries the loop it read. Only an
//! Initial whose edit carries a value registered here reads groups.
use super::*;
use std::collections::BTreeMap;
use std::sync::Mutex;
use worth_query_decl::facade::application_naming::{NameSelection, ResolvedName};

/// What the decision read: the member's loop and the held loop's
/// resolution, each as the reader answered or refused it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observed {
    pub loop_of: Result<Option<WorthQueryName<Loop>>, String>,
    pub resolved: Result<Option<WorthQueryName<Loop>>, String>,
    /// The loop the build received from the decision.
    pub built_with: Option<WorthQueryName<Loop>>,
}

static PLANS: Mutex<BTreeMap<u64, (WorthQueryName<Member>, WorthQueryName<Loop>)>> =
    Mutex::new(BTreeMap::new());
static OBSERVED: Mutex<BTreeMap<u64, Observed>> = Mutex::new(BTreeMap::new());

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Has the Initial for an edit to `value` read `member`'s loop and resolve
/// `held`.
pub fn read_groups(value: u64, member: WorthQueryName<Member>, held: WorthQueryName<Loop>) {
    lock(&PLANS).insert(value, (member, held));
}

/// What the last Initial for an edit to `value` read and built with.
pub fn observed(value: u64) -> Option<Observed> {
    lock(&OBSERVED).remove(&value)
}

/// Reads the planned groups; the loop `resolve_group` answered `Exact`
/// is what the build carries.
pub(super) fn decide(
    reader: &mut DecisionReader<'_, '_, '_, Schema, initial::Binding>,
    value: u64,
) -> Option<WorthQueryName<Loop>> {
    let (member, held) = lock(&PLANS).get(&value).copied()?;
    let names = reader.reader();
    let loop_of = names
        .group_name_of::<Loop, Member>(&member)
        .map_err(|denial| format!("{denial:?}"));
    let resolved = match names.resolve_group::<Loop, Member>(&held, &NameSelection::Exact) {
        Ok(Ok(ResolvedName::Exact(group))) => Ok(Some(group)),
        Ok(other) => Err(format!("{other:?}")),
        Err(denial) => Err(format!("{denial:?}")),
    };
    let carried = resolved.clone().ok().flatten();
    lock(&OBSERVED).insert(
        value,
        Observed {
            loop_of,
            resolved,
            built_with: None,
        },
    );
    carried
}

/// Records the loop the build received.
pub(super) fn built(value: u64, group: Option<WorthQueryName<Loop>>) {
    if let Some(observed) = lock(&OBSERVED).get_mut(&value) {
        observed.built_with = group;
    }
}
