//! Each installed commit family checks the applied state once; publication carries it.
use std::sync::{Arc, Mutex};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{runtime::*, transactions::*};

struct Rule {
    point: InvariantExecutionPoint,
    calls: Arc<Mutex<Vec<InvariantExecutionPoint>>>,
}
impl CustomInvariantRule for Rule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(format!(
                    "court.proposed.{}",
                    self.point.diagnostic_label()
                )),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("proposed state"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(10000),
                access: CustomInvariantAccessContract::default(),
                execution_point: self.point,
                groups: InvariantGroupSet::all(),
                cost_class: InvariantCostClass::Touched,
                failure_effect: if self.point == InvariantExecutionPoint::SnapshotPublication {
                    InvariantFailureEffect::BlockPublication
                } else {
                    InvariantFailureEffect::BlockCommit
                },
            },
        }
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.calls.lock().unwrap().push(self.point);
        Ok(CustomInvariantVerdict::Pass)
    }
}

#[test]
fn every_commit_family_evaluates_once_and_publication_reuses_the_result() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let points = [
        InvariantExecutionPoint::CommitBoundary,
        InvariantExecutionPoint::MutationSensitive,
        InvariantExecutionPoint::SnapshotPublication,
    ];
    let registry = worth_relational::facade::schema::RelationalSchemaRegistry::new()
        .register_entity_kind(worth_relational::facade::schema::EntityKindRegistration {
            kind_id: worth_relational::facade::identity::KindId(9),
            kind_name: "proposal".into(),
            schema_id: worth_relational::facade::schema::SchemaId("proposal".into()),
            schema_version_id: worth_relational::facade::schema::SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    let mut catalog = InvariantCatalog::default();
    catalog
        .registrations
        .push(InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(
                worth_foundational::facade::AspectKey::new("name").unwrap(),
                worth_foundational::facade::FieldKey::new("value").unwrap(),
            ),
        ));
    let mut builder = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .invariant_catalog(catalog);
    for point in points {
        builder = builder.custom_invariant(
            CustomInvariantRegistration::new(Rule {
                point,
                calls: calls.clone(),
            })
            .unwrap(),
        );
    }
    let medium = worth_testkit_relational::durable_medium::TestDurableMedium::new();
    let runtime = builder.build_durable(medium.attachment()).unwrap();
    let predecessor = runtime.current_version_id();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut tx = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("one-state").push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: worth_relational::facade::identity::PartitionId(7),
                kind_id: worth_relational::facade::identity::KindId(9),
                client_key: worth_relational::facade::symbols::ClientKey::raw("created"),
                fields: Default::default(),
            },
        ))),
        execution.allocation_policy(),
    )
    .unwrap();
    let candidate = runtime.prepare_branch_transaction(&execution, tx).unwrap();
    let mut before = calls.lock().unwrap().clone();
    before.sort();
    let mut expected = points.to_vec();
    expected.sort();
    assert_eq!(
        before, expected,
        "each declared family must check this exact proposal once"
    );
    let worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime
            .publication_port()
            .compare_and_publish(&execution, candidate)
    else {
        panic!("accepted state must publish")
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let semantic = committed.semantic_result();
    assert_eq!(
        semantic.validation().len(),
        1,
        "one actual aggregate is recorded"
    );
    let meaning = &semantic.validation()[0];
    let live = &committed.validation().invariant_executions()[0];
    assert_eq!(
        meaning.metadata.execution_point,
        InvariantExecutionPoint::ProposedState
    );
    assert_eq!(meaning.metadata.version, live.metadata().version_id());
    assert_eq!(
        meaning.metadata.current_version,
        live.metadata().current_version_id()
    );
    assert_eq!(meaning.metadata.version, committed.commit.version_id);
    assert_eq!(meaning.metadata.current_version, predecessor);
    assert_eq!(
        serde_json::to_value(meaning.metadata.observation_kind).unwrap(),
        "Committed"
    );
    for point in points {
        let check = meaning
            .checks
            .iter()
            .find(|check| check.execution_point == point && check.custom_provenance.is_some())
            .unwrap();
        let provenance = check.custom_provenance.as_ref().unwrap();
        let observation = if point == InvariantExecutionPoint::CommitBoundary {
            "Committed"
        } else {
            assert!(matches!(
                point,
                InvariantExecutionPoint::MutationSensitive
                    | InvariantExecutionPoint::SnapshotPublication
            ));
            "Speculative"
        };
        assert_eq!(
            serde_json::to_value(provenance.observation_kind).unwrap(),
            observation
        );
        assert_eq!(provenance.version, committed.commit.version_id);
        assert_eq!(provenance.current_version, predecessor);
    }
    let artifact = medium
        .offered()
        .pop()
        .expect("actual durable original frame");
    let readmitted =
        worth_relational::facade::durability::RelationalCanonicalCommitArtifact::readmit(
            artifact.as_bytes(),
            &execution,
        )
        .unwrap();
    assert_eq!(readmitted.envelope().semantic_result(), semantic);

    let mut after = calls.lock().unwrap().clone();
    after.sort();
    assert_eq!(
        after, before,
        "publishing unchanged state must carry its proof"
    );
    assert!(committed.validation().summary().commit_boundary_seen);
    assert!(committed.validation().summary().mutation_sensitive_seen);
    assert!(committed.validation().summary().snapshot_publication_seen);
    let decisions = committed
        .validation()
        .invariant_executions()
        .iter()
        .flat_map(|execution| execution.decision_log())
        .collect::<Vec<_>>();
    assert!(decisions
        .iter()
        .any(|row| matches!(row.rule, InvariantReportedRule::Native(_))));
    for (index, row) in decisions.iter().enumerate() {
        assert!(
            !decisions[..index].contains(row),
            "a rule/witness was evaluated twice: {row:?}"
        );
    }
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
}

#[test]
fn aggregate_reporting_point_cannot_silently_register_a_rule_that_never_runs() {
    assert!(matches!(
        CustomInvariantRegistration::new(Rule {
            point: InvariantExecutionPoint::ProposedState,
            calls: Default::default(),
        }),
        Err(CustomInvariantRegistrationError::AggregateExecutionPoint)
    ));
    let mut registration =
        InvariantRegistration::commit_boundary_blocking(InvariantRule::MaxMergedIntents(1));
    registration.execution_point = InvariantExecutionPoint::ProposedState;
    let result = RelationalRuntimeApi::builder()
        .invariant_catalog(InvariantCatalog {
            registrations: vec![registration],
        })
        .build();
    assert!(matches!(
        result,
        Err(
            RelationalRuntimeConstructionDenial::InvariantExecutionPoint {
                execution_point: InvariantExecutionPoint::ProposedState,
                ..
            }
        )
    ));
}
