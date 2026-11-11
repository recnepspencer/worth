use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectValueLocator, BoundarySourceLocator,
    CanonicalFieldPath, FieldKey, LocatorAuthority,
};

use super::{
    decode_aspect_field_locator, decode_aspect_value_locator, decode_boundary_source_locator,
    encode_aspect_field_locator, encode_aspect_value_locator, encode_boundary_source_locator,
};

#[test]
fn aspect_value_locator_codec_roundtrips_whole_aspect_and_struct_field() {
    for locator in [
        AspectValueLocator::whole_aspect(AspectLocator::new(
            LocatorAuthority::Authoritative,
            AspectKey::new("deploy.replicas").expect("valid aspect key"),
        )),
        AspectValueLocator::struct_field(AspectFieldLocator::new(
            LocatorAuthority::ReceiptBearing,
            AspectKey::new("deploy.config").expect("valid aspect key"),
            CanonicalFieldPath::new(vec![
                FieldKey::new("routing").expect("valid field key"),
                FieldKey::new("weight").expect("valid field key"),
            ])
            .expect("valid field path"),
        )),
    ] {
        let encoded = encode_aspect_value_locator(&locator);
        let decoded = decode_aspect_value_locator(&encoded).expect("decode locator bytes");

        assert_eq!(decoded, locator);
    }
}

#[test]
fn aspect_field_locator_codec_rejects_whole_aspect_locator_bytes() {
    let whole_aspect = AspectValueLocator::whole_aspect(AspectLocator::new(
        LocatorAuthority::Authoritative,
        AspectKey::new("deploy.replicas").expect("valid aspect key"),
    ));

    let error = decode_aspect_field_locator(&encode_aspect_value_locator(&whole_aspect))
        .expect_err("whole-aspect bytes are not aspect-field locator bytes");

    assert!(
        error.to_string().contains("expected aspect field locator"),
        "unexpected field locator codec error: {error}"
    );
}

#[test]
fn aspect_field_locator_codec_roundtrips_canonical_bytes() {
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("deploy.config").expect("valid aspect key"),
        CanonicalFieldPath::single(FieldKey::new("replicas").expect("valid field key")),
    );

    let decoded = decode_aspect_field_locator(&encode_aspect_field_locator(&locator))
        .expect("decode aspect field locator bytes");

    assert_eq!(decoded, locator);
}

#[test]
fn boundary_source_locator_codec_roundtrips_aspect_loci() {
    for locator in [
        BoundarySourceLocator::aspect(AspectLocator::new(
            LocatorAuthority::SupportOnly,
            AspectKey::new("deploy.replicas").expect("valid aspect key"),
        )),
        BoundarySourceLocator::aspect_field(AspectFieldLocator::new(
            LocatorAuthority::SupportOnly,
            AspectKey::new("deploy.config").expect("valid aspect key"),
            CanonicalFieldPath::single(FieldKey::new("replicas").expect("valid field key")),
        )),
    ] {
        let encoded = encode_boundary_source_locator(&locator).expect("encode source locator");
        let decoded = decode_boundary_source_locator(&encoded).expect("decode source locator");

        assert_eq!(decoded, locator);
    }
}

#[test]
fn aspect_value_locator_codec_rejects_malformed_bytes() {
    let malformed_locator = vec![99, 1];

    let error = decode_aspect_value_locator(&malformed_locator)
        .expect_err("malformed locator bytes should fail");

    assert!(
        error
            .to_string()
            .contains("unknown aspect value locator tag")
            || error.to_string().contains("input ended early"),
        "unexpected locator codec error: {error}"
    );
}

#[test]
fn owned_decode_streaming_locator_roster_preserves_old_named_and_compact_bytes() {
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Current {
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator_arc_slice")]
        locators: Arc<[AspectFieldLocator]>,
    }
    #[derive(Serialize)]
    struct Previous {
        locators: Vec<Vec<u8>>,
    }
    for count in [0, 1, 17] {
        let locators = (0..count)
            .map(|ordinal| {
                AspectFieldLocator::new(
                    LocatorAuthority::Planned,
                    AspectKey::new(format!("roster.aspect{ordinal}")).unwrap(),
                    CanonicalFieldPath::single(FieldKey::new("field").unwrap()),
                )
            })
            .collect::<Vec<_>>();
        let previous = Previous {
            locators: locators.iter().map(encode_aspect_field_locator).collect(),
        };
        let current = Current {
            locators: Arc::from(locators),
        };
        for (old, new) in [
            (
                rmp_serde::to_vec_named(&previous).unwrap(),
                rmp_serde::to_vec_named(&current).unwrap(),
            ),
            (
                rmp_serde::to_vec(&previous).unwrap(),
                rmp_serde::to_vec(&current).unwrap(),
            ),
        ] {
            assert_eq!(old, new);
            assert_eq!(rmp_serde::from_slice::<Current>(&old).unwrap(), current);
        }
    }
}
