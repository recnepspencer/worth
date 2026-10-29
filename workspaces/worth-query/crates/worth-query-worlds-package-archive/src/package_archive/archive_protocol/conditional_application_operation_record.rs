pub use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
};
pub use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
pub use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity,
    WorthQueryPortableApplicationConditionalOperationBinding,
    WorthQueryPortableApplicationConditionalOperationBindingParts,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
    WorthQueryPortablePackageReconstruction, WorthQueryPortablePackageReconstructionLimits,
    WorthQueryPortablePackageRecordFamily as Family, WorthQueryValidatedPortableDomainPackage,
};
pub use worth_query_package_archive::facade::*;

pub const VERSION_SIX_CONDITIONAL_BINDING_HEX: &str = "0006000900000003000000cc00000013617263686976652e636f6e646974696f6e616c00000011436f6e646974696f6e616c536368656d610000001b436f6e646974696f6e616c417263686976654f7065726174696f6e00000025776f7274682e71756572792e617263686976652e636f6e646974696f6e616c2d696e707574000000106c65646765722d62616c616e63653a310000004066356337363266396237616165373466643039373238373536396361363063313062356537316164623732303232373032343933316164363363373338353531";
// Retained protocol7 framing is unsupported under the current-only reader.
pub const VERSION_SEVEN_CONDITIONAL_BINDING_HEX: &str = "0007000900000003000000cc00000013617263686976652e636f6e646974696f6e616c00000011436f6e646974696f6e616c536368656d610000001b436f6e646974696f6e616c417263686976654f7065726174696f6e00000025776f7274682e71756572792e617263686976652e636f6e646974696f6e616c2d696e707574000000106c65646765722d62616c616e63653a310000004066356337363266396237616165373466643039373238373536396361363063313062356537316164623732303232373032343933316164363363373338353531";
// Displaced protocol9 is refused; protocol10 has no Live resource fields.
pub const VERSION_NINE_CONDITIONAL_BINDING_HEX: &str = "0009000900000003000000cc00000013617263686976652e636f6e646974696f6e616c00000011436f6e646974696f6e616c536368656d610000001b436f6e646974696f6e616c417263686976654f7065726174696f6e00000025776f7274682e71756572792e617263686976652e636f6e646974696f6e616c2d696e707574000000106c65646765722d62616c616e63653a310000004066356337363266396237616165373466643039373238373536396361363063313062356537316164623732303232373032343933316164363363373338353531";
pub const VERSION_THIRTEEN_CONDITIONAL_BINDING_HEX: &str = "000d000900000003000000cc00000013617263686976652e636f6e646974696f6e616c00000011436f6e646974696f6e616c536368656d610000001b436f6e646974696f6e616c417263686976654f7065726174696f6e00000025776f7274682e71756572792e617263686976652e636f6e646974696f6e616c2d696e707574000000106c65646765722d62616c616e63653a310000004066356337363266396237616165373466643039373238373536396361363063313062356537316164623732303232373032343933316164363363373338353531";
pub const VERSION_FOURTEEN_CONDITIONAL_BINDING_HEX: &str = "000e000900000003000000cc00000013617263686976652e636f6e646974696f6e616c00000011436f6e646974696f6e616c536368656d610000001b436f6e646974696f6e616c417263686976654f7065726174696f6e00000025776f7274682e71756572792e617263686976652e636f6e646974696f6e616c2d696e707574000000106c65646765722d62616c616e63653a310000004066356337363266396237616165373466643039373238373536396361363063313062356537316164623732303232373032343933316164363363373338353531";
pub const VERSION_SIXTEEN_CONDITIONAL_BINDING_HEX: &str = "0010000900000003000000cc00000013617263686976652e636f6e646974696f6e616c00000011436f6e646974696f6e616c536368656d610000001b436f6e646974696f6e616c417263686976654f7065726174696f6e00000025776f7274682e71756572792e617263686976652e636f6e646974696f6e616c2d696e707574000000106c65646765722d62616c616e63653a310000004066356337363266396237616165373466643039373238373536396361363063313062356537316164623732303232373032343933316164363363373338353531";
pub const VERSION_SEVENTEEN_CONDITIONAL_BINDING_HEX: &str = "0011000900000003000000cc00000013617263686976652e636f6e646974696f6e616c00000011436f6e646974696f6e616c536368656d610000001b436f6e646974696f6e616c417263686976654f7065726174696f6e00000025776f7274682e71756572792e617263686976652e636f6e646974696f6e616c2d696e707574000000106c65646765722d62616c616e63653a310000004066356337363266396237616165373466643039373238373536396361363063313062356537316164623732303232373032343933316164363363373338353531";
pub const VERSION_EIGHTEEN_CONDITIONAL_BINDING_HEX: &str = "0012000900000003000000cc00000013617263686976652e636f6e646974696f6e616c00000011436f6e646974696f6e616c536368656d610000001b436f6e646974696f6e616c417263686976654f7065726174696f6e00000025776f7274682e71756572792e617263686976652e636f6e646974696f6e616c2d696e707574000000106c65646765722d62616c616e63653a310000004066356337363266396237616165373466643039373238373536396361363063313062356537316164623732303232373032343933316164363363373338353531";

pub struct Schema;
pub struct Operation;
pub struct Input;

worth_query_declaration::worth_query_portable_type!(
    Input => "worth.query.archive.conditional-input"
);
worth_query_declaration::worth_query_structured_value_binding!(
    pub InputBinding for Input {
        identity: "worth.query.archive.conditional-input"
    }
);

impl ApplicationOperationMarkerIdentity<Schema> for Operation {
    type InputBinding = InputBinding;
    const IDENTIFIER: &'static str = "ConditionalArchiveOperation";
}

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "archive.conditional";
    const NAME: &'static str = "ConditionalSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        let operation = ApplicationOperationRef::<Self, Operation, Input>::from_declaration();
        ApplicationSchemaDeclarationBuilder::for_schema()
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
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

pub fn conditional_package() -> WorthQueryValidatedPortableDomainPackage {
    let operation = super::domain_operation_record::fixture::operation();
    let binding = WorthQueryPortableApplicationConditionalOperationBinding::from_untrusted_parts(
        WorthQueryPortableApplicationConditionalOperationBindingParts {
            schema_owner: Schema::OWNER.to_owned(),
            schema_name: Schema::NAME.to_owned(),
            application_operation: Operation::IDENTIFIER.to_owned(),
            input_type: WorthQueryPortableTypeIdentity::from_untrusted(
                "worth.query.archive.conditional-input".to_owned(),
            ),
            domain_operation_slot: operation.identity().slot(),
            domain_operation_canonical_identity: operation.canonical_identity().to_owned(),
        },
    );
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "archive.conditional",
        1,
        0,
    ))
    .domain_operation(operation)
    .application_schema(Schema::declaration().unwrap())
    .conditional_application_operation_erased(binding)
    .validate()
    .unwrap()
}

pub fn conditional_view(
    exported: &worth_query_installation::facade::WorthQueryPortablePackageRecordSet,
) -> worth_query_installation::facade::WorthQueryPortablePackageRecordView<'_> {
    exported
        .views()
        .find(|view| view.family() == Family::ConditionalApplicationOperation)
        .unwrap()
}
