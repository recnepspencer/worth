//! #233 sequential demand and Query output-availability courts.
use super::*;

/// The model write (a = 3) and the later link of `carriage:late` to `a`, as
/// one positioned edit (`positioned`) or one commit per stage, on a fresh
/// world, and returns that host.
fn link_late(positioned: bool) -> WorthQueryProgramApplicationRuntime<Schema, Program> {
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
    let edits: Vec<Vec<bool>> = if positioned {
        vec![vec![false, true]]
    } else {
        vec![vec![false], vec![true]]
    };
    for (ordinal, stages) in edits.into_iter().enumerate() {
        let mut edit = request
            .edit_envelope()
            .idempotency(format!("link-late:{positioned}:{ordinal}"))
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        for attach in stages {
            if attach {
                edit.push_stage::<ApplicationNoOutputGraph, _>(attach::Intent {
                    parent: "a".into(),
                    child: "carriage:late".into(),
                    pending_child: false,
                })
            } else {
                edit.push_stage::<Tree, _>(fixture::model::Intent {
                    key: "a".into(),
                    value: 3,
                    key_derived_target: false,
                })
            }
            .unwrap();
        }
        let outcome = edit
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    }
    host
}

/// #233 (H1, D2): the positioned edit publishes `first:carriage:late` = 350
/// eagerly. One commit per stage leaves it underived: before demand its keyed
/// read refuses `UnknownEntity` (storage truth: no row is committed). A
/// current demand on the sequential side then derives the same 350.
#[test]
fn a_sequential_later_link_derives_the_positioned_value_on_demand() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let late = |host: &WorthQueryProgramApplicationRuntime<Schema, Program>| {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        host.request(&principal, &scope)
            .query(query::first_output::Read("first:carriage:late".into()))
            .execute()
            .map(|rows| rows.rows().first().map(|row| row.value))
    };
    let positioned = link_late(true);
    assert_eq!(
        late(&positioned).unwrap(),
        Some((5 + 30) * 10),
        "the positioned edit derives the linked consumer eagerly"
    );
    let host = link_late(false);
    let committed = late(&host);
    assert!(
        format!("{committed:?}").contains("UnknownEntity"),
        "one commit per stage leaves the linked consumer underived: {committed:?}"
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut demand = request
        .demand(Demand {
            source: "carriage:late".into(),
            selector: None,
        })
        .start_in_program::<Program, Tree>(&host)
        .unwrap();
    for count in 0..128 {
        assert!(count < 127, "the demand did not settle");
        match demand.settle(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Settled(output) => {
                drop(output);
                break;
            }
            WorthQueryApplicationOutputDemandProgress::Progressed
            | WorthQueryApplicationOutputDemandProgress::Pending => {}
        }
    }
    demand.close();
    let demanded = request
        .query(query::first_output::Read("first:carriage:late".into()))
        .execute()
        .unwrap();
    assert_eq!(
        demanded.rows()[0].value,
        (5 + 30) * 10,
        "a current demand derives the value the positioned edit publishes eagerly"
    );
}

/// One `availability` probe of `carriage:late`, in its own commit, after linking
/// it to `a` in one positioned edit (`eager`) or one commit per stage. `demand`
/// settles a current demand before the probe. (Inside the linking edit the probe
/// would read over the applied candidate, where `output_availability` has no
/// source context and refuses `ContextUnavailable` in either world.)
fn availability_after_link(eager: bool, demand: bool) -> Result<u64, String> {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
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
    let batches: Vec<Vec<u8>> = if eager {
        vec![vec![0, 1], vec![2]]
    } else {
        vec![vec![0], vec![1], vec![2]]
    };
    let mut answer = Err("no probe".to_owned());
    for (ordinal, batch) in batches.into_iter().enumerate() {
        if demand && batch == [2] {
            let mut demand = request
                .demand(Demand {
                    source: "carriage:late".into(),
                    selector: None,
                })
                .start_in_program::<Program, Tree>(&host)
                .unwrap();
            for count in 0..128 {
                assert!(count < 127, "the demand did not settle");
                if let WorthQueryApplicationOutputDemandProgress::Settled(output) =
                    demand.settle(&request).unwrap()
                {
                    drop(output);
                    break;
                }
            }
            demand.close();
        }
        let mut edit = request
            .edit_envelope()
            .idempotency(format!("availability-link:{ordinal}"))
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        let mut probe = None;
        for stage in batch {
            match stage {
                0 => drop(
                    edit.push_stage::<Tree, _>(fixture::model::Intent {
                        key: "a".into(),
                        value: 3,
                        key_derived_target: false,
                    })
                    .unwrap(),
                ),
                1 => drop(
                    edit.push_stage::<ApplicationNoOutputGraph, _>(attach::Intent {
                        parent: "a".into(),
                        child: "carriage:late".into(),
                        pending_child: false,
                    })
                    .unwrap(),
                ),
                _ => {
                    probe = Some(
                        edit.push_stage::<ApplicationNoOutputGraph, _>(availability::Intent {
                            source: "carriage:late".into(),
                        })
                        .unwrap(),
                    )
                }
            }
        }
        match edit
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
        {
            Outcome::Committed(receipt) => {
                if let Some(probe) = probe {
                    answer = Ok(receipt.result(&probe).unwrap().value);
                }
            }
            other => {
                answer = Err(format!("{other:?}"));
                break;
            }
        }
    }
    eprintln!("[233diff] availability eager={eager} demand={demand}: {answer:?}");
    answer
}

/// #233 D2: the typed not-yet-derived answer. Query's output availability gives
/// the eagerly derived linked consumer as `Current` 350, the one a commit per
/// stage leaves underived as `Pending` (where a keyed read refuses
/// `UnknownEntity`), and the sequential side as `Current` 350 after a demand.
#[test]
fn output_availability_answers_pending_for_a_sequentially_underived_linked_consumer() {
    let answers = [
        availability_after_link(true, false),
        availability_after_link(false, false),
        availability_after_link(false, true),
    ];
    assert_eq!(answers, [Ok((5 + 30) * 10), Ok(0), Ok((5 + 30) * 10)]);
}
