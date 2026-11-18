use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[path = "native_aspect_contract_record/records.rs"]
mod storage_model;
pub use storage_model::*;
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

// Current framing and A's schema-v19-derived fragment row revision; v18 stays frozen.
pub const VERSION_NINETEEN_NATIVE_ASPECT_HEX: &str = "0013000b00000002000000af0000000c4e6174697665536368656d610000000c4e6174697665456e746974790000001161613463633463333838343638383932340000001161613463633463333838343638383932340fc38577fb0ddb818e43e2b350ce347a000100020002000100020000000200000003416765000a0001000100020000000354616700100001000100020000000200000003416765000000035461670001000000116161346363346333383834363838393234";

pub struct NativeSchema;

impl ApplicationSchema for NativeSchema {
    const OWNER: &'static str = "archive.native";
    const NAME: &'static str = "NativeSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
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
        .as_chunks::<2>()
        .0
        .iter()
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

impl fragment::Fragment for NativeSchema {
    const NAME: &'static str = "NativeSchema";
    type Bindings = NoBindings;
    type Members = (NativeModel,);
}
