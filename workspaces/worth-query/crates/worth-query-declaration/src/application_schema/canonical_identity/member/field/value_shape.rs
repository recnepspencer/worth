//! Canonical declared codec meaning, independent of Rust type identity.
use crate::application_schema::{
    canonical_basis::ApplicationSchemaCanonicalBasis, ApplicationCanonicalValueContract,
};
use worth_foundational::facade::{ValuePrimitive, ValueShape};

pub(super) fn append_contract(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    contract: &ApplicationCanonicalValueContract,
) {
    basis.text(format!("{prefix}.codec.protocol"), contract.wire_protocol());
    let bounds = contract.bounds();
    bound(
        basis,
        format!("{prefix}.codec.maximum-wire-bytes"),
        bounds.maximum_wire_bytes(),
    );
    bound(
        basis,
        format!("{prefix}.codec.maximum-string-bytes"),
        bounds.maximum_string_bytes(),
    );
    bound(
        basis,
        format!("{prefix}.codec.maximum-sequence-length"),
        bounds.maximum_sequence_length(),
    );
    bound(
        basis,
        format!("{prefix}.codec.maximum-depth"),
        bounds.maximum_depth(),
    );
    append_shape(basis, &format!("{prefix}.codec.shape"), contract.shape());
}

fn bound(basis: &mut ApplicationSchemaCanonicalBasis, key: String, maximum: Option<usize>) {
    match maximum {
        Some(maximum) => basis.usize(key, maximum),
        None => basis.text(key, "unrestricted"),
    }
}

fn append_shape(basis: &mut ApplicationSchemaCanonicalBasis, prefix: &str, shape: &ValueShape) {
    match shape {
        ValueShape::Recursive { binding, body } => {
            basis.text(format!("{prefix}.kind"), "recursive");
            basis.usize(format!("{prefix}.binding"), *binding);
            append_shape(basis, &format!("{prefix}.body"), body);
        }
        ValueShape::Reference { binding } => {
            basis.text(format!("{prefix}.kind"), "reference");
            basis.usize(format!("{prefix}.binding"), *binding);
        }
        ValueShape::TaggedEnum { tag, variants } => {
            basis.text(format!("{prefix}.kind"), "internal-enum");
            basis.text(format!("{prefix}.tag"), tag);
            basis.usize(format!("{prefix}.length"), variants.len());
            for (index, variant) in variants.iter().enumerate() {
                let prefix = format!("{prefix}[{index}]");
                basis.text(format!("{prefix}.name"), variant.name());
                basis.usize(format!("{prefix}.length"), variant.fields().len());
                for (index, field) in variant.fields().iter().enumerate() {
                    let prefix = format!("{prefix}[{index}]");
                    basis.text(format!("{prefix}.name"), field.name());
                    append_shape(basis, &prefix, field.shape());
                }
            }
        }
        ValueShape::Primitive(kind) => basis.text(format!("{prefix}.kind"), primitive(*kind)),
        ValueShape::Sequence(child) | ValueShape::Optional(child) => {
            basis.text(
                format!("{prefix}.kind"),
                if matches!(shape, ValueShape::Sequence(_)) {
                    "sequence"
                } else {
                    "optional"
                },
            );
            append_shape(basis, &format!("{prefix}.child"), child);
        }
        ValueShape::Tuple(children) => {
            basis.text(format!("{prefix}.kind"), "tuple");
            basis.usize(format!("{prefix}.length"), children.len());
            for (index, child) in children.iter().enumerate() {
                append_shape(basis, &format!("{prefix}[{index}]"), child);
            }
        }
        ValueShape::Struct(fields) => {
            basis.text(format!("{prefix}.kind"), "struct");
            basis.usize(format!("{prefix}.length"), fields.len());
            for (index, field) in fields.iter().enumerate() {
                let prefix = format!("{prefix}[{index}]");
                basis.text(format!("{prefix}.name"), field.name());
                append_shape(basis, &prefix, field.shape());
            }
        }
        ValueShape::Enum(variants) => {
            basis.text(format!("{prefix}.kind"), "external-enum");
            basis.usize(format!("{prefix}.length"), variants.len());
            for (index, variant) in variants.iter().enumerate() {
                let prefix = format!("{prefix}[{index}]");
                basis.text(format!("{prefix}.name"), variant.name());
                basis.text(
                    format!("{prefix}.form"),
                    if variant.payload().is_some() {
                        "newtype"
                    } else {
                        "unit"
                    },
                );
                if let Some(payload) = variant.payload() {
                    append_shape(basis, &prefix, payload);
                }
            }
        }
    }
}

fn primitive(kind: ValuePrimitive) -> &'static str {
    match kind {
        ValuePrimitive::Bool => "bool",
        ValuePrimitive::I8 => "i8",
        ValuePrimitive::I16 => "i16",
        ValuePrimitive::I32 => "i32",
        ValuePrimitive::I64 => "i64",
        ValuePrimitive::U8 => "u8",
        ValuePrimitive::U16 => "u16",
        ValuePrimitive::U32 => "u32",
        ValuePrimitive::U64 => "u64",
        ValuePrimitive::String => "string",
        ValuePrimitive::Unit => "unit",
    }
}
