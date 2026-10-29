pub use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    StringApplicationValueBinding, U64ApplicationValueBinding,
};
pub use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecordFamily as Family,
    WorthQueryValidatedPortableDomainPackage,
};
pub use worth_query_package_archive::facade::*;

pub const VERSION_SIX_NATIVE_ASPECT_HEX: &str = "0006000b00000002000000890000000c4e6174697665536368656d610000000c4e6174697665456e746974790000000750726f66696c650000000750726f66696c650000000091621100000000000000000300020000000200000003416765000a00010001000200000003546167001000010001000200000002000000034167650000000354616700010000000750726f66696c65";
// Retained protocol7 framing is unsupported under the current-only reader.
pub const VERSION_SEVEN_NATIVE_ASPECT_HEX: &str = "0007000b00000002000000890000000c4e6174697665536368656d610000000c4e6174697665456e746974790000000750726f66696c650000000750726f66696c650000000091621100000000000000000300020000000200000003416765000a00010001000200000003546167001000010001000200000002000000034167650000000354616700010000000750726f66696c65";
// Displaced protocol9 is refused; protocol10 has no Live resource fields.
pub const VERSION_NINE_NATIVE_ASPECT_HEX: &str = "0009000b00000002000000910000000c4e6174697665536368656d610000000c4e6174697665456e746974790000000750726f66696c650000000750726f66696c6500000000916211000000000000000003000100020002000100020000000200000003416765000a00010001000200000003546167001000010001000200000002000000034167650000000354616700010000000750726f66696c65";
pub const VERSION_THIRTEEN_NATIVE_ASPECT_HEX: &str = "000d000b00000002000000910000000c4e6174697665536368656d610000000c4e6174697665456e746974790000000750726f66696c650000000750726f66696c6500000000916211000000000000000003000100020002000100020000000200000003416765000a00010001000200000003546167001000010001000200000002000000034167650000000354616700010000000750726f66696c65";
pub const VERSION_FOURTEEN_NATIVE_ASPECT_HEX: &str = "000e000b00000002000000910000000c4e6174697665536368656d610000000c4e6174697665456e746974790000000750726f66696c650000000750726f66696c6500000000916211000000000000000003000100020002000100020000000200000003416765000a00010001000200000003546167001000010001000200000002000000034167650000000354616700010000000750726f66696c65";
pub const VERSION_SIXTEEN_NATIVE_ASPECT_HEX: &str = "0010000b00000002000000910000000c4e6174697665536368656d610000000c4e6174697665456e746974790000000750726f66696c650000000750726f66696c6500000000916211000000000000000003000100020002000100020000000200000003416765000a00010001000200000003546167001000010001000200000002000000034167650000000354616700010000000750726f66696c65";
pub const VERSION_SEVENTEEN_NATIVE_ASPECT_HEX: &str = "0011000b00000002000000910000000c4e6174697665536368656d610000000c4e6174697665456e746974790000000750726f66696c650000000750726f66696c6500000000916211000000000000000003000100020002000100020000000200000003416765000a00010001000200000003546167001000010001000200000002000000034167650000000354616700010000000750726f66696c65";
pub const VERSION_EIGHTEEN_NATIVE_ASPECT_HEX: &str = "0012000b00000002000000910000000c4e6174697665536368656d610000000c4e6174697665456e746974790000000750726f66696c650000000750726f66696c6500000000916211000000000000000003000100020002000100020000000200000003416765000a00010001000200000003546167001000010001000200000002000000034167650000000354616700010000000750726f66696c65";

pub struct NativeSchema;
worth_query_declaration::worth_query_entity!(NativeEntity for NativeSchema);
worth_query_declaration::worth_query_aspect!(
    Profile for NativeSchema, NativeEntity;
    identity = AspectIdentity(0x9162_1100),
    revision = AspectContractRevision(3),
);
worth_query_declaration::worth_query_field!(
    Age for NativeSchema, NativeEntity, Profile:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_declaration::worth_query_field!(
    Tag for NativeSchema, NativeEntity, Profile:
    String => StringApplicationValueBinding, read_only, no_equality
);

impl ApplicationSchema for NativeSchema {
    const OWNER: &'static str = "archive.native";
    const NAME: &'static str = "NativeSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(NativeEntity::reference())
            .aspect(NativeEntity::reference(), Profile::reference())
            .field(NativeEntity::reference(), Age::reference())
            .field(NativeEntity::reference(), Tag::reference())
            .build()
    }
}

pub fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut encoded, byte| {
            write!(&mut encoded, "{byte:02x}").unwrap();
            encoded
        },
    )
}

pub fn decode_hex(encoded: &str) -> Vec<u8> {
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digits = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(digits, 16).unwrap()
        })
        .collect()
}

pub fn native_package() -> WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "archive.native",
        1,
        0,
    ))
    .application_schema(NativeSchema::declaration().unwrap())
    .validate()
    .unwrap()
}

pub fn native_view(
    exported: &worth_query_installation::facade::WorthQueryPortablePackageRecordSet,
) -> worth_query_installation::facade::WorthQueryPortablePackageRecordView<'_> {
    exported
        .views()
        .find(|view| view.family() == Family::NativeAspectContract)
        .unwrap()
}

pub fn assert_decode_kind(bytes: &[u8], expected: WorthQueryPackageArchiveDenialKind) {
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert_eq!(decoder.decode_frame(bytes).unwrap_err().kind(), expected);
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}
