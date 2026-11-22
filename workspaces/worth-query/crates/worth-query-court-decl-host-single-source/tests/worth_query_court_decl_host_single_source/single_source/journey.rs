//! Body actions declared by a concrete-schema feature and by each member of
//! a generic family commit through the public Host and read back.
use super::flags::{Archive, FlagFeature, Pin, SetArchived, SetPinned};
use super::program::{install, Host, Program};
use super::read::{ReadTask, TaskRow};
use super::schema::Schema;
use super::tasks::Rename;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

/// Commits one input through the ordinary program request; `key` is its
/// idempotency key.
macro_rules! commit {
    ($host:expr, $input:expr, $key:expr) => {{
        let scope = request_scope();
        let principal = authenticate_operator($host.installed_schema(), &scope);
        let outcome = $host
            .runtime()
            .request(&principal, &scope)
            .advanced()
            .mutate($input)
            .without_source()
            .idempotency(&$key)
            .execute_in_program($host, ExecutionAllocationPolicy::SystemAllocation)
            .expect("the authored body executes");
        assert!(
            matches!(outcome, WorthQueryApplicationMutationOutcome::Committed { .. }),
            "{outcome:?}"
        );
    }};
}

pub(super) fn read(host: &Host, task: u64) -> TaskRow {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let rows = host
        .runtime()
        .request(&principal, &scope)
        .query(ReadTask { task })
        .execute()
        .expect("the task read executes")
        .rows()
        .to_vec();
    assert_eq!(rows.len(), 1, "{rows:?}");
    rows.into_iter().next().unwrap()
}

#[test]
fn a_concrete_schema_body_action_commits_its_authored_body() {
    let host = install().expect("the board installs");
    let rename = Rename {
        task: 1,
        title: "Ship plan".to_owned(),
    };
    commit!(&host, rename, 1_u64);
    assert_eq!(read(&host, 1).title, "Ship plan");
    assert_eq!(read(&host, 2).title, "Review plan");
}

#[test]
fn each_generic_family_member_installs_and_runs_its_own_body_action() {
    let host = install().expect("the board installs");
    commit!(&host, SetArchived { task: 1, value: true }, 2_u64);
    commit!(&host, SetPinned { task: 2, value: true }, 3_u64);
    let (first, second) = (read(&host, 1), read(&host, 2));
    assert_eq!((first.archived, first.pinned), (true, false));
    assert_eq!((second.archived, second.pinned), (false, true));
}

#[test]
fn each_family_member_is_a_distinct_feature_of_the_program() {
    assert_eq!(
        <FlagFeature<Archive> as ApplicationFeature<Schema>>::IDENTITY,
        "single-source.flags.archive.v1"
    );
    assert_eq!(
        <FlagFeature<Pin> as ApplicationFeature<Schema>>::IDENTITY,
        "single-source.flags.pin.v1"
    );
    assert_eq!(
        <Program as ApplicationProgramDefinition<Schema>>::feature_specs().len(),
        3
    );
}
