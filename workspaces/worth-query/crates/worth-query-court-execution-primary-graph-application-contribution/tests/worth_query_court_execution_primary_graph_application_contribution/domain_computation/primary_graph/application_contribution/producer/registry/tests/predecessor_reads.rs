//! Installed declarations refuse foreign, missing and effectful historical operations.
use super::{declared, InstalledProducer, InstalledProducerEdition, TypeId};
use worth_query_declaration::facade::application_schema::{self as schema, *};
use worth_query_execution::facade::application_contribution::{
    ProducerPublicationOwnership as Ownership, WorthQueryPredecessorReadContract as Contract,
    WorthQueryPredecessorReadDenial as Denial, WorthQueryPredecessorReadOperation as Read,
};
use worth_query_installation::facade as domain;

worth_query_declaration::worth_query_structured_value_binding!(pub InputBinding for String { identity: "worth.predecessor.input" });

struct Model;
impl schema::fragment::Fragment for Model {
    const NAME: &'static str = "PredecessorModel";
    type Bindings = schema::fragment::NoBindings;
    type Members = (InputEntity,);
}
struct InputEntity;
impl schema::fragment::Entity for InputEntity {
    const NAME: &'static str = "InputEntity";
    type Aspects = ();
    type Relations = ();
}
impl OperationReads<ReadInput> for InputEntity {}
impl OperationReads<OtherRead> for InputEntity {}
struct Schema;
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "predecessor_contract";
    const NAME: &'static str = "PredecessorContract";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::from_fragment(Model)?
            .effect(Effect::reference())
            .operation(
                ReadInput::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation(
                OtherRead::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation(
                Emits::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_entity(
                ReadInput::reference(),
                ApplicationEntityRef::<Schema, InputEntity>::from_schema_identifier("InputEntity"),
            )
            .operation_read_entity(
                OtherRead::reference(),
                ApplicationEntityRef::<Schema, InputEntity>::from_schema_identifier("InputEntity"),
            )
            .operation_emit(Emits::reference(), Effect::reference())
            .build()
    }
}
macro_rules! operation {
    ($type:ident, $name:literal) => {
        struct $type;
        impl ApplicationOperationMarkerIdentity<Schema> for $type {
            type InputBinding = InputBinding;
            const IDENTIFIER: &'static str = $name;
        }
        impl $type {
            const fn reference() -> ApplicationOperationRef<Schema, Self, String> {
                ApplicationOperationRef::from_declaration()
            }
        }
    };
}
operation!(ReadInput, "ReadInput");
operation!(OtherRead, "OtherRead");
operation!(Missing, "Missing");
operation!(ForeignRead, "ReadInput");
operation!(Emits, "Emits");
struct Effect;
impl ApplicationEffectMarkerIdentity<Schema> for Effect {
    type PayloadBinding = InputBinding;
    const IDENTIFIER: &'static str = "Emitted";
}
impl Effect {
    const fn reference() -> ApplicationEffectRef<Schema, Self, String> {
        ApplicationEffectRef::from_declaration()
    }
}
impl OperationEmits<Emits> for Effect {}
const READS: Contract<Schema> = Contract::new(&[Read::new(ReadInput::reference())]);
const OTHER: Contract<Schema> = Contract::new(&[Read::new(OtherRead::reference())]);
const MISSING: Contract<Schema> = Contract::new(&[Read::new(Missing::reference())]);
const FOREIGN: Contract<Schema> = Contract::new(&[Read::new(ForeignRead::reference())]);
const EMITTING: Contract<Schema> = Contract::new(&[Read::new(Emits::reference())]);
const DUPLICATE: Contract<Schema> = Contract::new(&[
    Read::new(ReadInput::reference()),
    Read::new(ReadInput::reference()),
]);
fn installed() -> domain::WorthQueryInstalledApplicationSchema<Schema> {
    let declaration = Schema::declaration().unwrap();
    let admitted = domain::WorthQueryInstallationAdmissionProfile::new("predecessor", "contract")
        .admit(
            domain::WorthQueryPortableDomainPackage::new(
                domain::WorthQueryPortableDomainIdentity::new(Schema::OWNER, 1, 0),
            )
            .application_schema(declaration.clone())
            .validate()
            .unwrap(),
        )
        .unwrap();
    domain::WorthQueryInstalledPackageIndex::build(
        domain::WorthQueryInstallationRuntimeIdentity::fresh(),
        domain::WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
    .unwrap()
}
#[test]
fn predecessor_contract_requires_the_declared_exact_operation_type() {
    assert_eq!(READS.require::<ReadInput>(), Ok(()));
    assert_eq!(
        READS.require::<OtherRead>(),
        Err(Denial::UndeclaredPredecessorRead)
    );
    assert_eq!(
        READS.require::<ForeignRead>(),
        Err(Denial::UndeclaredPredecessorRead)
    );
    assert_eq!(
        Contract::<Schema>::new(&[]).require::<ReadInput>(),
        Err(Denial::UndeclaredPredecessorRead)
    );
}
#[test]
fn installation_requires_owned_read_only_real_operations_without_duplicates() {
    let schema = installed();
    READS
        .validate(&schema, Ownership::SourceScopeCascade)
        .unwrap();
    assert!(READS.validate(&schema, Ownership::Independent).is_err());
    for contract in [MISSING, FOREIGN, EMITTING, DUPLICATE] {
        assert!(contract
            .validate(&schema, Ownership::SourceScopeCascade)
            .is_err());
    }
}
#[test]
fn predecessor_reads_change_producer_equality_and_portable_edition() {
    let base = declared(TypeId::of_for_test::<InstalledProducer>());
    let mut opted = base.clone();
    opted.set_predecessor_reads_for_test(Some(READS));
    let mut other = base.clone();
    other.set_predecessor_reads_for_test(Some(OTHER));
    assert!(!base.has_same_meaning_as_for_test(&opted));
    assert!(!opted.has_same_meaning_as_for_test(&other));
    let edition = |declaration| {
        InstalledProducerEdition::from_declaration_for_test(
            declaration,
            worth_foundational::facade::ApplicationBuildIdentity::parse(
                "11111111-1111-1111-1111-111111111111",
            )
            .unwrap(),
        )
        .unwrap()
        .digest_for_test()
    };
    assert_ne!(edition(&base), edition(&opted));
    assert_ne!(edition(&opted), edition(&other));
}
