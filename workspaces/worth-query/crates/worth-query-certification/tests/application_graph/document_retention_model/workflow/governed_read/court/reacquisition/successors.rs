//! A P0 source's intent remains its own when a current P1 successor is admitted.
use super::*;
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowDefinitionBuilder;

#[test]
fn carried_source_and_p1_migration_or_fork_successor_have_distinct_native_intents() {
    for forked in [false, true] {
        let mut application = install();
        support_successor(&mut application);
        let p0 = *application.program_runtime().installed_program().revision();
        let p1 = *application
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .installed_program()
            .revision();
        assert_ne!(p0, p1);
        let first = publish(&application);
        let prepared = prepare(&application, first.clone(), 92_720);
        let source_intent = prepared.discovery_intent();
        let source = started(prepared.execute());
        let source_id = source.instance().entity_id();
        drop(source);
        // Publish the replacement before adoption; Carry legitimately carries
        // its current meaning to P1 before native successor preparation.
        let mut builder = ApplicationWorkflowDefinitionBuilder::<GovernedDocumentWorkflow>::new(
            "governed-original-intent",
            workflow::definition_limits(),
        )
        .unwrap();
        let resumed = builder.terminal("resumed").unwrap();
        builder.start(&resumed);
        let target = publication::publish_definition(
            &application,
            builder.finish().unwrap().validate().unwrap(),
            WorkflowDefinitionExpectedPredecessor::Published(first),
            92_721,
        );
        carry(&application, &p1);
        let runtime = application.runtime();
        let scope = request_scope();
        let operator = authenticate_operator(runtime.installed_schema(), &scope);
        let reviewer = authenticate_reviewer(runtime.installed_schema(), &scope);
        let before = runtime
            .request(&reviewer, &scope)
            .retain_current_read()
            .unwrap();
        let source = live(read(&application, &reviewer, &scope, &before, &source_intent).unwrap());
        assert_eq!(source.entity_id(), source_id);
        assert_eq!(source.program_revision(), &p0);
        let source_branch = source.branch();
        let branch = if forked {
            application
                .with_host_advancement(|phase| {
                    runtime
                        .branches()
                        .fork(source_branch)
                        .components(|components| {
                            components.fork_relational().reuse_exact_signal_basis()
                        })
                        .create(&phase)
                        .unwrap()
                })
                .unwrap()
        } else {
            source_branch
        };
        if forked {
            let copied_basis = runtime
                .request(&reviewer, &scope)
                .on_branch(branch)
                .retain_current_read()
                .unwrap();
            let copied = runtime
                .request(&reviewer, &scope)
                .on_branch(branch)
                .at(&copied_basis)
                .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                    application.vocabulary(),
                    &source_intent,
                    input(),
                    input(),
                )
                .unwrap();
            assert!(
                matches!(
                    copied,
                    WorthQueryWorkflowInstanceRead::Historical {
                        posture: WorthQueryWorkflowHistoricalPosture::Fork,
                        ..
                    }
                ),
                "copied truth yields no executable live locator"
            );
        }
        let mutation = runtime
            .request(&operator, &scope)
            .on_branch(branch)
            .mutate(WorkflowInstanceStartIntent {
                input: WorkflowInstanceStartInput {
                    document_identity: DOCUMENT_IDENTITY.to_owned(),
                },
            })
            .without_source()
            .idempotency(&92_722_u64);
        let prepared = if forked {
            mutation.prepare_workflow_fork_continuation(
                application.vocabulary(),
                source,
                target,
                "resumed",
            )
        } else {
            mutation.prepare_workflow_instance_migration(
                application.vocabulary(),
                source,
                target,
                "resumed",
            )
        }
        .unwrap();
        let successor_intent = prepared.discovery_intent();
        assert_ne!(source_intent, successor_intent);
        let performed = started(prepared.execute());
        let successor_id = performed.instance().entity_id();
        assert_ne!(source_id, successor_id);
        drop(performed);
        let after = runtime
            .request(&reviewer, &scope)
            .on_branch(branch)
            .retain_current_read()
            .unwrap();
        let source_view = runtime
            .request(&reviewer, &scope)
            .on_branch(branch)
            .at(&after)
            .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                application.vocabulary(),
                &source_intent,
                input(),
                input(),
            )
            .unwrap();
        assert!(
            matches!(source_view, WorthQueryWorkflowInstanceRead::Historical { observation, .. } if observation.state() == WorthQueryWorkflowObservedState::Migrated)
        );
        let successor = live(
            runtime
                .request(&reviewer, &scope)
                .on_branch(branch)
                .at(&after)
                .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                    application.vocabulary(),
                    &successor_intent,
                    input(),
                    input(),
                )
                .unwrap(),
        );
        assert_eq!(successor.entity_id(), successor_id);
        assert_eq!(successor.program_revision(), &p1);
        assert_eq!(successor.start_node_path(), "resumed");
        // Retained source truth cannot become the successor or silently use head.
        let retained_source =
            read(&application, &reviewer, &scope, &before, &source_intent).unwrap();
        match retained_source {
            WorthQueryWorkflowInstanceRead::Live {
                instance,
                observation,
            } if forked => {
                assert_eq!(instance.entity_id(), source_id);
                assert_eq!(observation.state(), WorthQueryWorkflowObservedState::Ready);
                assert_eq!(observation.selected_commit(), before.selected_commit());
            }
            WorthQueryWorkflowInstanceRead::Historical { observation, .. } => {
                assert_eq!(observation.state(), WorthQueryWorkflowObservedState::Ready);
                assert_eq!(observation.selected_commit(), before.selected_commit());
            }
            _ => panic!("retained source basis must preserve its original Ready truth"),
        }
        if forked {
            let main = runtime
                .request(&reviewer, &scope)
                .on_branch(source_branch)
                .retain_current_read()
                .unwrap();
            assert_eq!(
                live(read(&application, &reviewer, &scope, &main, &source_intent).unwrap())
                    .entity_id(),
                source_id
            );
        }
        assert!(matches!(
            advance(&application, successor, 92_723),
            WorkflowProgressOutcome::Completed(_)
        ));
    }
}
