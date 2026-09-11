//! Hostile evidence for whole-candidate operation compilation.

use worth_foundational::facade::{
    BoundaryProtocolIdentity, BoundaryProtocolVersion, CanonicalDigestId,
};
use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract,
    DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef, DeclaredPreImageDemand,
    DeclaredPreImageLocus, DeclaredRecordedInverse,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationAspectRef, ApplicationEntityMarkerIdentity,
    ApplicationEntityRef, ApplicationExternalEffectProtocol, ApplicationFieldMarkerIdentity,
    ApplicationFieldPresence, ApplicationFieldRef, ApplicationOperationDecisionReadTarget,
    ApplicationOperationMarkerIdentity, ApplicationOperationProgramTarget, ApplicationOperationRef,
    ApplicationSchema, ApplicationSchemaBindingIdentity, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember, DeclaredApplicationFieldValue,
    RequiredApplicationFieldValue, U64ApplicationValueBinding,
    WorthQueryExternalEffectCorrelationFamily,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

use super::operation_compilation::WorthQueryApplicationOperationCompilation;
use crate::application_operation::{
    WorthQueryApplicationOperationInstallationDenial,
    WorthQueryCompiledApplicationOperationContracts,
};

mod portable_source_seam;

struct Schema;
struct Entity;
struct Aspect;
struct Field;
struct CompilationOperation;
worth_query_declaration::worth_query_structured_value_binding!(
    CompilationInputBinding for () { identity: "worth.rust.unit" }
);

impl ApplicationOperationMarkerIdentity<Schema> for CompilationOperation {
    type InputBinding = CompilationInputBinding;
    const IDENTIFIER: &'static str = "CompilationOperation";
}

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth-query-installation-tests";
    const NAME: &'static str = "OperationCompilationFixture";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(
                ApplicationEntityRef::<Self, Entity>::from_schema_identifier(Entity::IDENTIFIER),
            )
            .aspect(
                ApplicationEntityRef::<Self, Entity>::from_schema_identifier(Entity::IDENTIFIER),
                ApplicationAspectRef::<Self, Entity, Aspect>::from_schema_identifier(
                    Aspect::IDENTIFIER,
                ),
            )
            .field(
                ApplicationEntityRef::<Self, Entity>::from_schema_identifier(Entity::IDENTIFIER),
                ApplicationFieldRef::<Self, Entity, Aspect, Field, u64>::from_schema_types(),
            )
            .build()
    }
}

impl ApplicationEntityMarkerIdentity<Schema> for Entity {
    const IDENTIFIER: &'static str = "Account";
}

impl ApplicationAspectMarkerIdentity<Schema, Entity> for Aspect {
    const IDENTIFIER: &'static str = "State";
    const ASPECT_IDENTITY: worth_query_declaration::facade::application_schema::AspectIdentity =
        worth_query_declaration::facade::application_schema::AspectIdentity(0x9161200b);
    const CONTRACT_REVISION:
        worth_query_declaration::facade::application_schema::AspectContractRevision =
        worth_query_declaration::facade::application_schema::AspectContractRevision(1);
}

impl ApplicationFieldMarkerIdentity<Schema, Entity, Aspect> for Field {
    const IDENTIFIER: &'static str = "balance";
}

impl DeclaredApplicationFieldValue for Field {
    type Value = u64;
    type Binding = U64ApplicationValueBinding;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}

impl RequiredApplicationFieldValue for Field {}

#[test]
fn initial_installation_cannot_borrow_a_sibling_operations_exact_read() {
    let denied = compile(&members("sibling", "freeze", 64));
    assert!(
        denied.is_err(),
        "the exact demanded read belongs to sibling, not freeze"
    );

    let installed = compile(&members("freeze", "freeze", 64))
        .expect("moving that read onto the candidate operation installs");
    assert!(installed.aftermath().is_some());
}

#[test]
fn reinstallation_recompiles_only_the_presented_candidates_aftermath() {
    let baseline = compile(&members("freeze", "freeze", 64)).unwrap();
    let sibling_owned = compile(&members("freeze", "sibling", 64)).unwrap();
    assert!(sibling_owned.aftermath().is_none());
    assert_ne!(baseline, sibling_owned);

    let bound_drift = compile(&members("freeze", "freeze", 65)).unwrap();
    assert_ne!(baseline, bound_drift);
}

#[test]
fn contract_compilation_is_order_independent() {
    let mut canonical = members("freeze", "freeze", 64);
    canonical.extend([
        ApplicationSchemaMember::OperationProgram {
            operation: "freeze".to_owned(),
            target: ApplicationOperationProgramTarget::Delete {
                entity: "Account".to_owned(),
            },
        },
        ApplicationSchemaMember::OperationProgram {
            operation: "freeze".to_owned(),
            target: ApplicationOperationProgramTarget::Write {
                entity: "Account".to_owned(),
                aspect: "State".to_owned(),
                field: "balance".to_owned(),
            },
        },
    ]);
    let mut reordered = canonical.clone();
    reordered.reverse();
    assert_eq!(compile(&canonical).unwrap(), compile(&reordered).unwrap());
}

#[test]
fn emit_only_operation_has_external_effect_but_no_graph_touch_or_mutation_effect() {
    let members = vec![
        operation("freeze"),
        ApplicationSchemaMember::OperationProgram {
            operation: "freeze".to_owned(),
            target: ApplicationOperationProgramTarget::Emit {
                effect: "notification".to_owned(),
            },
        },
        ApplicationSchemaMember::OperationExternalEffect {
            operation: "freeze".to_owned(),
            effect: "notification".to_owned(),
            rust_payload_type: WorthQueryPortableTypeIdentity::declared("NotificationPayload"),
            protocol: ApplicationExternalEffectProtocol::new(
                BoundaryProtocolIdentity::new("test.notification"),
                BoundaryProtocolVersion::new(1),
            ),
            maximum_payload_bytes: 128,
            correlation_family: WorthQueryExternalEffectCorrelationFamily::new(
                "notification-correlation",
            )
            .unwrap(),
        },
        ApplicationSchemaMember::OperationDecisionFactBudget {
            operation: "freeze".to_owned(),
            maximum_fact_count: 1,
        },
        ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation: "freeze".to_owned(),
            maximum_work_units: 1,
        },
    ];
    let contracts = compile(&members).expect("emit-only operation compiles");
    assert!(matches!(
        contracts.touches(),
        crate::domain_operation::WorthQueryOperationTouchContract::NotRequired
    ));
    assert!(matches!(
        contracts.effects(),
        crate::domain_operation::WorthQueryOperationEffectContract::NotRequired
    ));
    assert_eq!(contracts.emissions().emissions().len(), 1);
    assert_eq!(
        contracts.emissions().emissions()[0].effect(),
        "notification"
    );
    assert!(matches!(
        contracts.external_effect(),
        crate::application_aftermath::InstalledExternalEffectContract::Declared { .. }
    ));
}

#[test]
fn in_process_emission_is_retained_without_forging_external_dispatch_authority() {
    let members = vec![
        operation("freeze"),
        ApplicationSchemaMember::OperationProgram {
            operation: "freeze".to_owned(),
            target: ApplicationOperationProgramTarget::Emit {
                effect: "activity".to_owned(),
            },
        },
        ApplicationSchemaMember::OperationDecisionFactBudget {
            operation: "freeze".to_owned(),
            maximum_fact_count: 1,
        },
        ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation: "freeze".to_owned(),
            maximum_work_units: 1,
        },
    ];

    let contracts = compile(&members).expect("in-process emission compiles");

    assert_eq!(contracts.emissions().emissions().len(), 1);
    assert_eq!(contracts.emissions().emissions()[0].effect(), "activity");
    assert!(!contracts.external_effect().is_declared());
    assert!(matches!(
        contracts.touches(),
        crate::domain_operation::WorthQueryOperationTouchContract::NotRequired
    ));
}

fn compile(
    members: &[ApplicationSchemaMember],
) -> Result<
    WorthQueryCompiledApplicationOperationContracts,
    WorthQueryApplicationOperationInstallationDenial,
> {
    compile_with_portable_members(members, members)
}

fn compile_with_portable_members(
    members: &[ApplicationSchemaMember],
    portable_members: &[ApplicationSchemaMember],
) -> Result<
    WorthQueryCompiledApplicationOperationContracts,
    WorthQueryApplicationOperationInstallationDenial,
> {
    let binding = ApplicationSchemaBindingIdentity::from_installed_parts(
        1,
        1,
        CanonicalDigestId::new([1; 32]),
        CanonicalDigestId::new([2; 32]),
    );
    let declaration = Schema::declaration().expect("fixture schema is valid");
    let (_, schema_work) = crate::application_schema::derive_installed_schema_identity(
        declaration.erased().identity(),
    )
    .expect("fixture schema identity is canonical");
    let native =
        crate::application_schema::compile_portable_native_contract_records(declaration.erased())
            .expect("fixture portable native contracts compile");
    let catalog =
        crate::application_schema::compile_native_contract_catalog(&binding, &native, schema_work)
            .expect("fixture native catalog compiles");
    let portable = crate::application_operation::compile_portable_operation_contract_record(
        Schema::NAME,
        portable_members,
        &native,
        "freeze",
        WorthQueryPortableTypeIdentity::declared("FixtureInput"),
    )
    .expect("fixture portable operation contract compiles");
    WorthQueryApplicationOperationCompilation::resolve(
        binding,
        members,
        &portable,
        "freeze",
        "FixtureInput",
    )?
    .compile_contracts(Vec::new(), &catalog)
}

fn members(
    read_owner: &str,
    aftermath_owner: &'static str,
    bound: usize,
) -> Vec<ApplicationSchemaMember> {
    vec![
        operation("freeze"),
        operation("sibling"),
        ApplicationSchemaMember::OperationProgram {
            operation: "freeze".to_owned(),
            target: ApplicationOperationProgramTarget::Create {
                entity: "Audit".to_owned(),
            },
        },
        ApplicationSchemaMember::OperationDecisionRead {
            operation: read_owner.to_owned(),
            target: ApplicationOperationDecisionReadTarget::Field {
                entity: "Account".to_owned(),
                aspect: "State".to_owned(),
                field: "balance".to_owned(),
            },
        },
        ApplicationSchemaMember::OperationDecisionFactBudget {
            operation: "freeze".to_owned(),
            maximum_fact_count: 4,
        },
        ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation: "freeze".to_owned(),
            maximum_work_units: 16,
        },
        recorded_inverse_member(aftermath_owner, bound),
    ]
}

fn operation(operation: &str) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Operation {
        operation: operation.to_owned(),
        input_type: WorthQueryPortableTypeIdentity::declared("FixtureInput"),
    }
}

fn recorded_inverse_member(operation: &'static str, bound: usize) -> ApplicationSchemaMember {
    let field = ApplicationFieldRef::<Schema, Entity, Aspect, Field, u64>::from_schema_types();
    let inverse = DeclaredRecordedInverse::new(
        "unfreeze",
        DeclaredLoweringCorrespondenceRef::new("inverse").unwrap(),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        DeclaredPreImageDemand::new([DeclaredPreImageLocus::from_field(field)], bound).unwrap(),
    )
    .unwrap();
    let contract = DeclaredApplicationAftermathContract::runtime_alone(
        DeclaredCorrectionMechanism::RecordedInverse(inverse),
    );
    let definition =
        ApplicationOperationRef::<Schema, CompilationOperation, ()>::from_declaration()
            .definition()
            .no_external_effect()
            .aftermath(contract)
            .finish();
    let declaration = ApplicationSchemaDeclarationBuilder::<Schema>::for_schema()
        .operation(definition)
        .build()
        .expect("the matching operation builder associates the aftermath");
    let mut member = declaration
        .erased()
        .members()
        .iter()
        .find(|member| matches!(member, ApplicationSchemaMember::OperationAftermath { .. }))
        .expect("the matching operation emits its portable aftermath member")
        .clone();
    let ApplicationSchemaMember::OperationAftermath {
        operation: installed,
        ..
    } = &mut member
    else {
        unreachable!()
    };
    *installed = operation.to_owned();
    member
}
