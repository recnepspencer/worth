//! A fork that takes its own Signal component still runs its source's program.
//!
//! Branch program activation is Relational truth carried through World, so
//! which program an occurrence runs must not depend on whether the fork shares
//! its source's exact Signal basis or forks Signal alongside Relational.

use crate::bounded_dimension_model::host::{publish_on_second_program, SEED_DIMENSION};
use crate::bounded_dimension_model::presented_request::set_dimension;
use crate::bounded_dimension_model::programs::DimensionProgramP0;
use crate::bounded_dimension_model::readback::read_dimension;
use crate::bounded_dimension_model::settled_verdict::{settle, DimensionVerdict};

/// A value only the second program's law admits.
const HIGH_DIMENSION: u64 = 15;
/// A value only the first program's law admits.
const LOW_DIMENSION: u64 = 3;

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
        settle(set_dimension(&host, fork, HIGH_DIMENSION, 0x9175_0051)),
        DimensionVerdict::Performed(HIGH_DIMENSION),
        "the fork runs the second program its source was running"
    );
    assert_eq!(read_dimension(host.runtime(), fork), HIGH_DIMENSION);
    assert_eq!(
        read_dimension(host.runtime(), main),
        SEED_DIMENSION,
        "a write on the fork cannot reach the branch it forked from"
    );

    assert_eq!(
        settle(set_dimension(&host, fork, LOW_DIMENSION, 0x9175_0052)),
        DimensionVerdict::violated("bounded-dimension-v2"),
        "the fork carries its source's law, not the other rostered program's"
    );
    assert_eq!(read_dimension(host.runtime(), fork), HIGH_DIMENSION);

    let peer = host
        .supported_program::<DimensionProgramP0>()
        .expect("the rostered peer is nameable on every occurrence of this host");
    assert_eq!(
        settle(set_dimension(&peer, fork, LOW_DIMENSION, 0x9175_0053)),
        DimensionVerdict::inactive(),
        "forking both components cannot activate a program the source never ran"
    );
    assert_eq!(read_dimension(host.runtime(), fork), HIGH_DIMENSION);
    assert_eq!(read_dimension(host.runtime(), main), SEED_DIMENSION);
}
