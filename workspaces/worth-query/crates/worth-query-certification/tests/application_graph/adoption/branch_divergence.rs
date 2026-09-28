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

use crate::document_retention_model::host::{
    publish_on_first_program, publish_on_second_program, SEED_RETENTION,
};
use crate::document_retention_model::presented_request::set_retention;
use crate::document_retention_model::programs::{
    RetentionProgramP0, RetentionProgramP1, UnrosteredRetentionProgram,
};
use crate::document_retention_model::readback::{observe_head, read_retention};
use crate::document_retention_model::rules::{V1_CEILING, V2_CEILING, V2_FLOOR};
use crate::document_retention_model::schema::DocumentRetentionSchema;
use crate::document_retention_model::settled_verdict::{settle, RetentionVerdict};

/// A value only the first program's law admits.
const LOW_RETENTION: u64 = 3;
/// A value only the second program's law admits.
const HIGH_RETENTION: u64 = 15;
/// A second value only the first program's law admits, used on a fork.
const FORK_RETENTION: u64 = 9;

/// The two probe values must sit on opposite sides of the two laws, or nothing
/// below distinguishes the programs. Checked at compile time from the bounds.
const _: () = {
    assert!(LOW_RETENTION <= V1_CEILING);
    assert!(LOW_RETENTION < V2_FLOOR);
    assert!(HIGH_RETENTION > V1_CEILING);
    assert!(V2_FLOOR <= HIGH_RETENTION && HIGH_RETENTION <= V2_CEILING);
    assert!(FORK_RETENTION <= V1_CEILING);
    assert!(SEED_RETENTION <= V1_CEILING);
    assert!(V2_FLOOR <= SEED_RETENTION && SEED_RETENTION <= V2_CEILING);
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
        settle(set_retention(&host, branch, LOW_RETENTION, 0x9175_0001)),
        RetentionVerdict::Performed(LOW_RETENTION)
    );
    assert_eq!(read_retention(host.runtime(), branch), LOW_RETENTION);

    assert_eq!(
        settle(set_retention(&host, branch, HIGH_RETENTION, 0x9175_0002)),
        RetentionVerdict::violated("document-retention-v1"),
        "the value the second law admits must be refused by the first program's law"
    );
    assert_eq!(
        read_retention(host.runtime(), branch),
        LOW_RETENTION,
        "a refused candidate cannot have moved the branch"
    );
}

#[test]
fn a_second_program_occurrence_enforces_the_second_law() {
    let host = publish_on_second_program();
    let branch = host.current_world();

    assert_eq!(
        settle(set_retention(&host, branch, HIGH_RETENTION, 0x9175_0011)),
        RetentionVerdict::Performed(HIGH_RETENTION)
    );
    assert_eq!(read_retention(host.runtime(), branch), HIGH_RETENTION);

    assert_eq!(
        settle(set_retention(&host, branch, LOW_RETENTION, 0x9175_0012)),
        RetentionVerdict::violated("document-retention-v2"),
        "the value the first law admits must be refused by the second program's law"
    );
    assert_eq!(
        read_retention(host.runtime(), branch),
        HIGH_RETENTION,
        "a refused candidate cannot have moved the branch"
    );
}

#[test]
fn a_rostered_peer_program_is_named_but_not_active() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    assert!(
        host.supported_program::<RetentionProgramP1>().is_some(),
        "the peer program this host rostered must be nameable"
    );
    assert!(
        host.supported_program::<UnrosteredRetentionProgram>()
            .is_none(),
        "a program this host never rostered cannot be nameable"
    );

    let before = observe_head(host.runtime(), branch);
    assert_eq!(
        settle(set_retention(&host, branch, HIGH_RETENTION, 0x9175_0021)),
        RetentionVerdict::violated("document-retention-v1"),
        "a rostered peer's law cannot govern an occurrence running another program"
    );
    let after = observe_head(host.runtime(), branch);
    assert_eq!(
        read_retention(host.runtime(), branch),
        SEED_RETENTION,
        "a refused candidate cannot have moved the branch"
    );
    assert_eq!(
        before.selected_commit(),
        after.selected_commit(),
        "a refused candidate cannot have moved the commit head"
    );
}

#[test]
fn a_rostered_peer_program_is_named_but_not_active_on_a_second_program_host() {
    let host = publish_on_second_program();
    let branch = host.current_world();
    assert!(
        host.supported_program::<RetentionProgramP0>().is_some(),
        "the peer program this host rostered must be nameable"
    );

    let before = observe_head(host.runtime(), branch);
    assert_eq!(
        settle(set_retention(&host, branch, LOW_RETENTION, 0x9175_0031)),
        RetentionVerdict::violated("document-retention-v2")
    );
    let after = observe_head(host.runtime(), branch);
    assert_eq!(read_retention(host.runtime(), branch), SEED_RETENTION);
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
        settle(set_retention(&host, fork, FORK_RETENTION, 0x9175_0041)),
        RetentionVerdict::Performed(FORK_RETENTION),
        "the fork runs the program its source was running"
    );
    assert_eq!(read_retention(host.runtime(), fork), FORK_RETENTION);
    assert_eq!(
        read_retention(host.runtime(), main),
        SEED_RETENTION,
        "a write on the fork cannot reach the branch it forked from"
    );

    assert_eq!(
        settle(set_retention(&host, fork, LOW_RETENTION, 0x9175_0042)),
        RetentionVerdict::Performed(LOW_RETENTION),
        "the other rostered program's law does not reach this fork"
    );
    assert_eq!(read_retention(host.runtime(), fork), LOW_RETENTION);

    assert_eq!(
        settle(set_retention(&host, fork, HIGH_RETENTION, 0x9175_0043)),
        RetentionVerdict::violated("document-retention-v1"),
        "the fork carries its source's law, not the other rostered program's"
    );
    assert_eq!(read_retention(host.runtime(), fork), LOW_RETENTION);

    assert_eq!(read_retention(host.runtime(), main), SEED_RETENTION);
}

fn validated_first() -> ValidatedApplicationProgram<DocumentRetentionSchema, RetentionProgramP0> {
    ApplicationProgramAuthoring::<DocumentRetentionSchema, RetentionProgramP0>::begin()
        .validated_program()
        .expect("the first program is authored validly")
}

fn validated_second() -> ValidatedApplicationProgram<DocumentRetentionSchema, RetentionProgramP1> {
    ApplicationProgramAuthoring::<DocumentRetentionSchema, RetentionProgramP1>::begin()
        .validated_program()
        .expect("the second program is authored validly")
}
