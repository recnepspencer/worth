use super::*;
#[test]
fn query_local_read_ceiling_keeps_the_ordinary_denial_and_admitted_work() {
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let selected = world.selected_product();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let root = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
    let query = world
        .application
        .installed_schema()
        .certification_query(PublicScopedAccountSummaryQuery::reference())
        .unwrap();
    let admit = |work| {
        selected
            .retain_selection()
            .unwrap()
            .admit_application_query(
                &query,
                &access,
                ApplicationQueryParameterSet::new(),
                WorthQueryProductQueryControls::new(
                    NonZeroUsize::new(10).unwrap(),
                    NonZeroUsize::new(work).unwrap(),
                    &scope,
                ),
            )
            .unwrap()
    };
    let ordinary = world
        .application
        .execute_application_query_one_shot(admit(3))
        .err()
        .expect("the three-unit read cannot materialize its two fields");
    let fields = query
        .read_family_binding()
        .planning_contract()
        .projections()
        .len() as u64;
    let first_read = 1 + (1 + fields) + (1 + fields);
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(16 << 20),
        scope.cancellation().execution_token(),
        Some(scope.deadline()),
    );
    let mut plans = WorthQueryDerivedPairReadPlans::new(admit(100000), admit(3));
    ExecutionRequest::serial(&serial)
        .in_scope(|lease| {
            owner_stage::run(lease, 8192, |context| {
                plans.admit(&world.application, root.entity_id(), context)
            })
        })
        .unwrap()
        .unwrap();
    let map = ExecutionMap::<_, ()>::from_keyless_partitions(BTreeMap::from([(
        PartitionIdentity::new(1),
        KeylessPartition {
            value: plans.worker(&world.application, root.entity_id()).unwrap(),
            kernel_scratch_bytes: 0,
            max_result_bytes: plans.result_capacity().unwrap(),
        },
    )]))
    .unwrap();
    let (outcome, _) = ExecutionWorkCeiling::new(u64::MAX)
        .run_serial(&serial, || {
            map.run_owned(None, |pair, context| pair.run(context))
        })
        .unwrap();
    match outcome {
        MapOutcome::Stopped {
            completed_prefix,
            reason,
            report,
            ..
        } => {
            assert!(completed_prefix.is_empty());
            assert_eq!(
                reason,
                MapStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause: worth_execution::MapKernelFailure::Domain(Denial::ReadDenied {
                        root: root.entity_id(),
                        denial: ordinary
                    })
                }
            );
            // The second selection's one unit is admitted; its three-unit
            // projection is refused by the existing Query-local limit.
            assert_eq!(report.charged_work(), 1 + first_read + 1);
        }
        MapOutcome::Complete { .. } => panic!("the Query-local read must deny"),
    }
    drop(serial.memory().reserve(serial.memory().limit()).unwrap());
}
