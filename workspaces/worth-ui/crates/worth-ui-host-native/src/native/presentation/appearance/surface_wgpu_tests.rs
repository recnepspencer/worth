use super::mounted_mechanic_fixtures::{
    allocation, logical_length, mounted_surface, MountedSurfaceFixtureInput,
};
use super::{UiNativeAppearanceScale, UiNativeSurfacePipeline};
use crate::native::presentation::{
    draw_presentation_operations, presentation_pipelines, rectangle_vertices,
    UiNativeRasterOperation, GPU_WAIT_DEADLINE,
};
use wgpu::util::DeviceExt;
use worth_ui_host_contract::{
    UiAppearanceClip, UiMountedAppearanceColor, UiMountedPresentationOpacity, UiMountedSurfacePaint,
};

#[test]
fn production_shader_paints_the_border_through_all_four_rounded_corners() {
    let primitive = UiNativeSurfacePipeline::prepare(
        &mounted_surface(MountedSurfaceFixtureInput {
            allocation: allocation(0, 0, 64_000, 64_000),
            clip: UiAppearanceClip::new(0, 0, 64_000, 64_000).unwrap(),
            radii: [logical_length(24_000); 4],
            paint: UiMountedSurfacePaint::FillAndBorder {
                fill: UiMountedAppearanceColor::from_straight_srgba([10, 20, 30, 255]).into(),
                border: UiMountedAppearanceColor::from_straight_srgba([220, 90, 30, 255]),
                inward_width: logical_length(2_000),
            },
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
        }),
        UiNativeAppearanceScale::qualified(1_000).unwrap(),
    )
    .unwrap();
    let pixels = render_samples(&primitive, [[7, 7], [56, 7], [56, 56], [7, 56]]);
    assert!(
        pixels
            .iter()
            .all(|pixel| pixel[3] > 200 && pixel[0] > pixel[1]),
        "rounded border pixels: {pixels:?}"
    );
}

#[test]
fn production_gradient_shader_preserves_allocation_axis_under_clipping_and_opacity() {
    use worth_ui_host_contract::{NormalizedPoint, UiMountedLinearGradient, UiMountedSurfaceFill};
    let fill = UiMountedSurfaceFill::LinearGradient(
        UiMountedLinearGradient::new(
            NormalizedPoint::new(0, 0).unwrap(),
            NormalizedPoint::new(10_000, 0).unwrap(),
            [
                UiMountedAppearanceColor::from_straight_srgba([255, 0, 0, 255]),
                UiMountedAppearanceColor::from_straight_srgba([0, 0, 255, 0]),
            ],
        )
        .unwrap(),
    );
    let primitive = UiNativeSurfacePipeline::prepare(
        &mounted_surface(MountedSurfaceFixtureInput {
            allocation: allocation(0, 0, 64_000, 64_000),
            clip: UiAppearanceClip::new(16_000, 0, 48_000, 64_000).unwrap(),
            radii: [logical_length(0); 4],
            paint: UiMountedSurfacePaint::Fill(fill),
            opacity: UiMountedPresentationOpacity::from_runtime_composition(32_768),
        }),
        UiNativeAppearanceScale::qualified(1_000).unwrap(),
    )
    .unwrap();
    let pixels = render_samples(&primitive, [[8, 32], [16, 32], [32, 32], [63, 32]]);
    // At x=16.5, alpha=(1-16.5/64)/2, not 1/2 at the clip origin.
    // RGB is premultiplied red; transparent blue must add no blue fringe.
    assert_eq!(pixels[0], [0; 4]);
    for (pixel, expected_alpha) in pixels[1..].iter().zip([95_u8, 63, 1]) {
        assert!(pixel[3].abs_diff(expected_alpha) <= 1, "{pixels:?}");
        assert_eq!([pixel[1], pixel[2]], [0, 0], "{pixels:?}");
    }
}

fn render_samples(
    primitive: &super::surface_pipeline::UiNativeSurfacePrimitive,
    points: [[u32; 2]; 4],
) -> [[u8; 4]; 4] {
    let surface = primitive.raster_operation([64, 64]).unwrap().unwrap();
    let rect = surface.rect();
    let operations = [UiNativeRasterOperation::Surface(surface)];
    let (device, queue, _) = crate::native::text_atlas::qualified_test_device();
    let vertices = rectangle_vertices(rect, [0; 4]);
    let mut bytes = Vec::new();
    for vertex in vertices {
        for value in vertex.position.into_iter().chain(vertex.color) {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
    }
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("worth-ui-rounded-surface-vertices"),
        contents: &bytes,
        usage: wgpu::BufferUsages::VERTEX,
    });
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("worth-ui-rounded-surface-target"),
        size: wgpu::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("worth-ui-rounded-surface-readback"),
        size: 1_024,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    draw_presentation_operations(
        &device,
        &mut encoder,
        &target.create_view(&wgpu::TextureViewDescriptor::default()),
        Some(&vertex_buffer),
        None,
        &operations,
        None,
        &presentation_pipelines(&device),
        true,
    );
    for (index, [x, y]) in points.into_iter().enumerate() {
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: (index * 256) as u64,
                    bytes_per_row: Some(256),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }
    let submission = queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    readback.map_async(wgpu::MapMode::Read, .., move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(GPU_WAIT_DEADLINE),
        })
        .unwrap();
    receiver.recv().unwrap().unwrap();
    let mapped = readback.get_mapped_range(..);
    [0, 256, 512, 768].map(|offset| <[u8; 4]>::try_from(&mapped[offset..offset + 4]).unwrap())
}

#[test]
fn production_shader_renders_vector_holes_round_strokes_and_finite_shadow_falloff() {
    use worth_ui_host_contract::{
        NormalizedPoint as P, UiSoftShadowGeometry, UiSurfaceGeometry as G,
        UiVectorSurfaceGeometry as V, VectorPath, VectorPathSegment as S,
    };
    let p = |x, y| P::new(x, y).unwrap();
    let path = VectorPath::new(vec![
        S::MoveTo(p(0, 0)),
        S::LineTo(p(10_000, 0)),
        S::LineTo(p(10_000, 10_000)),
        S::LineTo(p(0, 10_000)),
        S::Close,
        S::MoveTo(p(2_500, 2_500)),
        S::LineTo(p(7_500, 2_500)),
        S::LineTo(p(7_500, 7_500)),
        S::LineTo(p(2_500, 7_500)),
        S::Close,
    ])
    .unwrap();
    let render = |geometry, points| {
        let mechanic = super::mounted_mechanic_fixtures::mounted_surface_geometry(
            MountedSurfaceFixtureInput {
                allocation: allocation(0, 0, 64_000, 64_000),
                clip: UiAppearanceClip::new(0, 0, 64_000, 64_000).unwrap(),
                radii: [logical_length(0); 4],
                paint: UiMountedSurfacePaint::Fill(
                    UiMountedAppearanceColor::from_straight_srgba([120, 60, 240, 255]).into(),
                ),
                opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            },
            worth_ui_host_contract::UiMountedSurfaceBorderEdges::ALL,
            geometry,
        );
        let primitive = UiNativeSurfacePipeline::prepare(
            &mechanic,
            UiNativeAppearanceScale::qualified(1_000).unwrap(),
        )
        .unwrap();
        render_samples(&primitive, points)
    };
    // An independently specified square ring: central pixels must remain clear.
    let pixels = render(
        G::Vector(V::fill(path).unwrap()),
        [[8, 8], [32, 32], [55, 55], [20, 20]],
    );
    assert_eq!(pixels.map(|p| p[3]), [255, 0, 255, 0]);
    let line = VectorPath::new(vec![S::MoveTo(p(0, 5_000)), S::LineTo(p(10_000, 5_000))]).unwrap();
    let pixels = render(
        G::Vector(V::stroke(line, logical_length(4_000)).unwrap()),
        [[2, 32], [32, 32], [32, 28], [61, 32]],
    );
    assert_eq!(pixels.map(|p| p[3]), [255, 255, 0, 255]);
    // Sigma=4, caster starts at x=12: successive outside samples must fade.
    let pixels = render(
        G::SoftShadow(
            UiSoftShadowGeometry::new(logical_length(4_000), logical_length(4_000)).unwrap(),
        ),
        [[32, 32], [10, 32], [6, 32], [0, 32]],
    );
    assert_eq!(pixels[0][3], 255);
    assert!(
        pixels[1][3] > pixels[2][3] && pixels[2][3] > pixels[3][3],
        "shadow: {pixels:?}"
    );
    assert!(pixels[3][3] <= 1);
}
