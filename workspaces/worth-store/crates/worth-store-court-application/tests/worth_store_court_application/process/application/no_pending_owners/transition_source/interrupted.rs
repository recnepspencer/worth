//! A closed A Home with one entity, then a committed one-step intent whose
//! durable target step adds a role. The binding never happens: interrupted.
use super::*;
use worth_foundational::facade::{
    aspects, AspectIdentity, AspectKey, FieldKey, HomeProgramTransitionPlanDigest,
    HomeProgramTransitionStepAssociation, ScalarAspectType,
};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    durability::{ReadmittedRelationalHomeProgramCatalogs, RelationalHomeProgramPlanningRoot},
    identity::KindId,
    mvcc::{RelationalCallerAssociation, RelationalTransactionIntent},
    runtime::{
        home_program_transition_body_digest, PreparedRelationalHomeProgramContracts,
        RelationalHomeProgramStepContracts, RelationalRuntime,
    },
    schema::*,
};
use worth_runtime_world::facade::{RuntimeWorldCancellationSource, WorldOpenAttemptOutcome};
use worth_store_application::ApplicationHomeTransitionIntentOutcome;
use worth_store_testkit_application_oracle::world_home::WorldHomeFixture;

pub(in super::super) struct Interrupted {
    /// The predecessor's schema, which the durable target step replaced.
    pub(in super::super) initial: SchemaAuthoritySnapshot,
    /// Rows the predecessor held when the intent recorded it.
    pub(in super::super) predecessor_rows: usize,
}

/// The successor registry: the source roles at TARGET plus one later role.
pub(in super::super) fn target_registry() -> RelationalSchemaRegistry {
    let contract = aspects()
        .contract()
        .for_key(AspectKey::new("reserved").unwrap())
        .identified_by(AspectIdentity(u64::MAX))
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(
            aspects()
                .struct_fields()
                .required("value", ScalarAspectType::UInt64)
                .finish()
                .unwrap(),
        );
    crate::schema_transition::registry(crate::schema_transition::TARGET)
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(2000),
            kind_name: "later-role".into(),
            schema_id: SchemaId("oracle".into()),
            schema_version_id: crate::schema_transition::TARGET,
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                DeclaredAspectContractBinding {
                    binding: AspectBinding::EntityField {
                        field: FieldKey::new("reserved").unwrap(),
                    },
                    contract,
                },
            ]),
        })
        .unwrap()
}

fn proposed() -> ProposedSchemaTransition {
    let oracle = || SchemaId("oracle".into());
    ProposedSchemaTransition {
        source_schema_id: oracle(),
        source_schema_version_id: crate::schema_transition::SOURCE,
        target_schema_id: oracle(),
        target_schema_version_id: crate::schema_transition::TARGET,
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::EntityKind,
                oracle(),
                crate::schema_transition::TARGET,
                Some(KindId(2000)),
                "later-role",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::FreeText {
                detail: "publish the next basis with an additional empty kind and its aspect \
                         contract"
                    .into(),
                declared_intent: FreeFormSchemaDiffIntent::Additive,
            },
        )],
    }
}

/// Entities of the one kind both schemas share, as `basis` observes them.
pub(in super::super) fn rows(
    native: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    execution: &ExecutionRequest<'_, '_>,
) -> usize {
    native
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap()
        .bounded_entities_of_kind(execution, KindId(1), 10)
        .unwrap()
        .unwrap()
        .records()
        .len()
}

pub(in super::super) fn home(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> Interrupted {
    {
        // The close checkpoint selects the image the planning root is read from.
        let predecessor = open(location, execution);
        let root = predecessor.bootstrap_root(execution);
        let cancellation = RuntimeWorldCancellationSource::new();
        let prepared = predecessor.prepare_publication(
            execution,
            root.product_branch().clone(),
            crate::batches::create(0),
            &cancellation.token(),
        );
        drop(
            predecessor
                .publish(execution, prepared, &cancellation.token())
                .consume(),
        );
        drop(root);
        close(predecessor.home, execution);
    }
    let mut fixture = worth_store_testkit_application_oracle::world_home::open(
        super::super::transition_opening::successor_request(location),
        execution,
    )
    .expect("the same Home readmits the genuine declared predecessor before transition");
    let source = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native = fixture.home.relational().runtime();
    let interrupted = Interrupted {
        initial: native.config().schema.registry.authority_snapshot(),
        predecessor_rows: rows(native, source.basis().relational_basis(), execution),
    };
    let intent = RelationalTransactionIntent::ordinary().with_schema_transition(proposed(), None);
    let batch = crate::batches::create(1);
    let bodies =
        [
            home_program_transition_body_digest(execution, &intent, std::slice::from_ref(&batch))
                .unwrap(),
        ];
    let contracts = native
        .prepare_home_program_contracts(
            execution,
            source.basis().relational_basis(),
            target_registry(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
    let reserved = fixture
        .home
        .reserve_home_program_transition_plan(
            source.clone(),
            HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
            1,
            contracts.target_meaning(execution, 0).unwrap(),
            &bodies,
            execution,
        )
        .unwrap();
    fixture
        .home
        .stage_home_program_transition_intent(
            reserved,
            worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(),
            execution,
        )
        .unwrap();
    loop {
        execution.consult().unwrap();
        match fixture
            .home
            .advance_home_program_transition_intent(execution)
            .unwrap()
        {
            ApplicationHomeTransitionIntentOutcome::Committed => break,
            ApplicationHomeTransitionIntentOutcome::Pending => std::thread::yield_now(),
            other => panic!("actual Home intent refused: {other:?}"),
        }
    }
    let mut grant = fixture
        .home
        .admit_home_program_transition_step(0, &source, execution)
        .unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let reservation = grant.take_reservation().unwrap();
    let pending = fixture
        .home
        .world()
        .publication_port()
        .prepare_home_transition_step(
            execution,
            source.clone(),
            fixture.admit_initial_program_source(&source),
            intent,
            grant.plan(),
            grant.original(),
            reservation,
            grant.admission(),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let mut outcome = pending.advance(execution);
    let opened = loop {
        execution.consult().unwrap();
        match outcome {
            WorldOpenAttemptOutcome::Opened(opened) => break opened,
            WorldOpenAttemptOutcome::Pending(pending) => outcome = pending.advance(execution),
            other => panic!("assigned Home open refused: {other:?}"),
        }
    };
    let association =
        RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
    let native = fixture.home.relational().runtime();
    let mut transaction = native
        .begin_home_program_step(
            execution,
            opened.expected_head().basis().relational_basis(),
            RelationalHomeProgramStepContracts::Target {
                contracts: &contracts,
                transition: Some((proposed(), None)),
            },
            grant.plan(),
            HomeProgramTransitionStepAssociation::from_recorded(grant.plan().identity(), 0),
            grant.original(),
            &association,
            grant.admission(),
            worth_relational::facade::indexes::DerivedIndexMaintenanceBudget {
                maximum_work_units: None,
                maximum_cold_record_slots: None,
                maximum_derived_rows: None,
            },
        )
        .unwrap();
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction.validate(execution, native).unwrap();
    let prepared = opened.prepare_native(native, execution, proposal).unwrap();
    let performed = fixture.publish(execution, prepared, &cancellation.token());
    assert_eq!(
        performed.new_product_head().reference_generation().get(),
        source.reference_generation().get() + 1
    );
    assert!(
        native
            .config()
            .schema
            .registry
            .entity_registration(KindId(2000))
            .is_err(),
        "construction configuration is not a target publication"
    );
    drop(performed.consume());
    // No binding: the original durable owner custody is abandoned here.
    drop((grant, contracts, source, fixture));
    interrupted
}

/// Reopen as the resumed transition does: readmit the planning root once,
/// compile the successor on it, then readmit the catalogs on the same root.
pub(in super::super) struct Resumed {
    pub(in super::super) root: RelationalHomeProgramPlanningRoot,
    pub(in super::super) catalogs: ReadmittedRelationalHomeProgramCatalogs,
    pub(in super::super) contracts: PreparedRelationalHomeProgramContracts,
}
pub(in super::super) fn resume(
    fixture: &WorldHomeFixture,
    execution: &ExecutionRequest<'_, '_>,
) -> Resumed {
    let native = fixture.home.relational().runtime();
    let authority = fixture
        .home
        .relational()
        .world_recovery_authority()
        .unwrap();
    let root = authority
        .readmit_home_program_planning_root(execution, native)
        .unwrap();
    let contracts = native
        .prepare_home_program_contracts(
            execution,
            root.basis(),
            target_registry(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
    let grant = fixture
        .home
        .admit_home_program_transition_catalog(execution)
        .unwrap();
    let catalogs = authority
        .readmit_home_program_catalogs(execution, native, &grant, &root, &contracts)
        .unwrap();
    drop(grant);
    Resumed {
        root,
        catalogs,
        contracts,
    }
}
