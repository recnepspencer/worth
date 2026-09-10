use crate::native::UiNativePresentationAccess;
use wgpu::util::DeviceExt;

#[path = "pipeline/shaders.rs"]
mod shaders;
use shaders::{
    ALPHA_GLYPH_SHADER, ANALYTIC_SURFACE_SHADER, COLOR_GLYPH_SHADER, RASTER_SHADER,
    RETAINED_TO_SURFACE_SHADER,
};

const RASTER_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
    wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];
const GLYPH_ATTRIBUTES: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

pub(super) struct UiNativePresentationPipelines {
    filled: wgpu::RenderPipeline,
    clearing: wgpu::RenderPipeline,
    surface: wgpu::RenderPipeline,
    alpha: wgpu::RenderPipeline,
    color: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
}

fn transfer_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    pipeline_with_blend(device, shader, format, None, &[])
}

pub(super) fn presentation_pipelines(device: &wgpu::Device) -> UiNativePresentationPipelines {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("worth-ui-retained-raster"),
        source: wgpu::ShaderSource::Wgsl(RASTER_SHADER.into()),
    });
    let buffers = [wgpu::VertexBufferLayout {
        array_stride: 24,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &RASTER_ATTRIBUTES,
    }];
    let blended = pipeline_with_blend(
        device,
        &shader,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        &buffers,
    );
    let replacing = pipeline_with_blend(
        device,
        &shader,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        Some(wgpu::BlendState::REPLACE),
        &buffers,
    );
    let surface_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("worth-ui-analytic-surface"),
        source: wgpu::ShaderSource::Wgsl(ANALYTIC_SURFACE_SHADER.into()),
    });
    let surface = pipeline_with_blend(
        device,
        &surface_shader,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        &buffers,
    );
    let glyph_buffers = [wgpu::VertexBufferLayout {
        array_stride: 32,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &GLYPH_ATTRIBUTES,
    }];
    let alpha_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("worth-ui-alpha-glyph"),
        source: wgpu::ShaderSource::Wgsl(ALPHA_GLYPH_SHADER.into()),
    });
    let color_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("worth-ui-color-glyph"),
        source: wgpu::ShaderSource::Wgsl(COLOR_GLYPH_SHADER.into()),
    });
    let alpha = pipeline_with_blend(
        device,
        &alpha_shader,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        &glyph_buffers,
    );
    let color = pipeline_with_blend(
        device,
        &color_shader,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        &glyph_buffers,
    );
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("worth-ui-text-atlas-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    UiNativePresentationPipelines {
        filled: blended,
        clearing: replacing,
        surface,
        alpha,
        color,
        sampler,
    }
}

fn pipeline_with_blend(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
    buffers: &[wgpu::VertexBufferLayout<'_>],
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("worth-ui-filled-rectangle-pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers,
            compilation_options: Default::default(),
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

pub(super) fn draw_presentation_operations(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    raster_vertex_buffer: Option<&wgpu::Buffer>,
    glyph_vertex_buffer: Option<&wgpu::Buffer>,
    operations: &[super::UiNativeRasterOperation],
    atlas: Option<&crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
    pipelines: &UiNativePresentationPipelines,
    clear_target: bool,
) {
    let surface_bind_groups = operations
        .iter()
        .map(|operation| match operation {
            super::UiNativeRasterOperation::Surface(surface) => {
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("worth-ui-analytic-surface-data"),
                    contents: &surface.storage_bytes(),
                    usage: wgpu::BufferUsages::STORAGE,
                });
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("worth-ui-analytic-surface-bind-group"),
                    layout: &pipelines.surface.get_bind_group_layout(0),
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });
                Some((buffer, bind_group))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let glyph_bind_groups = operations
        .iter()
        .map(|operation| match operation {
            super::UiNativeRasterOperation::Glyph(command) => {
                let pages = atlas.expect("admitted glyph command retains native atlas pages");
                let (view, _) = pages
                    .page_view(command.atlas_kind, command.atlas_page)
                    .expect("admitted glyph command retains its exact atlas page");
                let pipeline = if super::text::source_is_intrinsic_color(*command) {
                    &pipelines.color
                } else {
                    &pipelines.alpha
                };
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("worth-ui-glyph-atlas-bind-group"),
                    layout: &pipeline.get_bind_group_layout(0),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&pipelines.sampler),
                        },
                    ],
                }))
            }
            super::UiNativeRasterOperation::Clear(_)
            | super::UiNativeRasterOperation::FilledRect { .. }
            | super::UiNativeRasterOperation::Surface(_) => None,
        })
        .collect::<Vec<_>>();
    let attachments = [Some(wgpu::RenderPassColorAttachment {
        view: target,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: if clear_target {
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
            } else {
                wgpu::LoadOp::Load
            },
            store: wgpu::StoreOp::Store,
        },
    })];
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("worth-ui-retained-raster-pass"),
        color_attachments: &attachments,
        ..Default::default()
    });
    let mut raster_index = 0_usize;
    let mut glyph_index = 0_usize;
    for (operation_index, operation) in operations.iter().enumerate() {
        match operation {
            super::UiNativeRasterOperation::Clear(_)
            | super::UiNativeRasterOperation::FilledRect { .. } => {
                let buffer = raster_vertex_buffer.expect("raster operation retains vertices");
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.set_pipeline(
                    if matches!(operation, super::UiNativeRasterOperation::Clear(_)) {
                        &pipelines.clearing
                    } else {
                        &pipelines.filled
                    },
                );
                let start = u32::try_from(raster_index * 6)
                    .expect("profile-bounded raster vertices fit u32");
                pass.draw(start..start + 6, 0..1);
                raster_index += 1;
            }
            super::UiNativeRasterOperation::Surface(_) => {
                let buffer = raster_vertex_buffer.expect("surface operation retains vertices");
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.set_pipeline(&pipelines.surface);
                pass.set_bind_group(
                    0,
                    &surface_bind_groups[operation_index]
                        .as_ref()
                        .expect("surface operation retains one bind group")
                        .1,
                    &[],
                );
                let start = u32::try_from(raster_index * 6)
                    .expect("profile-bounded surface vertices fit u32");
                pass.draw(start..start + 6, 0..1);
                raster_index += 1;
            }
            super::UiNativeRasterOperation::Glyph(command) => {
                let buffer = glyph_vertex_buffer.expect("glyph operation retains vertices");
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.set_pipeline(if super::text::source_is_intrinsic_color(*command) {
                    &pipelines.color
                } else {
                    &pipelines.alpha
                });
                pass.set_bind_group(
                    0,
                    glyph_bind_groups[operation_index]
                        .as_ref()
                        .expect("glyph operation retains one bind group"),
                    &[],
                );
                let start =
                    u32::try_from(glyph_index * 6).expect("profile-bounded glyph vertices fit u32");
                pass.draw(start..start + 6, 0..1);
                glyph_index += 1;
            }
        }
    }
}

pub(super) fn retained_transfer(
    graphics: &UiNativePresentationAccess,
) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
    let shader = graphics
        .device()
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("worth-ui-retained-to-surface"),
            source: wgpu::ShaderSource::Wgsl(RETAINED_TO_SURFACE_SHADER.into()),
        });
    let pipeline = transfer_pipeline(
        graphics.device(),
        &shader,
        wgpu::TextureFormat::Bgra8UnormSrgb,
    );
    let view = graphics
        .retained_target()
        .create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = graphics
        .device()
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("worth-ui-retained-to-surface-bind-group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });
    (pipeline, bind_group)
}

pub(super) fn draw_retained_to_surface(
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    pipeline: &wgpu::RenderPipeline,
    bind_group: &wgpu::BindGroup,
) {
    let attachments = [Some(wgpu::RenderPassColorAttachment {
        view: target,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
        },
    })];
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("worth-ui-retained-to-surface-pass"),
        color_attachments: &attachments,
        ..Default::default()
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.draw(0..3, 0..1);
}
