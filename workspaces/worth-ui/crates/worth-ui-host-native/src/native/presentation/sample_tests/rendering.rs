use wgpu::util::DeviceExt;
use worth_ui_host_contract::UiMountedPortalOverlayMechanic;

use crate::native::presentation::{
    draw_presentation_operations, presentation_pipelines,
    raster::{raster_damage_for_basis, UiNativeRasterBasis},
    rectangle_vertices, UiNativePresentationPortPlan, UiNativeRasterOperation,
};

pub(super) fn render_sample_pixels(
    basis: UiNativeRasterBasis,
    portal: UiMountedPortalOverlayMechanic,
    plan: UiNativePresentationPortPlan,
) -> [[u8; 4]; 3] {
    let (device, queue, _) = crate::native::text_atlas::qualified_test_device();
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("worth-ui-sample-pixel-target"),
        size: wgpu::Extent3d {
            width: 80,
            height: 32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("worth-ui-sample-pixel-readback"),
        size: 768,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let initial_rect = raster_damage_for_basis(portal.clip_bounds(), basis)
        .unwrap()
        .unwrap();
    let initial = [UiNativeRasterOperation::FilledRect {
        rect: initial_rect,
        source_rgba8: [220, 40, 20, 255],
    }];
    let initial_buffer = raster_vertex_buffer(&device, &initial, "worth-ui-sample-initial");
    let sample_buffer = raster_vertex_buffer(&device, &plan.operations, "worth-ui-sample-derived");
    let pipelines = presentation_pipelines(&device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    draw_presentation_operations(
        &device,
        &mut encoder,
        &view,
        Some(&initial_buffer),
        None,
        &initial,
        None,
        &pipelines,
        true,
    );
    draw_presentation_operations(
        &device,
        &mut encoder,
        &view,
        Some(&sample_buffer),
        None,
        &plan.operations,
        None,
        &pipelines,
        false,
    );
    for (index, [x, y]) in [[20, 10], [50, 10], [42, 10]].into_iter().enumerate() {
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
            timeout: Some(crate::native::presentation::GPU_WAIT_DEADLINE),
        })
        .unwrap();
    receiver.recv().unwrap().unwrap();
    let bytes = readback.get_mapped_range(..);
    let pixels = [0, 256, 512].map(|offset| bytes[offset..offset + 4].try_into().unwrap());
    drop(bytes);
    readback.unmap();
    pixels
}

fn raster_vertex_buffer(
    device: &wgpu::Device,
    operations: &[UiNativeRasterOperation],
    label: &str,
) -> wgpu::Buffer {
    let mut bytes = Vec::new();
    for operation in operations {
        let vertices = match *operation {
            UiNativeRasterOperation::Clear(rect) => rectangle_vertices(rect, [0, 0, 0, 0]),
            UiNativeRasterOperation::FilledRect { rect, source_rgba8 } => {
                rectangle_vertices(rect, source_rgba8)
            }
            UiNativeRasterOperation::Glyph(_) => continue,
            UiNativeRasterOperation::Surface(_) => continue,
        };
        for vertex in vertices {
            for value in vertex.position.into_iter().chain(vertex.color) {
                bytes.extend_from_slice(&value.to_ne_bytes());
            }
        }
    }
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: &bytes,
        usage: wgpu::BufferUsages::VERTEX,
    })
}
