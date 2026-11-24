//! `CanonicalF64` 2D box court: negative coordinates, signed zero, NaN/reversed probe refusal,
//! wire non-canonicality refusal, and same-host post-refusal parity.
use super::operations::AssetSpec;
use super::schema::{FindInFloatBox2D, FloatBox2DIndex};
use super::{install, keys};
use worth_query_decl::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ApplicationValue,
    ApplicationValueDecodeDenial, ApplicationValueEncodeDenial, CanonicalF64, InlineBytes,
    ValueCodecDenial,
};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::ordered_index::{WorthQueryBox2DF64, WorthQueryBox2DF64Binding};
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

#[test]
fn canonical_f64_box2d_handles_negative_coordinates_signed_zero_and_nan_refusal() {
    let mut s1 = AssetSpec::i64_only(1, "NegEquip", (0, 10), [0, 0, 0], [1, 1, 1]);
    s1.fmin_bits = [
        CanonicalF64::from_f64(-50.5).0,
        CanonicalF64::from_f64(-20.0).0,
    ];
    s1.fmax_bits = [
        CanonicalF64::from_f64(-10.25).0,
        CanonicalF64::from_f64(-0.0).0,
    ];
    let mut s2 = AssetSpec::i64_only(2, "OriginEquip", (0, 10), [0, 0, 0], [1, 1, 1]);
    s2.fmin_bits = [
        CanonicalF64::from_f64(0.0).0,
        CanonicalF64::from_f64(-5.0).0,
    ];
    s2.fmax_bits = [
        CanonicalF64::from_f64(12.5).0,
        CanonicalF64::from_f64(8.5).0,
    ];

    let host = install(&[s1, s2], None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let req = host.runtime().request(&principal, &scope);

    let idx = req
        .query(FindInFloatBox2D {
            floor: "floor-1".into(),
            region: WorthQueryBox2DF64::from_f64(-100.0, -100.0, 100.0, 100.0),
        })
        .ensure_index_ready::<FloatBox2DIndex>()
        .unwrap();

    // Negative coordinate query matches only NegEquip.
    assert_eq!(
        keys(
            req.query(FindInFloatBox2D {
                floor: "floor-1".into(),
                region: WorthQueryBox2DF64::from_f64(-30.0, -15.0, -5.0, 1.0),
            })
            .execute()
            .unwrap()
            .rows()
        ),
        [1]
    );
    // Half-open y=[0.0, 5.0) does NOT overlap s1's y=[-20.0, -0.0), DOES overlap s2's y=[-5.0, 8.5).
    assert_eq!(
        keys(
            req.query(FindInFloatBox2D {
                floor: "floor-1".into(),
                region: WorthQueryBox2DF64::from_f64(-60.0, 0.0, 20.0, 5.0),
            })
            .execute()
            .unwrap()
            .rows()
        ),
        [2]
    );

    // Non-canonical float bits (e.g. non-canonical NaN payload) refuse at wire boundary.
    let non_canonical = WorthQueryBox2DF64 {
        min_x_bits: 0x7ff8_0000_0000_0001,
        min_y_bits: 0,
        max_x_bits: CanonicalF64::from_f64(10.0).0,
        max_y_bits: CanonicalF64::from_f64(10.0).0,
    };
    assert!(matches!(
        WorthQueryBox2DF64Binding::encode(&non_canonical),
        Err(ApplicationValueEncodeDenial::Canonical {
            cause: ValueCodecDenial::NonCanonical,
            ..
        })
    ));
    let mut raw = vec![1u8];
    for bits in [
        0x7ff8_0000_0000_0001u64,
        0,
        CanonicalF64::from_f64(10.0).0,
        CanonicalF64::from_f64(10.0).0,
    ] {
        raw.extend_from_slice(&bits.to_le_bytes());
    }
    let raw = ApplicationValue::InlineBytes(InlineBytes::try_new(raw).unwrap());
    assert!(matches!(
        WorthQueryBox2DF64Binding::decode(&raw),
        Err(ApplicationValueDecodeDenial::Canonical {
            cause: ValueCodecDenial::NonCanonical,
            ..
        })
    ));

    // Canonical NaN and reversed float probes refuse with Provider(InvalidProbeBounds).
    for bad in [
        WorthQueryBox2DF64::from_f64(f64::NAN, 0.0, 10.0, 10.0),
        WorthQueryBox2DF64::from_f64(10.0, 0.0, 5.0, 10.0),
    ] {
        match req
            .query(FindInFloatBox2D {
                floor: "floor-1".into(),
                region: bad,
            })
            .execute()
        {
            Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
                if matches!(
                    cause.cause(),
                    WorthQueryManagedIndexDenialCause::Work(
                        WorthQueryManagedIndexWorkDenial::Provider(_)
                    )
                ) => {}
            Err(other) => {
                panic!("invalid float probe must refuse with Provider denial, got {other:?}")
            }
            Ok(_) => panic!("invalid float probe must not succeed"),
        }
    }

    // Same-host post-refusal parity: index remains healthy and exact.
    assert_eq!(
        keys(
            req.query(FindInFloatBox2D {
                floor: "floor-1".into(),
                region: WorthQueryBox2DF64::from_f64(-30.0, -15.0, -5.0, 1.0),
            })
            .execute()
            .unwrap()
            .rows()
        ),
        [1]
    );
    idx.close();
}
