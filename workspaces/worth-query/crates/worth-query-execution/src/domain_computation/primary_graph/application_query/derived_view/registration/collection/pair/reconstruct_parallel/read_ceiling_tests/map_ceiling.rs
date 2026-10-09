use super::*;
#[test]
fn map_one_short_at_bounded_lookup_refuses_the_whole_increment() {
    let world = installed_authorization_world(true);
    let scope = live_scope();
    let old = world.selected_product();
    let unrelated = old
        .resolve_entity(
            AccountStatus::reference(),
            "unrelated".to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let field = AccountStatus::reference();
    let locator = graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    drop(old);
    // The declared edit makes exactly two indexed roots match "open".
    publish_relational_mutation(
        &world,
        WorkerIntentBatch::new("two-lookup-candidates").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: unrelated.entity_id(),
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    locator,
                    StringApplicationValueBinding::encode(&"open".to_owned()).unwrap(),
                )])),
            }),
        )),
    );
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
            AccountLabel::reference(),
            "primary".to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
    let first_query = world
        .application
        .installed_schema()
        .certification_query(PublicScopedAccountSummaryQuery::reference())
        .unwrap();
    let second_query = world
        .application
        .installed_schema()
        .certification_query(NestedAccountQuery::reference())
        .unwrap();
    let first = selected
        .retain_selection()
        .unwrap()
        .admit_application_query(
            &first_query,
            &access,
            ApplicationQueryParameterSet::new(),
            current_controls(&scope),
        )
        .unwrap();
    let second = selected
        .retain_selection()
        .unwrap()
        .admit_application_query(
            &second_query,
            &access,
            ApplicationQueryParameterSet::new()
                .bind(status_parameter(), "open".to_owned())
                .unwrap(),
            current_controls(&scope),
        )
        .unwrap();
    let fields = first_query
        .read_family_binding()
        .planning_contract()
        .projections()
        .len() as u64;
    // One selected root; one projected record plus each field; one source
    // entity plus each field. No predicate or relation in this first read.
    let first_read = 1 + (1 + fields) + (1 + fields);
    let before_lookup = 1 + first_read; // the pair invocation
    let indexed_inputs =
        std::collections::BTreeSet::from([root.entity_id(), unrelated.entity_id()]);
    let lookup = (indexed_inputs.len() as u64).min(2); // declared candidate limit two
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(16 << 20),
        scope.cancellation().execution_token(),
        Some(scope.deadline()),
    );
    let mut plans = WorthQueryDerivedPairReadPlans::new(first, second);
    ExecutionRequest::serial(&serial)
        .in_scope(|lease| {
            owner_stage::run(lease, |context| {
                plans.admit(&world.application, root.entity_id(), context)
            })
        })
        .unwrap()
        .unwrap();
    let capacity = plans.result_capacity().unwrap();
    let input = plans.worker(&world.application, root.entity_id()).unwrap();
    let map = ExecutionMap::<_, ()>::from_keyless_partitions(BTreeMap::from([(
        PartitionIdentity::new(1),
        KeylessPartition {
            value: input,
            kernel_scratch_bytes: 0,
            max_result_bytes: capacity,
        },
    )]))
    .unwrap();
    let (outcome, _) = ExecutionWorkCeiling::new(before_lookup + lookup - 1)
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
                MapStop::WorkExhausted {
                    identity: PartitionIdentity::new(1)
                }
            );
            assert_eq!(
                report.charged_work(),
                before_lookup,
                "the refused lookup increment charges no units"
            );
        }
        MapOutcome::Complete { .. } => panic!("one-short lookup must stop"),
    }
    drop(serial.memory().reserve(serial.memory().limit()).unwrap());
}
