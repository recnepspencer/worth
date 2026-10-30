//! An unresolved publication orders only its own journal. While the runtime
//! serves, the branch-transitions journal refuses as `Unresolved` and resolves
//! its position as unresolved; the canonical commits keep reading and
//! resolving, and their journal does not refuse as unresolved. Store's
//! physical layer still admits one pending root-changing publication per
//! incarnation, so a commit in that window is refused as proven no-effect
//! (`ScopeConflict`) rather than ordered behind the transition. Recovery then
//! decides the transition's fate alone and both journals publish again.

#[allow(unused_imports)]
use crate::production_entry::ordered_journals::unresolved_isolation::*;

use {
    crate::production_entry::ordered_journals::branch_transitions_oracle::measure,
    crate::production_entry::ordered_journals::branch_transitions_oracle::prepared_store,
};

#[test]
fn an_unresolved_journal_blocks_only_itself_until_reopen() {
    let counts = measure(prepared_store(TARGET).path(), TARGET);
    let directives = [
        worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
        worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
    ];
    let isolated = directives.iter().any(|directive| {
        counts.iter().any(|(role, count)| {
            (1..=*count)
                .any(|nth| isolates(*role, std::num::NonZeroU64::new(nth).unwrap(), directive))
        })
    });
    assert!(
        isolated,
        "no unresolved transition left a serving runtime to observe"
    );
}
