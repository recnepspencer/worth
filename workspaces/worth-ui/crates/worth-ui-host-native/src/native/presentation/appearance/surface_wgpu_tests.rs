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
                fill: UiMountedAppearanceColor::from_straight_srgba([10, 20, 30, 255]),
                border: UiMountedAppearanceColor::from_straight_srgba([220, 90, 30, 255]),
                inward_width: logical_length(2_000),
            },
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
        }),
        UiNativeAppearanceScale::qualified(1_000).unwrap(),
    )
    .unwrap();
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
    for (index, [x, y]) in [[7, 7], [56, 7], [56, 56], [7, 56]].into_iter().enumerate() {
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
    let pixels =
        [0, 256, 512, 768].map(|offset| <[u8; 4]>::try_from(&mapped[offset..offset + 4]).unwrap());
    assert!(
        pixels
            .iter()
            .all(|pixel| pixel[3] > 200 && pixel[0] > pixel[1]),
        "rounded border pixels: {pixels:?}"
    );
}
