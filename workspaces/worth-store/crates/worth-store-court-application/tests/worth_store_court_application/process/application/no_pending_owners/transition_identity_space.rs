//! Original schema roles remain reserved through the real Home publication route.
use super::*;
use worth_foundational::facade::{
    aspects, AspectIdentity, AspectKey, FieldKey, HomeProgramTransitionPlanDigest,
    HomeProgramTransitionStepAssociation, ScalarAspectType,
};
use worth_relational::facade::{
    identity::KindId,
    mvcc::{RelationalCallerAssociation, RelationalTransactionIntent},
    runtime::{home_program_transition_body_digest, RelationalHomeProgramStepContracts},
    schema::*,
};
use worth_runtime_world::facade::{RuntimeWorldCancellationSource, WorldOpenAttemptOutcome};
use worth_store_application::{
    ApplicationHomeNativeSchemaPreparation, ApplicationHomeTransitionIntentOutcome,
};

#[test]
fn prepared_identity_bounds_include_later_schema_roles_beyond_the_initial_installation() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let initial = {
        let mut fixture = super::transition_opening::open_transition(directory.path(), &execution);
        let source = fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap();
        let initial = fixture
            .home
            .relational()
            .runtime()
            .config()
            .schema
            .registry
            .authority_snapshot();
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
        let target = crate::schema_transition::registry(crate::schema_transition::TARGET)
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
            .unwrap();
        let proposed = ProposedSchemaTransition {
            source_schema_id: SchemaId("oracle".into()), source_schema_version_id: crate::schema_transition::SOURCE,
            target_schema_id: SchemaId("oracle".into()), target_schema_version_id: crate::schema_transition::TARGET,
            diff_atoms: vec![SchemaDiffAtom::new(
                SchemaElementRef::new(SchemaElementKind::EntityKind, SchemaId("oracle".into()), crate::schema_transition::TARGET, Some(KindId(2000)), "later-role"),
                vec![SchemaStratum::StructuralShape, SchemaStratum::PublicationContract], SchemaPublicationImpact::ObservableSurfaceChanged,
                SchemaSubscriberImpact::None, HistoricalInterpretationSensitivity::NotSensitive,
                SchemaDiffDetail::FreeText { detail: "publish the next basis with an additional empty kind and its aspect contract".into(), declared_intent: FreeFormSchemaDiffIntent::Additive },
            )],
        };
        let intent =
            RelationalTransactionIntent::ordinary().with_schema_transition(proposed.clone(), None);
        let batch = crate::batches::create(0);
        let bodies = [home_program_transition_body_digest(
            &execution,
            &intent,
            std::slice::from_ref(&batch),
        )
        .unwrap()];
        let contracts = fixture
            .home
            .relational()
            .runtime()
            .prepare_home_program_contracts(
                &execution,
                source.basis().relational_basis(),
                target,
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
                contracts.target_meaning(&execution, 0).unwrap(),
                &bodies,
                &execution,
            )
            .unwrap();
        fixture.home.stage_home_program_transition_intent(reserved,
            worth_query_testkit_installation::facade::installation::application_home_binding::admitted_transition_successor_binding(), &execution).unwrap();
        loop {
            execution.consult().unwrap();
            match fixture
                .home
                .advance_home_program_transition_intent(&execution)
                .unwrap()
            {
                ApplicationHomeTransitionIntentOutcome::Committed => break,
                ApplicationHomeTransitionIntentOutcome::Pending => std::thread::yield_now(),
                other => panic!("actual Home intent refused: {other:?}"),
            }
        }
        let mut grant = fixture
            .home
            .admit_home_program_transition_step(0, &source, &execution)
            .unwrap();
        let cancellation = RuntimeWorldCancellationSource::new();
        let reservation = grant.take_reservation().unwrap();
        let pending = fixture
            .home
            .world()
            .publication_port()
            .prepare_home_transition_step(
                &execution,
                source,
                intent,
                grant.plan(),
                grant.original(),
                reservation,
                grant.admission(),
                &cancellation.token(),
                None,
            )
            .unwrap();
        let mut outcome = pending.advance(&execution);
        let opened = loop {
            execution.consult().unwrap();
            match outcome {
                WorldOpenAttemptOutcome::Opened(opened) => break opened,
                WorldOpenAttemptOutcome::Pending(pending) => outcome = pending.advance(&execution),
                other => panic!("assigned Home open refused: {other:?}"),
            }
        };
        let association =
            RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
        let native = fixture.home.relational().runtime();
        let mut transaction = native
            .begin_home_program_step(
                &execution,
                opened.expected_head().basis().relational_basis(),
                RelationalHomeProgramStepContracts::Target {
                    contracts: &contracts,
                    transition: Some((proposed, None)),
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
        let proposal = transaction.validate(&execution, native).unwrap();
        let prepared = opened.prepare_native(native, &execution, proposal).unwrap();
        let performed = fixture.publish(&execution, prepared, &cancellation.token());
        assert_eq!(performed.new_product_head().reference_generation().get(), 1);
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
        drop((grant, contracts, fixture));
        initial
    };
    let input = super::transition_opening::successor_request(directory.path());
    let admitted = worth_store_application::admit_home(
        worth_store_application::recover_home(input, &execution).unwrap(),
        &execution,
    )
    .unwrap();
    let mut home = worth_store_application::attach_relational(admitted, &execution)
        .unwrap_or_else(|_| panic!("actual same Home Native attachment"));
    {
        let ApplicationHomeNativeSchemaPreparation::Existing {
            native,
            source,
            maximum_kind,
            maximum_aspect,
            installation,
        } = home.prepare_native_schema_for_home(&execution).unwrap()
        else {
            panic!("same directory cannot initialize");
        };
        assert!(
            installation.is_none(),
            "a pending unbound transition grants no successor installation"
        );
        assert_eq!(source.authority_snapshot(), initial);
        assert_eq!(maximum_kind, Some(KindId(2000)));
        assert_eq!(maximum_aspect, Some(AspectIdentity(u64::MAX)));
        assert!(
            native.history().historical_latest_commit().is_none(),
            "projection does not replay or expose a readmitted receiver"
        );
    }
    // No transition continuation or final binding is claimed. The original
    // durable owner custody remains in the actual Home, then is abandoned.
    drop(home);
}
