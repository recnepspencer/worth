//! The retained frame lands pixel for pixel at the surface origin, with the
//! resize trace's frame stamp over its corner and nothing beyond its extent.
use super::{draw_retained_to_surface, presentation_pipelines, stamp_transfer};
use crate::native::resize_trace::{stamp_texels, STAMP_EXTENT};

/// Larger than the stamp on both axes.
const EXTENT: [u32; 2] = [192, 40];
/// Smaller than the stamp on both axes.
const NARROW: [u32; 2] = [100, 10];
/// A swapchain kept larger than [`EXTENT`] on both axes.
const SWAPCHAIN: [u32; 2] = [256, 64];
/// Row pitch of the readback, padded to the copy alignment.
const ROW_BYTES: u32 = 1024;
/// Magenta and black read the same through RGBA or BGRA order and sRGB
/// encoding, so the retained frame alternates them in a pattern no scaling
/// preserves.
const MAGENTA: [u8; 4] = [255, 0, 255, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];
const TRANSPARENT: [u8; 4] = [0; 4];

fn retained_texel(x: u32, y: u32) -> [u8; 4] {
    if x % 7 == y % 5 {
        MAGENTA
    } else {
        BLACK
    }
}

fn texture(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    extent: [u32; 2],
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("worth-ui-stamp-test"),
        size: wgpu::Extent3d {
            width: extent[0],
            height: extent[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

fn presented_pixels(frame: u64, extent: [u32; 2], target_extent: [u32; 2]) -> Vec<u8> {
    let (device, queue, _) = crate::native::text_atlas::qualified_test_device();
    let pipelines = presentation_pipelines(&device);
    let source = texture(
        &device,
        crate::native::graphics::qualified_target_format(),
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        extent,
    );
    queue.write_texture(
        source.as_image_copy(),
        &(0..extent[1])
            .flat_map(|y| (0..extent[0]).flat_map(move |x| retained_texel(x, y)))
            .collect::<Vec<_>>(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(extent[0] * 4),
            rows_per_image: Some(extent[1]),
        },
        source.size(),
    );
    let source_view = source.create_view(&wgpu::TextureViewDescriptor::default());
    let source_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("worth-ui-stamp-test-source"),
        layout: &pipelines.transfer.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&source_view),
        }],
    });
    let target = texture(
        &device,
        crate::native::graphics::qualified_surface_format(),
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        target_extent,
    );
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let stamp = stamp_transfer(&device, &queue, &pipelines.transfer, frame);
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("worth-ui-stamp-test-readback"),
        size: u64::from(ROW_BYTES * target_extent[1]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    draw_retained_to_surface(
        &mut encoder,
        &target_view,
        &pipelines.transfer,
        &source_group,
        extent,
        Some(&stamp),
    );
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ROW_BYTES),
                rows_per_image: Some(target_extent[1]),
            },
        },
        target.size(),
    );
    queue.submit([encoder.finish()]);
    let slice = readback.slice(..);
    slice.map_async(wgpu::MapMode::Read, |mapped| {
        mapped.expect("the stamp readback maps");
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(crate::native::presentation::GPU_WAIT_DEADLINE),
        })
        .expect("the stamp readback completes");
    slice.get_mapped_range().to_vec()
}

fn pixel(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = (y * ROW_BYTES + x * 4) as usize;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

fn assert_stamped(frame: u64, extent: [u32; 2], target_extent: [u32; 2]) {
    let bytes = presented_pixels(frame, extent, target_extent);
    let texels = stamp_texels(frame);
    for y in 0..target_extent[1] {
        for x in 0..target_extent[0] {
            let expected = if x >= extent[0] || y >= extent[1] {
                TRANSPARENT
            } else if x < STAMP_EXTENT[0] && y < STAMP_EXTENT[1] {
                let offset = ((y * STAMP_EXTENT[0] + x) * 4) as usize;
                [
                    texels[offset],
                    texels[offset + 1],
                    texels[offset + 2],
                    texels[offset + 3],
                ]
            } else {
                retained_texel(x, y)
            };
            assert_eq!(pixel(&bytes, x, y), expected, "pixel {x},{y}");
        }
    }
}

#[test]
fn the_stamp_replaces_only_its_own_pixels_at_the_origin() {
    assert_stamped(0x2_5a3c, EXTENT, EXTENT);
}

#[test]
fn a_target_smaller_than_the_stamp_shows_the_stamp_clipped() {
    assert_stamped(0x2_5a3c, NARROW, NARROW);
}

#[test]
fn a_larger_swapchain_shows_the_frame_unscaled_and_nothing_beyond_it() {
    assert_stamped(0x2_5a3c, EXTENT, SWAPCHAIN);
}
