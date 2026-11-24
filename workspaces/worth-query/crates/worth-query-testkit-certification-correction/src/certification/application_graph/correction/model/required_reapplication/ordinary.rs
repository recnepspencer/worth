//! Ordinary source and calculation co-commit, then reuse their real lineage.
use super::super::calculation;
use super::*;
use worth_relational::facade::inspection::observe_maintenance_work;

pub(super) fn run(worker: bool) {
    let limits_cfg = if worker {
        host::worker_limits()
    } else {
        host::installation_limits()
    };
    let application = host::open_directory_with_outputs(
        host::unique_directory(if worker {
            "ordinary-co-committed-worker"
        } else {
            "ordinary-co-committed"
        }),
        limits_cfg.clone(),
    );
    {
        let scope = request_scope();
        let principal = authenticate_operator(application.installed_schema(), &scope);
        let request = application.request(&principal, &scope);
        let key = "ordinary-co-committed-source".to_owned();
        let (source, writes) = observe_maintenance_work(8, || {
            request
                .advanced()
                .mutate(Edit {
                    key: "numeric".into(),
                    value: Value::Primary(11),
                })
                .without_source()
                .idempotency(&key)
                .execute_performed::<OutputProgram, Root>(
                    &application,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        });
        let WorthQueryApplicationPerformedMutationOutcome::Performed(source) = source else {
            panic!("ordinary source and output must perform together")
        };
        assert_eq!(
            writes.published_rows.len() + writes.omitted_publications,
            1,
            "one actual relational commit across all branches"
        );
        let original = source.receipt().clone();
        assert_eq!(
            read_calculation(&application),
            23,
            "the producer output is present before delivery starts"
        );
        let before = head(&application);
        let ((), delivery) = observe_maintenance_work(8, || {
            let mut started = source
                .start_required_outputs(
                    &application,
                    &request,
                    WorthQueryOutputDemandControls::host_policy(),
                )
                .unwrap_or_else(|_| panic!("actual required source custody"));
            let WorthQueryApplicationProgramOutputProgress::Settled(settled) = started
                .required_output_mut()
                .settle(&application, &request)
                .unwrap()
            else {
                panic!("ordinary member must settle")
            };
            assert_eq!(
                settled.root_producer_contacts_in_this_demand(),
                0,
                "co-committed ordinary output is already current"
            );
            assert!(
                settled.root_receipt().is_some(),
                "the actual ordinary producer retains its receipt"
            );
        });
        assert_eq!(
            delivery.published_rows.len() + delivery.omitted_publications,
            0
        );
        assert_eq!(head(&application), before);
        assert_eq!(read_calculation(&application), 23);
        request
            .corrections()
            .select(&original, limits())
            .expect("the original edit retains correct ordinary required-output lineage");
    }
    let dir = application.checkpoint_and_close_for_reopen();
    let cold = host::open_directory_with_outputs(dir, limits_cfg);
    assert_eq!(read_calculation(&cold), 23);
    let scope = request_scope();
    let principal = authenticate_operator(cold.installed_schema(), &scope);
    let request = cold.request(&principal, &scope);
    let ((), reads) = observe_maintenance_work(8, || {
        let mut demand = request
            .demand(calculation::Demand {
                key: "numeric".into(),
            })
            .start_in_program::<OutputProgram, Root>(&cold)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.settle(&request).unwrap()
        else {
            panic!("restored ordinary lineage settles")
        };
        assert_eq!(
            settled.producer_contacts_in_this_demand(),
            0,
            "cold readiness reuses the recorded ordinary output"
        );
    });
    assert_eq!(reads.published_rows.len() + reads.omitted_publications, 0);
    assert_eq!(
        host::read(&cold),
        [Row {
            operand: None,
            primary: 11,
            secondary: 7
        }]
    );
}
