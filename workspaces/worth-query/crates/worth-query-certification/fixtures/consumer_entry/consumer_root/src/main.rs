#![forbid(unsafe_code)]

use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ApplicationSchema,
};
use worth_query_decl::facade::worth_query_application;
use worth_query_host::facade::domain::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_query_parameter_entry::{
    ParameterContribution, ParameterCountBinding, ParameterSchemaBinding,
};
use worth_query_topology_entry::{
    TopologyContribution, TopologyLengthBinding, TopologySchemaBinding,
};

worth_query_application! {
    pub ConsumerSchema {
        owner: "worth.query.certification.consumer",
        version: (1, 0),
        contributions: [TopologyContribution, ParameterContribution],
    }
}

impl TopologySchemaBinding for ConsumerSchema {}
impl ParameterSchemaBinding for ConsumerSchema {}

fn main() {
    let declaration = ConsumerSchema::declaration()
        .expect("the root-owned contributions form one closed schema declaration");

    assert_eq!(declaration.contributions().len(), 2);
    assert_eq!(declaration.erased().members().len(), 8);
    assert_ne!(
        TopologyLengthBinding::IDENTITY,
        ParameterCountBinding::IDENTITY
    );

    let length = PositiveLength::new(42).expect("the fixture length is positive");
    let encoded = TopologyLengthBinding::encode(&length).expect("the binding accepts the value");
    assert_eq!(
        TopologyLengthBinding::decode(&encoded).expect("the binding decodes its carrier"),
        length
    );

    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        ConsumerSchema::OWNER,
        ConsumerSchema::MAJOR,
        ConsumerSchema::MINOR,
    ))
    .application_schema(declaration.clone())
    .validate()
    .expect("the root declaration forms a portable package");
    let admitted = WorthQueryInstallationAdmissionProfile::new(
        "worth.query.certification.support",
        "worth.query.certification.configuration",
    )
    .admit(package)
    .expect("the public admission path accepts the root package");
    let installed_index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .expect("the public installation path installs the root package");
    let installed = installed_index
        .bind_application_schema(declaration)
        .expect("the installed authority binds the exact root declaration");
    assert_eq!(installed.contributions().len(), 2);
    assert!(installed
        .contributions()
        .get("worth.query.certification.topology.v1")
        .is_some());
    assert!(installed
        .contributions()
        .get("worth.query.certification.parameter.v1")
        .is_some());

    hostile_binding_identity_and_unit_cannot_preserve_schema_identity();
}

fn hostile_binding_identity_and_unit_cannot_preserve_schema_identity() {
    let baseline = hostile::baseline::Schema::declaration().unwrap();
    let wrong_identity = hostile::wrong_identity::Schema::declaration().unwrap();
    let wrong_unit = hostile::wrong_unit::Schema::declaration().unwrap();

    assert_ne!(baseline.identity(), wrong_identity.identity());
    assert_ne!(baseline.identity(), wrong_unit.identity());
}

mod hostile {
    macro_rules! schema_with_binding {
        ($module:ident, $binding:ident, $identity:literal, $unit:ident, $unit_identity:literal) => {
            pub mod $module {
                use worth_query_consumer_values::PositiveLength;
                use worth_query_decl::facade::{
                    worth_query_application_schema, worth_query_aspect, worth_query_entity,
                    worth_query_field, worth_query_unit, worth_query_value_binding,
                };

                worth_query_value_binding! {
                    $binding for PositiveLength {
                        identity: $identity,
                        scalar: UInt64,
                        unit: $unit_identity,
                        frame: "worth.frames.model-local.v1",
                        encode: PositiveLength::get,
                        decode: PositiveLength::new,
                    }
                }
                worth_query_entity!(Body for Schema);
                worth_query_unit!($unit(()) in Schema);
                worth_query_aspect!(
                    Geometry for Schema, Body;
                    identity = AspectIdentity(0x9174_2001),
                    revision = AspectContractRevision(1),
                );
                worth_query_field!(
                    Length for Schema, Body, Geometry:
                    PositiveLength => $binding, unit $unit, read_write, equality
                );
                worth_query_application_schema! {
                    pub(crate) schema Schema {
                        owner: "worth.query.certification.binding-hostile",
                        version: (1, 0),
                        members: |schema| {
                            schema
                                .entity(Body::reference())
                                .unit($unit::reference())
                                .aspect(Body::reference(), Geometry::reference())
                                .field(Body::reference(), Length::reference())
                        }
                    }
                }
            }
        };
    }

    schema_with_binding!(
        baseline,
        BaselineBinding,
        "worth.query.certification.length.v1",
        Metre,
        "Metre"
    );
    schema_with_binding!(
        wrong_identity,
        WrongIdentityBinding,
        "worth.query.certification.foreign-length.v1",
        Metre,
        "Metre"
    );
    schema_with_binding!(
        wrong_unit,
        WrongUnitBinding,
        "worth.query.certification.length.v1",
        Millimetre,
        "Millimetre"
    );
}
