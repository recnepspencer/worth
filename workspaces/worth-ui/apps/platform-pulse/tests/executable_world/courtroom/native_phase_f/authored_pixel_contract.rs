pub(super) struct AuthoredPixelContract {
    pub(super) alpha_bounds: Box<[[u32; 4]]>,
    pub(super) intrinsic_bounds: Box<[[u32; 4]]>,
}

pub(super) fn assert_owner_projection(evidence: &serde_json::Value) -> AuthoredPixelContract {
    let presentation = &evidence["presentation"];
    assert_eq!(
        presentation["client_physical_size"],
        serde_json::json!([240, 144])
    );
    assert_eq!(presentation["scale_factor_milli"], 1_500);
    let alpha = presentation["alpha_glyphs"]
        .as_array()
        .expect("native alpha glyph attribution");
    let alpha_bounds = EXPECTED_ALPHA
        .iter()
        .map(|expected| {
            let observed = exactly_one_alpha(alpha, expected);
            assert_eq!(observed["source"], "AlphaOutline");
            expected.bounds
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    let intrinsic = presentation["intrinsic_glyphs"]
        .as_array()
        .expect("native intrinsic glyph attribution");
    let matches = intrinsic
        .iter()
        .filter(|glyph| {
            glyph["original_range"] == serde_json::json!([8, 19])
                && glyph["glyph_id"] == 2_316
                && glyph["target_bounds"] == serde_json::json!([115, 24, 141, 48])
        })
        .collect::<Vec<_>>();
    let [observed] = matches.as_slice() else {
        panic!("the authored intrinsic glyph must project exactly once");
    };
    assert_eq!(observed["source"], "ColorBitmap");
    assert_eq!(observed["glyph_id"], 2_316);
    assert_eq!(observed["palette"], 0);
    assert_eq!(
        observed["target_bounds"],
        serde_json::json!([115, 24, 141, 48])
    );

    AuthoredPixelContract {
        alpha_bounds,
        intrinsic_bounds: Box::new([[115, 24, 141, 48]]),
    }
}

fn exactly_one_alpha<'glyph>(
    glyphs: &'glyph [serde_json::Value],
    expected: &ExpectedAlphaGlyph,
) -> &'glyph serde_json::Value {
    let matches = glyphs
        .iter()
        .filter(|glyph| {
            glyph["original_range"] == serde_json::json!(expected.original_range)
                && glyph["glyph_id"] == expected.glyph_id
                && glyph["target_bounds"] == serde_json::json!(expected.bounds)
        })
        .collect::<Vec<_>>();
    let [glyph] = matches.as_slice() else {
        panic!(
            "authored alpha glyph {} at {:?} in {:?} must project exactly once",
            expected.glyph_id, expected.original_range, expected.bounds
        );
    };
    glyph
}

struct ExpectedAlphaGlyph {
    glyph_id: u64,
    original_range: [u64; 2],
    bounds: [u32; 4],
}

// Final authored text is ASYNC-C. Raster extents are half-open pixel boxes.
const EXPECTED_ALPHA: [ExpectedAlphaGlyph; 7] = [
    ExpectedAlphaGlyph {
        glyph_id: 36,
        original_range: [0, 1],
        bounds: [24, 28, 38, 45],
    },
    ExpectedAlphaGlyph {
        glyph_id: 54,
        original_range: [1, 2],
        bounds: [38, 28, 48, 45],
    },
    ExpectedAlphaGlyph {
        glyph_id: 60,
        original_range: [2, 3],
        bounds: [48, 28, 61, 45],
    },
    ExpectedAlphaGlyph {
        glyph_id: 49,
        original_range: [3, 4],
        bounds: [62, 28, 75, 45],
    },
    ExpectedAlphaGlyph {
        glyph_id: 38,
        original_range: [4, 5],
        bounds: [78, 28, 90, 45],
    },
    ExpectedAlphaGlyph {
        glyph_id: 16,
        original_range: [5, 6],
        bounds: [90, 37, 96, 40],
    },
    ExpectedAlphaGlyph {
        glyph_id: 38,
        original_range: [6, 7],
        bounds: [98, 28, 110, 45],
    },
];
