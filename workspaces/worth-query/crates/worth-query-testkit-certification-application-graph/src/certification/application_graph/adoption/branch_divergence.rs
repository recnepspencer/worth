//! Two hosts, one schema, one action, two programs, two different laws.
//!
//! Every host here installs the same two commit-boundary rule contracts. The
//! only thing that varies is which program the occurrence is running, and that
//! alone decides which rule governs a candidate. Nothing in this court asserts
//! a digest: it asserts the verdict a request reached and the value the branch
//! afterwards reports.

pub use worth_query_host::facade::declaration::application_program::{
    ApplicationProgramAuthoring, ValidatedApplicationProgram,
};

pub use worth_query_testkit_document_retention::facade::document_retention_model::host::{
    publish_on_first_program, publish_on_second_program, SEED_RETENTION,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::presented_request::set_retention;
pub use worth_query_testkit_document_retention::facade::document_retention_model::programs::{
    RetentionProgramP0, RetentionProgramP1, UnrosteredRetentionProgram,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::readback::{
    observe_head, read_retention,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::rules::{
    V1_CEILING, V2_CEILING, V2_FLOOR,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::schema::DocumentRetentionSchema;
pub use worth_query_testkit_document_retention::facade::document_retention_model::settled_verdict::{
    settle, RetentionVerdict,
};

/// A value only the first program's law admits.
pub const LOW_RETENTION: u64 = 3;
/// A value only the second program's law admits.
pub const HIGH_RETENTION: u64 = 15;
/// A second value only the first program's law admits, used on a fork.
pub const FORK_RETENTION: u64 = 9;

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

pub fn validated_first() -> ValidatedApplicationProgram<DocumentRetentionSchema, RetentionProgramP0>
{
    ApplicationProgramAuthoring::<DocumentRetentionSchema, RetentionProgramP0>::begin()
        .validated_program()
        .expect("the first program is authored validly")
}

pub fn validated_second() -> ValidatedApplicationProgram<DocumentRetentionSchema, RetentionProgramP1>
{
    ApplicationProgramAuthoring::<DocumentRetentionSchema, RetentionProgramP1>::begin()
        .validated_program()
        .expect("the second program is authored validly")
}
