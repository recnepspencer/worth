//! `CanonicalF64` 2D box court: negative coordinates, signed zero, NaN/reversed probe refusal,
//! wire non-canonicality refusal, and same-host post-refusal parity.
use super::operations::AssetSpec;
use super::schema::{FindInFloatBox2D, FloatBox2DIndex};
use super::{create_asset, install, keys};
use worth_query_decl::facade::application_query::ApplicationQueryInputPreparationDenial;
use worth_query_decl::facade::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ApplicationValue,
    ApplicationValueDecodeDenial, ApplicationValueEncodeDenial, CanonicalF64, InlineBytes,
    ValueCodecDenial,
};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::ordered_index::{
    WorthQueryBox2DF64, WorthQueryBox2DF64Binding, WorthQueryBox3DF64, WorthQueryBox3DF64Binding,
};
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

    // Signed-zero canonicalization: opening a watch with raw `-0.0` bits and
    // polling with `+0.0` encodes identical wire bytes (zero ForeignContext refusal).
    let neg_zero_raw_probe = WorthQueryBox2DF64 {
        min_x_bits: (-0.0_f64).to_bits(),
        min_y_bits: CanonicalF64::from_f64(-10.0).0,
        max_x_bits: CanonicalF64::from_f64(20.0).0,
        max_y_bits: CanonicalF64::from_f64(10.0).0,
    };
    let pos_zero_probe = WorthQueryBox2DF64::from_f64(0.0, -10.0, 20.0, 10.0);
    assert_eq!(
        WorthQueryBox2DF64::from_f64(-0.0, -10.0, 20.0, 10.0),
        pos_zero_probe
    );
    assert_eq!(
        WorthQueryBox2DF64Binding::encode(&neg_zero_raw_probe).unwrap(),
        WorthQueryBox2DF64Binding::encode(&pos_zero_probe).unwrap(),
    );
    let (init_w, mut float_watch) = req
        .query(FindInFloatBox2D {
            floor: "floor-1".into(),
            region: neg_zero_raw_probe,
        })
        .watch_index::<FloatBox2DIndex>()
        .unwrap();
    assert_eq!(keys(init_w.rows()), [2]);

    let mut s3 = AssetSpec::i64_only(3, "ZeroEdgeEquip", (0, 10), [0, 0, 0], [1, 1, 1]);
    s3.fmin_bits = [
        CanonicalF64::from_f64(1.0).0,
        CanonicalF64::from_f64(-1.0).0,
    ];
    s3.fmax_bits = [CanonicalF64::from_f64(5.0).0, CanonicalF64::from_f64(3.0).0];
    create_asset(&host, s3, 77);
    let WorthQueryManagedIndexWatchOutcome::Delivered(upd) = req
        .query(FindInFloatBox2D {
            floor: "floor-1".into(),
            region: pos_zero_probe,
        })
        .next_watch(&mut float_watch)
        .unwrap()
    else {
        panic!("polling +0.0 on watch opened with -0.0 must deliver without ForeignContext");
    };
    assert_eq!(keys(upd.read().rows()), [2, 3]);
    drop((init_w, upd));
    float_watch.close().unwrap();

    // Non-canonical float bits (raw -0.0, canonical/non-canonical NaN, +/-Infinity)
    // refuse directly at the codec boundary on BOTH encode and decode.
    for bad_bits in [
        0x7ff8_0000_0000_0001u64,
        CanonicalF64::from_f64(f64::NAN).0,
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
    ] {
        let bad_probe = WorthQueryBox2DF64 {
            min_x_bits: bad_bits,
            min_y_bits: 0,
            max_x_bits: CanonicalF64::from_f64(10.0).0,
            max_y_bits: CanonicalF64::from_f64(10.0).0,
        };
        assert!(matches!(
            WorthQueryBox2DF64Binding::encode(&bad_probe),
            Err(ApplicationValueEncodeDenial::Canonical {
                cause: ValueCodecDenial::NonCanonical,
                ..
            })
        ));
    }
    for bad_wire_bits in [
        (-0.0_f64).to_bits(),
        0x7ff8_0000_0000_0001u64,
        CanonicalF64::from_f64(f64::NAN).0,
        f64::INFINITY.to_bits(),
        f64::NEG_INFINITY.to_bits(),
    ] {
        let mut raw = vec![1u8];
        for bits in [
            bad_wire_bits,
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
    }
    // 3D float probe enforces the exact same signed-zero & finite-only codec policy.
    let box3_neg = WorthQueryBox3DF64::from_f64([-0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    let box3_pos = WorthQueryBox3DF64::from_f64([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    assert_eq!(box3_neg, box3_pos);
    assert!(matches!(
        WorthQueryBox3DF64Binding::encode(&WorthQueryBox3DF64::from_f64(
            [f64::NEG_INFINITY, 0.0, 0.0],
            [1.0, 1.0, 1.0],
        )),
        Err(ApplicationValueEncodeDenial::Canonical {
            cause: ValueCodecDenial::NonCanonical,
            ..
        })
    ));

    // Non-finite query probes refuse at parameter encoding with NonCanonical;
    // reversed finite probes refuse at adapter validate_probe with Provider denial.
    for bad_non_finite in [
        WorthQueryBox2DF64::from_f64(f64::NAN, 0.0, 10.0, 10.0),
        WorthQueryBox2DF64::from_f64(0.0, 0.0, f64::INFINITY, 10.0),
        WorthQueryBox2DF64::from_f64(f64::NEG_INFINITY, 0.0, 10.0, 10.0),
    ] {
        assert!(matches!(
            req.query(FindInFloatBox2D {
                floor: "floor-1".into(),
                region: bad_non_finite,
            })
            .execute(),
            Err(WorthQueryApplicationRequestQueryDenial::InputPreparation(
                ApplicationQueryInputPreparationDenial::ParameterEncoding(
                    ApplicationValueEncodeDenial::Canonical {
                        cause: ValueCodecDenial::NonCanonical,
                        ..
                    }
                )
            ))
        ));
    }
    match req
        .query(FindInFloatBox2D {
            floor: "floor-1".into(),
            region: WorthQueryBox2DF64::from_f64(10.0, 0.0, 5.0, 10.0),
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
            panic!("reversed float probe must refuse with Provider denial, got {other:?}")
        }
        Ok(_) => panic!("reversed float probe must not succeed"),
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
