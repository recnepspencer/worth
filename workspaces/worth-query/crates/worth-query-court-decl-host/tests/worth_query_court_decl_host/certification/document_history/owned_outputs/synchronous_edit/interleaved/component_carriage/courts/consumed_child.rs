//! #238 (PR C C4): the late link changes the incoming selection of a child an
//! earlier stage already consumed. Qualified use compares that selection's
//! support at the link's append, ends it there, and the child selects its
//! parents again and derives again; the edit commits the world one commit per
//! stage reaches, built here first.
use super::*;

/// The values both worlds are compared on: the derived outputs the stages
/// read or create (absent ones as `None`), and the observation and link
/// stages' results.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct World {
    pub(super) first_a: Option<u64>,
    pub(super) child: Option<u64>,
    pub(super) observed: u64,
    pub(super) attached: u64,
}

pub(super) fn outputs(request: &family::Request<'_, '_, '_>) -> (Option<u64>, Option<u64>) {
    let first = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .ok()
        .and_then(|rows| rows.rows().first().map(|row| row.value));
    let child = request
        .query(query::child_output::Read("from-first:first:a".into()))
        .execute()
        .ok()
        .and_then(|rows| rows.rows().first().map(|row| row.value));
    (first, child)
}

/// The same four stages, each its own commit, on a fresh world.
pub(super) fn sequential() -> World {
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        1,
        None,
        world::Configuration {
            envelope_records: true,
            interleaved_records: true,
            ..Default::default()
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let edit = |ordinal: usize| {
        request
            .edit_envelope()
            .idempotency(format!("pending-child-sequential:{ordinal}"))
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap()
    };
    let commit = |outcome: Outcome| {
        let Outcome::Committed(receipt) = outcome else {
            panic!("each sequential stage commits: {outcome:?}")
        };
        receipt
    };
    let mut model = edit(0);
    model
        .push_stage::<Tree, _>(fixture::model::Intent {
            key: "a".into(),
            value: 3,
            key_derived_target: false,
        })
        .unwrap();
    commit(model.execute_performed(ExecutionAllocationPolicy::SystemAllocation).unwrap());
    let mut after = edit(1);
    after
        .push_stage::<AfterTree, _>(fixture::after_producer::Intent {
            key: "envelope-cap".into(),
            value: 1,
            key_derived_target: false,
        })
        .unwrap();
    commit(after.execute_performed(ExecutionAllocationPolicy::SystemAllocation).unwrap());
    let mut observation = edit(2);
    let observed = observation
        .push_stage::<ApplicationNoOutputGraph, _>(value_observation::Intent {
            model: "a".into(),
            source: value_observation::ValueSource::Child("from-first:first:a".into()),
        })
        .unwrap();
    let observed = commit(
        observation
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
    )
    .result(&observed)
    .unwrap()
    .value;
    let mut link = edit(3);
    let attached = link
        .push_stage::<ApplicationNoOutputGraph, _>(attach::Intent {
            parent: "a".into(),
            child: "carriage:late".into(),
            pending_child: true,
        })
        .unwrap();
    let receipt = commit(link.execute_performed(ExecutionAllocationPolicy::SystemAllocation).unwrap());
    let attached = receipt.result(&attached).unwrap();
    let (first_a, child) = outputs(&request);
    World {
        first_a,
        child,
        observed,
        attached: attached.value,
    }
}
