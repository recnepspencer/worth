//! Two hosts, one schema, one action, two programs, two different laws.
//!
//! Every host here installs the same two commit-boundary rule contracts. The
//! only thing that varies is which program the occurrence is running, and that
//! alone decides which rule governs a candidate. Nothing in this court asserts
//! a digest: it asserts the verdict a request reached and the value the branch
//! afterwards reports.

use worth_query_host::facade::declaration::application_program::{
    ApplicationProgramAuthoring, ValidatedApplicationProgram,
};

use crate::bounded_dimension_model::host::{
    publish_on_first_program, publish_on_second_program, SEED_DIMENSION,
};
use crate::bounded_dimension_model::presented_request::set_dimension;
use crate::bounded_dimension_model::programs::{
    DimensionProgramP0, DimensionProgramP1, UnrosteredDimensionProgram,
};
use crate::bounded_dimension_model::readback::{observe_head, read_dimension};
use crate::bounded_dimension_model::rules::{V1_CEILING, V2_CEILING, V2_FLOOR};
use crate::bounded_dimension_model::schema::BoundedDimensionSchema;
use crate::bounded_dimension_model::settled_verdict::{settle, DimensionVerdict};

/// A value only the first program's law admits.
const LOW_DIMENSION: u64 = 3;
/// A value only the second program's law admits.
const HIGH_DIMENSION: u64 = 15;
/// A second value only the first program's law admits, used on a fork.
const FORK_DIMENSION: u64 = 9;

/// The two probe values must sit on opposite sides of the two laws, or nothing
/// below distinguishes the programs. Checked at compile time from the bounds.
const _: () = {
    assert!(LOW_DIMENSION <= V1_CEILING);
    assert!(LOW_DIMENSION < V2_FLOOR);
    assert!(HIGH_DIMENSION > V1_CEILING);
    assert!(V2_FLOOR <= HIGH_DIMENSION && HIGH_DIMENSION <= V2_CEILING);
    assert!(FORK_DIMENSION <= V1_CEILING);
    assert!(SEED_DIMENSION <= V1_CEILING);
    assert!(V2_FLOOR <= SEED_DIMENSION && SEED_DIMENSION <= V2_CEILING);
};

#[test]
fn two_programs_over_one_schema_carry_two_revisions() {
    let first = validated_first();
    let second = validated_second();
    assert_ne!(
        first.revision(),
        second.revision(),
        "programs declaring different rules cannot share a canonical revision"
    );
    assert_eq!(
        first.revision(),
        validated_first().revision(),
        "revalidating the same authored meaning must mint the same revision"
    );
    assert_eq!(second.revision(), validated_second().revision());
}

#[test]
fn a_first_program_occurrence_enforces_the_first_law() {
    let host = publish_on_first_program();
    let branch = host.current_world();

    assert_eq!(
        settle(set_dimension(&host, branch, LOW_DIMENSION, 0x9175_0001)),
        DimensionVerdict::Performed(LOW_DIMENSION)
    );
    assert_eq!(read_dimension(host.runtime(), branch), LOW_DIMENSION);

    assert_eq!(
        settle(set_dimension(&host, branch, HIGH_DIMENSION, 0x9175_0002)),
        DimensionVerdict::violated("bounded-dimension-v1"),
        "the value the second law admits must be refused by the first program's law"
    );
    assert_eq!(
        read_dimension(host.runtime(), branch),
        LOW_DIMENSION,
        "a refused candidate cannot have moved the branch"
    );
}

#[test]
fn a_second_program_occurrence_enforces_the_second_law() {
    let host = publish_on_second_program();
    let branch = host.current_world();

    assert_eq!(
        settle(set_dimension(&host, branch, HIGH_DIMENSION, 0x9175_0011)),
        DimensionVerdict::Performed(HIGH_DIMENSION)
    );
    assert_eq!(read_dimension(host.runtime(), branch), HIGH_DIMENSION);

    assert_eq!(
        settle(set_dimension(&host, branch, LOW_DIMENSION, 0x9175_0012)),
        DimensionVerdict::violated("bounded-dimension-v2"),
        "the value the first law admits must be refused by the second program's law"
    );
    assert_eq!(
        read_dimension(host.runtime(), branch),
        HIGH_DIMENSION,
        "a refused candidate cannot have moved the branch"
    );
}

#[test]
fn a_rostered_peer_program_is_named_but_not_active() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let peer = host
        .supported_program::<DimensionProgramP1>()
        .expect("the peer program this host rostered must be nameable");
    assert!(
        host.supported_program::<UnrosteredDimensionProgram>()
            .is_none(),
        "a program this host never rostered cannot be nameable"
    );

    let before = observe_head(host.runtime(), branch);
    assert_eq!(
        settle(set_dimension(
            &peer,
            branch,
            SEED_DIMENSION + 1,
            0x9175_0021
        )),
        DimensionVerdict::inactive(),
        "a rostered peer cannot act on an occurrence running another program"
    );
    let after = observe_head(host.runtime(), branch);
    assert_eq!(
        read_dimension(host.runtime(), branch),
        SEED_DIMENSION,
        "a denial before effects cannot have moved the branch"
    );
    assert_eq!(
        before.selected_commit(),
        after.selected_commit(),
        "a denial before effects cannot have moved the commit head"
    );
}

#[test]
fn a_rostered_peer_program_is_named_but_not_active_on_a_second_program_host() {
    let host = publish_on_second_program();
    let branch = host.current_world();
    let peer = host
        .supported_program::<DimensionProgramP0>()
        .expect("the peer program this host rostered must be nameable");

    let before = observe_head(host.runtime(), branch);
    assert_eq!(
        settle(set_dimension(
            &peer,
            branch,
            SEED_DIMENSION + 1,
            0x9175_0031
        )),
        DimensionVerdict::inactive()
    );
    let after = observe_head(host.runtime(), branch);
    assert_eq!(read_dimension(host.runtime(), branch), SEED_DIMENSION);
    assert_eq!(before.selected_commit(), after.selected_commit());
}

#[test]
fn a_forked_branch_inherits_the_law_its_source_was_running() {
    let host = publish_on_first_program();
    let main = host.current_world();
    let fork = host
        .runtime()
        .branches()
        .fork(main)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the Relational fork must publish");

    assert_eq!(
        settle(set_dimension(&host, fork, FORK_DIMENSION, 0x9175_0041)),
        DimensionVerdict::Performed(FORK_DIMENSION),
        "the fork runs the program its source was running"
    );
    assert_eq!(read_dimension(host.runtime(), fork), FORK_DIMENSION);
    assert_eq!(
        read_dimension(host.runtime(), main),
        SEED_DIMENSION,
        "a write on the fork cannot reach the branch it forked from"
    );

    assert_eq!(
        settle(set_dimension(&host, fork, LOW_DIMENSION, 0x9175_0042)),
        DimensionVerdict::Performed(LOW_DIMENSION),
        "the other rostered program's law does not reach this fork"
    );
    assert_eq!(read_dimension(host.runtime(), fork), LOW_DIMENSION);

    assert_eq!(
        settle(set_dimension(&host, fork, HIGH_DIMENSION, 0x9175_0043)),
        DimensionVerdict::violated("bounded-dimension-v1"),
        "the fork carries its source's law, not the other rostered program's"
    );
    assert_eq!(read_dimension(host.runtime(), fork), LOW_DIMENSION);

    let peer = host
        .supported_program::<DimensionProgramP1>()
        .expect("the rostered peer is nameable on every occurrence of this host");
    assert_eq!(
        settle(set_dimension(&peer, fork, SEED_DIMENSION, 0x9175_0044)),
        DimensionVerdict::inactive(),
        "forking an occurrence cannot activate a program it never ran"
    );
    assert_eq!(read_dimension(host.runtime(), fork), LOW_DIMENSION);
    assert_eq!(read_dimension(host.runtime(), main), SEED_DIMENSION);
}

fn validated_first() -> ValidatedApplicationProgram<BoundedDimensionSchema, DimensionProgramP0> {
    ApplicationProgramAuthoring::<BoundedDimensionSchema, DimensionProgramP0>::begin()
        .validated_program()
        .expect("the first program is authored validly")
}

fn validated_second() -> ValidatedApplicationProgram<BoundedDimensionSchema, DimensionProgramP1> {
    ApplicationProgramAuthoring::<BoundedDimensionSchema, DimensionProgramP1>::begin()
        .validated_program()
        .expect("the second program is authored validly")
}
