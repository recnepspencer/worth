pub(super) fn parse_control_points(argument: &str) -> Option<Vec<[u32; 2]>> {
    let points = argument
        .split(';')
        .map(|point| {
            let (x, y) = point.split_once(',')?;
            Some([x.parse().ok()?, y.parse().ok()?])
        })
        .collect::<Option<Vec<_>>>()?;
    (!points.is_empty() && points.len() <= 16).then_some(points)
}

pub(super) fn evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
    logical_points: &[[u32; 2]],
) -> Option<serde_json::Value> {
    let snapshot = receipt.visual_snapshot()?;
    let samples = logical_points
        .iter()
        .copied()
        .map(|logical| sample(snapshot, logical))
        .collect::<Option<Vec<_>>>()?;
    let affinity = snapshot.affinity();
    Some(serde_json::json!({
        "schema": "worth-ui-native-phase7-evidence-v1",
        "snapshot": {
            "identity": affinity[0],
            "presentation_attempt": affinity[1],
            "frame": affinity[2],
            "semantic_surface": affinity[3],
            "host_surface": affinity[4],
            "binding": affinity[5],
            "presentation_epoch": affinity[6],
            "relation": relation(snapshot.relation()),
            "native_client_origin": snapshot.native_client_origin(),
            "client_physical_dimensions": snapshot.client_physical_dimensions(),
            "viewport_logical_dimension_bits": snapshot.viewport_logical_dimension_bits(),
            "scale_bits": snapshot.scale_bits(),
            "translation_bits": snapshot.translation_bits(),
            "orientation": orientation(snapshot.coordinate_orientation()),
            "rounding": rounding(snapshot.coordinate_rounding()),
            "pixel_dimensions": snapshot.pixel_dimensions(),
            "pixel_stride": snapshot.pixel_stride(),
            "pixel_byte_count": snapshot.pixel_bytes().len(),
            "pixel_color_space": color_space(snapshot.pixel_color_space()),
            "visible_region_count": snapshot.visible_region_count(),
            "hit_test_region_count": snapshot.hit_test_region_count(),
            "cost_counters": snapshot.cost_counters(),
            "samples": samples,
        },
        "presentation": {
            "frame": receipt.presentation().map(|value| value.presented_frame()),
            "semantic_surface": receipt.presentation().map(|value| value.semantic_surface()),
            "host_surface": receipt.presentation().map(|value| value.host_surface()),
            "binding": receipt.presentation().map(|value| value.binding_generation()),
            "presentation_attempt": receipt.presentation().map(|value| value.presentation_attempt()),
        },
        "capture_resources": {
            "peak_readback_buffers": receipt.peak_census().readback_buffers,
            "peak_pending_submissions": receipt.peak_census().pending_submissions,
            "terminal_readback_buffers": receipt.terminal_census().readback_buffers,
            "terminal_pending_submissions": receipt.terminal_census().pending_submissions,
        },
        "terminal_zero": receipt.terminal_census().is_zero(),
    }))
}

fn sample(
    snapshot: &worth_ui_native_platform::UiNativeClientVisualSnapshotObservation,
    logical: [u32; 2],
) -> Option<serde_json::Value> {
    let scale = snapshot.scale_bits().map(f32::from_bits);
    let translation = snapshot.translation_bits().map(f32::from_bits);
    let dimensions = snapshot.pixel_dimensions();
    let x = project(logical[0] as f32 - translation[0], scale[0], snapshot)?;
    let projected_y = project(logical[1] as f32 - translation[1], scale[1], snapshot)?;
    let y = match snapshot.coordinate_orientation() {
        worth_ui_native_platform::UiNativeClientVisualCoordinateOrientation::TopLeftOrigin => {
            projected_y
        }
        worth_ui_native_platform::UiNativeClientVisualCoordinateOrientation::BottomLeftOrigin => {
            dimensions[1].checked_sub(projected_y.checked_add(1)?)?
        }
    };
    if x >= dimensions[0] || y >= dimensions[1] {
        return None;
    }
    let offset = usize::try_from(y)
        .ok()?
        .checked_mul(usize::try_from(snapshot.pixel_stride()).ok()?)?
        .checked_add(usize::try_from(x).ok()?.checked_mul(4)?)?;
    let rgba: [u8; 4] = snapshot
        .pixel_bytes()
        .get(offset..offset + 4)?
        .try_into()
        .ok()?;
    Some(serde_json::json!({
        "logical": logical,
        "physical": [x, y],
        "rgba": rgba,
    }))
}

fn project(
    logical: f32,
    scale: f32,
    snapshot: &worth_ui_native_platform::UiNativeClientVisualSnapshotObservation,
) -> Option<u32> {
    let physical = logical * scale;
    if !physical.is_finite() || physical < 0.0 || physical > u32::MAX as f32 {
        return None;
    }
    Some(match snapshot.coordinate_rounding() {
        worth_ui_native_platform::UiNativeClientVisualCoordinateRounding::PixelCenterNearest => {
            physical.round() as u32
        }
        worth_ui_native_platform::UiNativeClientVisualCoordinateRounding::FloorEdges => {
            physical.floor() as u32
        }
    })
}

fn relation(value: worth_ui_native_platform::UiNativeClientVisualSnapshotRelation) -> &'static str {
    match value {
        worth_ui_native_platform::UiNativeClientVisualSnapshotRelation::Current => "current",
        worth_ui_native_platform::UiNativeClientVisualSnapshotRelation::RetainedPredecessor => {
            "retained-predecessor"
        }
        worth_ui_native_platform::UiNativeClientVisualSnapshotRelation::Historical => "historical",
    }
}

fn orientation(
    value: worth_ui_native_platform::UiNativeClientVisualCoordinateOrientation,
) -> &'static str {
    match value {
        worth_ui_native_platform::UiNativeClientVisualCoordinateOrientation::TopLeftOrigin => {
            "top-left-origin"
        }
        worth_ui_native_platform::UiNativeClientVisualCoordinateOrientation::BottomLeftOrigin => {
            "bottom-left-origin"
        }
    }
}

fn rounding(
    value: worth_ui_native_platform::UiNativeClientVisualCoordinateRounding,
) -> &'static str {
    match value {
        worth_ui_native_platform::UiNativeClientVisualCoordinateRounding::PixelCenterNearest => {
            "pixel-center-nearest"
        }
        worth_ui_native_platform::UiNativeClientVisualCoordinateRounding::FloorEdges => {
            "floor-edges"
        }
    }
}

fn color_space(
    value: worth_ui_native_platform::UiNativeClientVisualPixelColorSpace,
) -> &'static str {
    match value {
        worth_ui_native_platform::UiNativeClientVisualPixelColorSpace::Srgb => "srgb",
        worth_ui_native_platform::UiNativeClientVisualPixelColorSpace::AdapterDeclared => {
            "adapter-declared"
        }
    }
}
