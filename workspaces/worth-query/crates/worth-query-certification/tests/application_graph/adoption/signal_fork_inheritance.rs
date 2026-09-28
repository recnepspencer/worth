//! A fork that takes its own Signal component still runs its source's program.
//!
//! Branch program activation is Relational truth carried through World, so
//! which program an occurrence runs must not depend on whether the fork shares
//! its source's exact Signal basis or forks Signal alongside Relational.

use crate::document_retention_model::host::{publish_on_second_program, SEED_RETENTION};
use crate::document_retention_model::presented_request::set_retention;
use crate::document_retention_model::readback::read_retention;
use crate::document_retention_model::settled_verdict::{settle, RetentionVerdict};

/// A value only the second program's law admits.
const HIGH_RETENTION: u64 = 15;
/// A value only the first program's law admits.
const LOW_RETENTION: u64 = 3;

#[test]
fn a_fork_of_both_components_inherits_the_law_its_source_was_running() {
    let host = publish_on_second_program();
    let main = host.current_world();
    let fork = host
        .runtime()
        .branches()
        .fork(main)
        .components(|components| components.fork_relational().fork_signal())
        .create()
        .expect("the Relational and Signal fork must publish");

    assert_eq!(
        settle(set_retention(&host, fork, HIGH_RETENTION, 0x9175_0051)),
        RetentionVerdict::Performed(HIGH_RETENTION),
        "the fork runs the second program its source was running"
    );
    assert_eq!(read_retention(host.runtime(), fork), HIGH_RETENTION);
    assert_eq!(
        read_retention(host.runtime(), main),
        SEED_RETENTION,
        "a write on the fork cannot reach the branch it forked from"
    );

    assert_eq!(
        settle(set_retention(&host, fork, LOW_RETENTION, 0x9175_0052)),
        RetentionVerdict::violated("document-retention-v2"),
        "the fork carries its source's law, not the other rostered program's"
    );
    assert_eq!(read_retention(host.runtime(), fork), HIGH_RETENTION);
    assert_eq!(read_retention(host.runtime(), main), SEED_RETENTION);
}
