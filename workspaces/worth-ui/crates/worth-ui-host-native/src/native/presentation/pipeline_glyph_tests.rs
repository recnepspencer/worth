use wgpu::util::DeviceExt;

use super::text::{glyph_vertices, UiNativeGlyphCommand};
use super::{draw_presentation_operations, presentation_pipelines, UiNativeRasterOperation};
use crate::native::text_atlas::{
    UiNativeGpuAtlasKind, UiNativeTextAtlasGpuPages, UiNativeTextAtlasGpuUploadRequest,
    UiNativeTextAtlasUpload,
};

#[test]
fn qualified_dx12_pipeline_applies_alpha_foreground_and_preserves_intrinsic_color() {
    let (device, queue, info) = crate::native::text_atlas::qualified_test_device();
    let mut resources = crate::native::UiNativeResourceRegistry::new();
    let mut pages = UiNativeTextAtlasGpuPages::new();
    pages
        .ensure_page(&device, &mut resources, UiNativeGpuAtlasKind::Alpha)
        .unwrap();
    pages
        .ensure_page(&device, &mut resources, UiNativeGpuAtlasKind::Color)
        .unwrap();
    upload_page(
        &device,
        &queue,
        &mut resources,
        &mut pages,
        UiNativeGpuAtlasKind::Alpha,
        alpha_key(),
        vec![255; 16],
        4,
    );
    upload_page(
        &device,
        &queue,
        &mut resources,
        &mut pages,
        UiNativeGpuAtlasKind::Color,
        color_key(),
        [0, 255, 0, 255].repeat(16),
        16,
    );
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(super::GPU_WAIT_DEADLINE),
        })
        .unwrap();
    pages.settle_pending(&mut resources);

    let operations = [
        UiNativeRasterOperation::Glyph(command(
            alpha_key(),
            UiNativeGpuAtlasKind::Alpha,
            [0.0, 0.0, 8.0, 8.0],
            worth_ui_host_contract::UiMountedRgba8::new(255, 0, 0, 255),
        )),
        UiNativeRasterOperation::Glyph(command(
            color_key(),
            UiNativeGpuAtlasKind::Color,
            [8.0, 0.0, 8.0, 8.0],
            worth_ui_host_contract::UiMountedRgba8::new(0, 0, 255, 255),
        )),
    ];
    let vertices = operations
        .iter()
        .filter_map(|operation| match operation {
            UiNativeRasterOperation::Glyph(command) => Some(glyph_vertices(*command, [16, 8])),
            _ => None,
        })
        .flatten()
        .collect::<Vec<_>>();
    let vertex_bytes = encode_vertices(&vertices);
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("worth-ui-qualified-glyph-pixel-vertices"),
        contents: &vertex_bytes,
        usage: wgpu::BufferUsages::VERTEX,
    });
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("worth-ui-qualified-glyph-pixel-target"),
        size: wgpu::Extent3d {
            width: 16,
            height: 8,
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
        label: Some("worth-ui-qualified-glyph-pixel-readback"),
        size: 512,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let pipelines = presentation_pipelines(&device);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    draw_presentation_operations(
        &device,
        &mut encoder,
        &view,
        None,
        Some(&vertex_buffer),
        &operations,
        Some(&pages),
        &pipelines,
        true,
    );
    copy_pixel(&mut encoder, &target, &readback, [4, 4], 0);
    copy_pixel(&mut encoder, &target, &readback, [12, 4], 256);
    let submission = queue.submit([encoder.finish()]);
    let bytes = map_readback(&device, &readback, submission);
    assert_eq!(&bytes[..4], &[255, 0, 0, 255]);
    assert_eq!(&bytes[256..260], &[0, 255, 0, 255]);
    drop(bytes);
    readback.unmap();
    pages
        .try_close(&mut resources)
        .unwrap_or_else(|_| panic!("settled test atlas pages close"));
    assert!(resources.current().is_zero());
    println!(
        "WORTH_UI_HP03_GPU={:?}:alpha-red:intrinsic-green",
        info.backend
    );
}

fn upload_page(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    resources: &mut crate::native::UiNativeResourceRegistry,
    pages: &mut UiNativeTextAtlasGpuPages,
    kind: UiNativeGpuAtlasKind,
    key: worth_ui_host_contract::UiGlyphRasterKey,
    pixels: Vec<u8>,
    row_pitch: u32,
) {
    let upload =
        UiNativeTextAtlasUpload::from_text_mechanics(key, 4, 4, row_pitch, pixels, [7; 32]);
    pages
        .upload(UiNativeTextAtlasGpuUploadRequest {
            device,
            queue,
            resources,
            kind,
            page: 0,
            origin: [0, 0],
            upload: &upload,
        })
        .unwrap();
}

fn command(
    key: worth_ui_host_contract::UiGlyphRasterKey,
    atlas_kind: UiNativeGpuAtlasKind,
    target: [f32; 4],
    foreground: worth_ui_host_contract::UiMountedRgba8,
) -> UiNativeGlyphCommand {
    let mechanic =
        worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text_from_correspondence(
            worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
            0,
            None,
        );
    let clip = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: 0.0,
            y: 0.0,
            width: 16.0,
            height: 8.0,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
        },
    )
    .unwrap();
    let page_extent = match atlas_kind {
        UiNativeGpuAtlasKind::Alpha => 1_024.0,
        UiNativeGpuAtlasKind::Color => 2_048.0,
    };
    UiNativeGlyphCommand {
        run: worth_ui_host_contract::UiGlyphRunView::from_text_mechanics(
            worth_ui_host_contract::UiGlyphRunViewInput {
                mechanic,
                layout: worth_ui_host_contract::UiQualifiedTextLayoutIdentity::from_text_mechanics(
                    [3; 32],
                ),
                paint_span:
                    worth_ui_host_contract::UiMountedTextPaintSpanIdentity::from_runtime_mounting(
                        [4; 32],
                    ),
                original_range: worth_ui_host_contract::UiTextOriginalRange::new(0, 1).unwrap(),
                foreground,
                raster_key: key,
                origin_x_millipoints: 0,
                origin_y_millipoints: 0,
                line_index: 0,
                visual_run_index: 0,
                clip_bounds: clip,
                layer_semantic_order: 0,
            },
        ),
        atlas_kind,
        foreground,
        atlas_page: 0,
        target,
        texture_uv: [0.0, 0.0, 4.0 / page_extent, 4.0 / page_extent],
        opacity: 1.0,
    }
}

fn alpha_key() -> worth_ui_host_contract::UiGlyphRasterKey {
    key(
        worth_ui_host_contract::UiGlyphRasterSource::AlphaOutline,
        11,
    )
}

fn color_key() -> worth_ui_host_contract::UiGlyphRasterKey {
    key(worth_ui_host_contract::UiGlyphRasterSource::ColorBitmap, 12)
}

fn key(
    source: worth_ui_host_contract::UiGlyphRasterSource,
    glyph_id: u32,
) -> worth_ui_host_contract::UiGlyphRasterKey {
    worth_ui_host_contract::UiGlyphRasterKey::from_text_mechanics(
        worth_ui_host_contract::UiGlyphRasterKeyInput {
            font_collection: worth_ui_host_contract::UiFontCollectionGeneration::new(1).unwrap(),
            font_collection_lineage:
                worth_ui_host_contract::UiFontCollectionLineageIdentity::from_text_mechanics(
                    [1; 32],
                ),
            profile: worth_ui_host_contract::UiTextProfileGeneration::new(1).unwrap(),
            face: worth_ui_host_contract::UiQualifiedFontFaceIdentity::from_text_mechanics(
                [2; 32], 0,
            ),
            glyph_id,
            variations: worth_ui_host_contract::UiGlyphVariationCoordinates::empty(),
            palette: worth_ui_host_contract::UiGlyphRasterPalette::new(0),
            size: worth_ui_host_contract::UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
            source,
            dpi_milli: 1_000,
            origin: worth_ui_host_contract::UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
        },
    )
    .unwrap()
}

fn encode_vertices(vertices: &[super::GlyphVertex]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vertices.len() * 32);
    for vertex in vertices {
        for value in vertex
            .position
            .into_iter()
            .chain(vertex.texture_uv)
            .chain(vertex.color)
        {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
    }
    bytes
}

fn copy_pixel(
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    buffer: &wgpu::Buffer,
    origin: [u32; 2],
    offset: u64,
) {
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: origin[0],
                y: origin[1],
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset,
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

fn map_readback(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    submission: wgpu::SubmissionIndex,
) -> wgpu::BufferView {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    buffer.map_async(wgpu::MapMode::Read, .., move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(super::GPU_WAIT_DEADLINE),
        })
        .unwrap();
    receiver.recv().unwrap().unwrap();
    buffer.get_mapped_range(..)
}
