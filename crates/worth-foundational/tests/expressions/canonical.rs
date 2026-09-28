//! Canonical identity and the draft codec: source, builder, and decoded
//! drafts agree; meaning changes identity and spelling does not.

use worth_foundational::expression_api::{
    expressions, ExpressionBuilder, ExpressionDenialDetail, ExpressionDenialFamily as Family,
    ExpressionFunctionCatalog, ExpressionProfile, ExpressionType,
};

use super::{admit, draft, empty_catalog, function, schema};

fn digest(source: &str) -> Vec<u8> {
    let admitted = admit(source).unwrap_or_else(|denial| panic!("{source}: {denial:?}"));
    admitted.identity().digest().value().bytes().to_vec()
}

#[test]
fn source_builder_and_decoded_drafts_share_encoding_and_identity() {
    let source = draft("width - 2.0 * depth / 1.0 > 0.5 || frame.material == Material::Steel");
    let mut builder = ExpressionBuilder::new();
    let width = builder.name("width").numeric();
    let two = builder.float(2.0);
    let depth = builder.name("depth").numeric();
    let scaled = builder.multiply(two, depth);
    let one = builder.float(1.0);
    let scaled = builder.divide(scaled, one);
    let difference = builder.subtract(width, scaled);
    let half = builder.float(0.5);
    let wide = builder.greater(difference, half);
    let frame = builder.name("frame");
    let material = builder.field(frame, "material");
    let steel = builder.name("Material::Steel");
    let is_steel = builder.equal(material, steel);
    let root = builder.or(wide, is_steel);
    let built = builder.finish(root).expect("the builder draft is bounded");
    assert_eq!(source.encode(), built.encode());

    let decoded = expressions()
        .decode(&source.encode())
        .expect("the encoding decodes");
    assert_eq!(decoded.encode(), source.encode());

    let schema = schema();
    let catalog = empty_catalog(&schema);
    let identities: Vec<_> = [source, built, decoded]
        .iter()
        .map(|draft| {
            draft
                .admit(&schema, &catalog, ExpressionProfile::interactive())
                .expect("admits")
                .identity()
                .clone()
        })
        .collect();
    assert_eq!(identities[0], identities[1]);
    assert_eq!(identities[0], identities[2]);
}

#[test]
fn spelling_does_not_change_identity() {
    let reference = digest("width*2.5");
    assert_eq!(digest("  width  *  2.50 "), reference);
    assert_eq!(digest("width * 25e-1"), reference);
    assert_eq!(digest("(width) * (2.5)"), reference);
}

#[test]
fn meaning_changes_identity() {
    let reference = digest("width * 2.5");
    assert_ne!(
        digest("depth * 2.5"),
        reference,
        "operand names are semantic"
    );
    assert_ne!(
        digest("2.5 * width"),
        reference,
        "operand order is authored order"
    );
    assert_ne!(digest("width * 2.25"), reference);
    assert_ne!(
        digest("quantity(1.0, mm) < clear_width"),
        digest("quantity(1.0, m) < clear_width")
    );
}

#[test]
fn identity_is_independent_of_the_admitting_profile() {
    let schema = schema();
    let catalog = empty_catalog(&schema);
    let draft = draft("members.map(m, m.thickness)");
    let interactive = draft
        .admit(&schema, &catalog, ExpressionProfile::interactive())
        .expect("admits");
    let engineering = draft
        .admit(&schema, &catalog, ExpressionProfile::engineering())
        .expect("admits");
    assert_eq!(interactive.identity(), engineering.identity());
}

#[test]
fn function_identity_excludes_parameter_names_and_includes_bodies() {
    let schema = schema();
    let catalog = |parameter: &str, body: &str| {
        let mut builder =
            ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive());
        let parameters = [(parameter, ExpressionType::Float64)];
        builder
            .install(function(
                "geometry::double",
                &parameters,
                ExpressionType::Float64,
                body,
            ))
            .expect("installs");
        builder.build()
    };
    let function_identity = |parameter: &str, body: &str| {
        let catalog = catalog(parameter, body);
        let identity = catalog
            .functions()
            .next()
            .expect("one function")
            .identity()
            .clone();
        identity
    };
    let x = function_identity("x", "x * 2.0");
    assert_eq!(x, function_identity("y", "y * 2.0"));
    assert_ne!(x, function_identity("x", "x + x"));

    let call_identity = |body: &str| {
        let catalog = catalog("x", body);
        draft("geometry::double(width)")
            .admit(&schema, &catalog, ExpressionProfile::interactive())
            .expect("admits")
            .identity()
            .clone()
    };
    assert_ne!(
        call_identity("x * 2.0"),
        call_identity("x + x"),
        "callers pin callee bodies"
    );
}

fn header(wire: u16, language: u16, count: u32) -> Vec<u8> {
    let mut bytes = b"WXDR".to_vec();
    bytes.extend_from_slice(&wire.to_le_bytes());
    bytes.extend_from_slice(&language.to_le_bytes());
    bytes.extend_from_slice(&count.to_le_bytes());
    bytes
}

fn bool_node(value: u8) -> [u8; 2] {
    [0, value]
}

fn decode_family(bytes: &[u8]) -> Family {
    expressions()
        .decode(bytes)
        .expect_err("the encoding is invalid")
        .family()
}

#[test]
fn decoding_denies_unknown_versions_before_structure() {
    let wire = expressions().decode(&header(2, 1, u32::MAX)).unwrap_err();
    assert!(matches!(
        wire.detail(),
        ExpressionDenialDetail::UnsupportedVersion { found: 2, .. }
    ));
    let language = expressions().decode(&header(1, 9, u32::MAX)).unwrap_err();
    assert!(matches!(
        language.detail(),
        ExpressionDenialDetail::UnsupportedVersion { found: 9, .. }
    ));
    assert_eq!(decode_family(b"NOPE"), Family::InvalidValue);
    assert_eq!(decode_family(&[]), Family::InvalidValue);
}

#[test]
fn decoding_denies_malformed_trees() {
    let mut valid = header(1, 1, 1);
    valid.extend_from_slice(&bool_node(1));
    assert!(expressions().decode(&valid).is_ok());

    let mut trailing = valid.clone();
    trailing.push(0);
    let mut bad_flag = header(1, 1, 1);
    bad_flag.extend_from_slice(&bool_node(2));
    let mut two_roots = header(1, 1, 2);
    two_roots.extend_from_slice(&bool_node(1));
    two_roots.extend_from_slice(&bool_node(0));
    // Unary `not` over node 1, which is not yet decoded.
    let mut forward_child = header(1, 1, 2);
    forward_child.extend_from_slice(&[7, 0, 1, 0, 0, 0]);
    forward_child.extend_from_slice(&bool_node(1));
    // Binary `or` reading node 0 twice.
    let mut shared_child = header(1, 1, 2);
    shared_child.extend_from_slice(&bool_node(1));
    shared_child.extend_from_slice(&[8, 12, 0, 0, 0, 0, 0, 0, 0, 0]);
    let mut unknown_tag = header(1, 1, 1);
    unknown_tag.extend_from_slice(&[99, 0]);
    let mut empty = header(1, 1, 0);
    empty.extend_from_slice(&bool_node(1));
    for bytes in [
        trailing,
        bad_flag,
        two_roots,
        forward_child,
        shared_child,
        unknown_tag,
        empty,
    ] {
        assert_eq!(decode_family(&bytes), Family::InvalidValue, "{bytes:?}");
    }
}

#[test]
fn decoding_validates_names_and_decimal_text() {
    let name_node = |segment: &[u8]| {
        let mut bytes = header(1, 1, 1);
        bytes.push(4);
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&(segment.len() as u32).to_le_bytes());
        bytes.extend_from_slice(segment);
        bytes
    };
    assert!(expressions().decode(&name_node(b"width")).is_ok());
    assert_eq!(decode_family(&name_node(b"let")), Family::Syntax);
    assert_eq!(decode_family(&name_node(b"9lives")), Family::Syntax);
    assert_eq!(decode_family(&name_node(&[0xff])), Family::InvalidValue);

    let float_node = |digits: &[u8], exponent: i32| {
        let mut bytes = header(1, 1, 1);
        bytes.push(2);
        bytes.extend_from_slice(&(digits.len() as u32).to_le_bytes());
        bytes.extend_from_slice(digits);
        bytes.extend_from_slice(&exponent.to_le_bytes());
        bytes
    };
    assert!(expressions().decode(&float_node(b"25", -1)).is_ok());
    assert!(
        expressions().decode(&float_node(b"250", -2)).is_err(),
        "trailing zeros"
    );
    assert!(
        expressions().decode(&float_node(b"025", -1)).is_err(),
        "leading zeros"
    );
    assert!(expressions().decode(&float_node(b"2.5", 0)).is_err());
}
