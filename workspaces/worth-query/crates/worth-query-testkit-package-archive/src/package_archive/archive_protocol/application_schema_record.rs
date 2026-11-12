pub use worth_query_declaration::facade::application_schema::{
    ApplicationEntityRef, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder,
};
pub use worth_query_declaration::facade::{
    application_history::ApplicationHistoryModelDomain,
    application_query::ApplicationQueryDisclosurePosture,
    application_schema::ApplicationSchemaMember,
};
pub use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecord,
    WorthQueryPortablePackageRecordFamily as Family,
};
pub use worth_query_package_archive::facade::*;

pub const VERSION_SIX_MINIMAL_SCHEMA_HEX: &str = "0006000800000001000000380000000d617263686976652e74657374730000000d4d696e696d616c536368656d610000000100000000000000010001000000044974656d";
// Retained protocol7 framing is unsupported under the current-only reader.
pub const VERSION_SEVEN_MINIMAL_SCHEMA_HEX: &str = "0007000800000001000000380000000d617263686976652e74657374730000000d4d696e696d616c536368656d610000000100000000000000010001000000044974656d";
// Displaced protocol9 is refused; protocol10 has no Live resource fields.
pub const VERSION_NINE_MINIMAL_SCHEMA_HEX: &str = "0009000800000001000000380000000d617263686976652e74657374730000000d4d696e696d616c536368656d610000000100000000000000010001000000044974656d";
pub const VERSION_THIRTEEN_MINIMAL_SCHEMA_HEX: &str = "000d000800000001000000380000000d617263686976652e74657374730000000d4d696e696d616c536368656d610000000100000000000000010001000000044974656d";
pub const VERSION_FOURTEEN_MINIMAL_SCHEMA_HEX: &str = "000e000800000001000000380000000d617263686976652e74657374730000000d4d696e696d616c536368656d610000000100000000000000010001000000044974656d";
pub const VERSION_SIXTEEN_MINIMAL_SCHEMA_HEX: &str = "0010000800000001000000380000000d617263686976652e74657374730000000d4d696e696d616c536368656d610000000100000000000000010001000000044974656d";
pub const VERSION_SEVENTEEN_MINIMAL_SCHEMA_HEX: &str = "0011000800000001000000380000000d617263686976652e74657374730000000d4d696e696d616c536368656d610000000100000000000000010001000000044974656d";
pub const VERSION_EIGHTEEN_MINIMAL_SCHEMA_HEX: &str = "0012000800000001000000380000000d617263686976652e74657374730000000d4d696e696d616c536368656d610000000100000000000000010001000000044974656d";
// Current record framing; the frozen v18 payload is unchanged.
pub const VERSION_NINETEEN_MINIMAL_SCHEMA_HEX: &str = "0013000800000001000000380000000d617263686976652e74657374730000000d4d696e696d616c536368656d610000000100000000000000010001000000044974656d";

pub struct Schema;
pub struct Entity;

impl worth_query_declaration::facade::application_schema::fragment::Entity for Entity {
    const NAME: &'static str = "Item";
    type Aspects = ();
    type Relations = ();
}

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "archive.tests";
    const NAME: &'static str = "MinimalSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::from_fragment(FixtureRecords)?.build()
    }
}

pub fn schema_package() -> worth_query_installation::facade::WorthQueryValidatedPortableDomainPackage
{
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "archive.tests",
        1,
        0,
    ))
    .application_schema(Schema::declaration().unwrap())
    .validate()
    .unwrap()
}

pub fn schema_view<'a>(
    exported: &'a worth_query_installation::facade::WorthQueryPortablePackageRecordSet,
) -> worth_query_installation::facade::WorthQueryPortablePackageRecordView<'a> {
    exported
        .views()
        .find(|view| view.family() == Family::ApplicationSchema)
        .unwrap()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn decode_hex(encoded: &str) -> Vec<u8> {
    encoded
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let pair = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(pair, 16).unwrap()
        })
        .collect()
}

pub use worth_query_declaration::facade::application_schema::validate_portable_application_schema_freshly;
pub use worth_query_installation::facade::WorthQueryPortablePackageReconstructionCandidate;

struct FixtureRecords;
impl worth_query_declaration::facade::application_schema::fragment::Fragment for FixtureRecords {
    const NAME: &'static str = "fixture_records";
    type Bindings = worth_query_declaration::facade::application_schema::fragment::NoBindings;
    type Members = (Entity,);
}
