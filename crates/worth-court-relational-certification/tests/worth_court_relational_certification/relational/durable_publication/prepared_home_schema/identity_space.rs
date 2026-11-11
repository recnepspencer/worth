//! Whole original-catalog DATA reserves identities beyond the installation.
use super::*;
use worth_foundational::facade::{aspects, AspectIdentity, AspectKey, FieldKey, ScalarAspectType};
use worth_relational::facade::{identity::KindId, schema::*};

#[test]
fn ordinary_schema_transition_refuses_new_roles_without_home_program_authority() {
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let (source, medium) = initial_input_source::initialized(execution);
    let original = source.config().schema.registry.authority_snapshot();
    let mut target = RelationalSchemaRegistry::new();
    for kind in &original.entity_kinds {
        let mut entry = source
            .config()
            .schema
            .registry
            .entity_registration(kind.kind_id)
            .unwrap()
            .clone();
        entry.schema_version_id = SchemaVersionId(2);
        target = target.register_entity_kind(entry).unwrap();
    }
    for kind in &original.relation_kinds {
        let mut entry = source
            .config()
            .schema
            .registry
            .relation_registration(kind.kind_id)
            .unwrap()
            .clone();
        entry.schema_version_id = SchemaVersionId(2);
        target = target.register_relation_kind(entry).unwrap();
    }
    let contract = aspects()
        .contract()
        .for_key(AspectKey::new("reserved").unwrap())
        .identified_by(AspectIdentity(3000))
        .at_revision(aspects().vocabulary().revision(1))
        .struct_aspect(
            aspects()
                .struct_fields()
                .required("value", ScalarAspectType::UInt64)
                .finish()
                .unwrap(),
        );
    target = target
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(2000),
            kind_name: "later-role".into(),
            schema_id: SchemaId("test".into()),
            schema_version_id: SchemaVersionId(2),
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
    let (_, basis) = source
        .owner_component_services()
        .basis_port()
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let transition = ProposedSchemaTransition {
        source_schema_id: SchemaId("test".into()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".into()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Schema,
                SchemaId("test".into()),
                SchemaVersionId(2),
                None,
                "later-identity-reservation",
            ),
            vec![SchemaStratum::PublicationContract],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::FreeText {
                detail: "publish the next declared basis with one additional empty role".into(),
                declared_intent: FreeFormSchemaDiffIntent::Additive,
            },
        )],
    };
    let offered = medium.offered().len();
    let before = current(execution, &source);
    let refused = match source.begin_branch_schema_transition(&basis, transition, None, target) {
        Err(refused) => refused,
        Ok(_) => {
            panic!("ordinary Native schema admission must not grant changed Home execution meaning")
        }
    };
    assert_eq!(refused.kind(), worth_relational::facade::runtime::RelationalSchemaTransitionAdmissionDenialKind::TargetContractMismatch);
    assert_eq!(current(execution, &source), before);
    assert_eq!(medium.offered().len(), offered);
    drop(basis);
    let receiver = durable_runtime(&medium);
    let prepared = receiver
        .durability()
        .prepare_home_recovery_source(
            execution,
            initial_input_source::journal_home_input(&medium, execution).unwrap(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    assert_eq!(
        prepared
            .initial_schema_data(execution)
            .unwrap()
            .authority_snapshot(),
        original
    );
    assert_eq!(
        prepared.occupied_identity_maxima(execution).unwrap().0,
        Some(KindId(901))
    );
    assert_eq!(medium.offered().len(), offered);
    assert!(
        receiver.history().historical_latest_commit().is_none(),
        "projection does not replay or expose a readmitted receiver"
    );
    drop(prepared);
    receiver.prepare_close().unwrap().finish();
    source.prepare_close().unwrap().finish();
}
