use crate::native::UiNativePresentationAccess;
use wgpu::util::DeviceExt;

use super::super::{
    copy_evidence_pixels, draw_presentation_operations, draw_retained_to_surface,
    presentation_pipelines, rectangle_vertices, retained_transfer, GlyphVertex, RasterVertex,
    UiNativePendingWgpuObligation, UiNativePresentationPipelines, UiNativeWgpuReadbackPoll,
};
use super::orchestrator::UiNativePresentationStagePort;
use super::phase::{
    UiNativeEncodedPresentation, UiNativePreparedPresentation, UiNativePresentHandoff,
    UiNativeSubmittedPresentation, UiNativeSurfaceAcquiredPresentation,
};
use super::{
    UiNativePresentationPortFailure, UiNativePresentationPortObservation,
    UiNativePresentationPortPlan, UiNativeRasterOperation,
};

pub(super) fn present(
    graphics: &mut UiNativePresentationAccess,
    atlas: Option<&crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
    plan: UiNativePresentationPortPlan,
    defer_initial_observation: bool,
    lifecycle: &mut crate::native::lifecycle::UiNativeLifecycleOrchestrator,
) -> Result<UiNativePresentationPortObservation, UiNativePresentationPortFailure> {
    let mut transaction = UiWgpuPresentationTransaction {
        graphics,
        atlas,
        plan: Some(plan),
        defer_initial_observation,
    };
    lifecycle.run_presentation(&mut transaction)
}

struct UiWgpuPresentationTransaction<'transaction, 'owners> {
    graphics: &'transaction mut UiNativePresentationAccess<'owners>,
    atlas: Option<&'transaction crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
    plan: Option<UiNativePresentationPortPlan>,
    defer_initial_observation: bool,
}

impl UiNativePresentationStagePort for UiWgpuPresentationTransaction<'_, '_> {
    type Prepared = UiNativePreparedPresentation;
    type Acquired = UiNativeSurfaceAcquiredPresentation;
    type Encoded = UiNativeEncodedPresentation;
    type Submitted = UiNativeSubmittedPresentation;
    type PresentHandoff = UiNativePresentHandoff;
    type Observation = UiNativePresentationPortObservation;
    type Failure = UiNativePresentationPortFailure;

    fn prepare(&mut self) -> Result<Self::Prepared, Self::Failure> {
        let cost = self.plan.as_ref().expect("one presentation plan").cost;
        Ok(UiNativePreparedPresentation::new(cost))
    }

    fn acquire(&mut self, prepared: Self::Prepared) -> Result<Self::Acquired, Self::Failure> {
        prepared
            .acquire(self.graphics)
            .map_err(UiNativePresentationPortFailure::Surface)
    }

    fn encode(&mut self, acquired: Self::Acquired) -> Result<Self::Encoded, Self::Failure> {
        let plan = self
            .plan
            .take()
            .expect("prepared presentation owns its plan");
        Ok(acquired.encode_with(|surface_texture| {
            encode_acquired(self.graphics, self.atlas, surface_texture, plan)
        }))
    }

    fn submit(&mut self, encoded: Self::Encoded) -> Result<Self::Submitted, Self::Failure> {
        Ok(encoded.submit(self.graphics.queue()))
    }

    fn hand_off(
        &mut self,
        submitted: Self::Submitted,
    ) -> Result<Self::PresentHandoff, Self::Failure> {
        Ok(submitted.hand_off())
    }

    fn observe(
        &mut self,
        handoff: Self::PresentHandoff,
    ) -> Result<Self::Observation, Self::Failure> {
        observe_handoff(self.graphics, handoff, self.defer_initial_observation)
    }
}

fn encode_acquired(
    graphics: &UiNativePresentationAccess,
    atlas: Option<&crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
    surface_texture: &wgpu::Texture,
    plan: UiNativePresentationPortPlan,
) -> (wgpu::CommandBuffer, wgpu::Buffer) {
    let (surface_pipeline, surface_bind_group) = retained_transfer(graphics);
    let retained_view = graphics
        .retained_target()
        .create_view(&wgpu::TextureViewDescriptor::default());
    let surface_view = surface_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let readback = evidence_buffer(graphics.device());
    let raster_vertices = raster_vertices(&plan.operations);
    let raster_vertex_buffer = vertex_buffer(
        graphics.device(),
        "worth-ui-retained-raster-vertices",
        encode_raster_vertices(&raster_vertices),
    );
    let glyph_vertices = glyph_vertices(&plan.operations, graphics.extent());
    let glyph_vertex_buffer = vertex_buffer(
        graphics.device(),
        "worth-ui-retained-glyph-vertices",
        encode_glyph_vertices(&glyph_vertices),
    );
    let pipelines = presentation_pipelines(graphics.device());
    let commands = encode(
        graphics,
        retained_view,
        surface_view,
        surface_pipeline,
        surface_bind_group,
        &readback,
        raster_vertex_buffer.as_ref(),
        glyph_vertex_buffer.as_ref(),
        atlas,
        &pipelines,
        plan,
    );
    (commands, readback)
}

fn vertex_buffer(
    device: &wgpu::Device,
    label: &'static str,
    bytes: Vec<u8>,
) -> Option<wgpu::Buffer> {
    (!bytes.is_empty()).then(|| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &bytes,
            usage: wgpu::BufferUsages::VERTEX,
        })
    })
}

fn observe_handoff(
    graphics: &UiNativePresentationAccess,
    handoff: UiNativePresentHandoff,
    defer_initial_observation: bool,
) -> Result<UiNativePresentationPortObservation, UiNativePresentationPortFailure> {
    let (readback, submission, cost) = handoff.into_parts();
    let mut pending = UiNativePendingWgpuObligation::new(
        readback,
        submission,
        cost,
        graphics.device_generation(),
    );
    if defer_initial_observation {
        pending.retain_async_handoff();
        return Err(UiNativePresentationPortFailure::ReadbackUnsettled(
            Box::new(pending),
        ));
    }
    match pending.poll_readback() {
        UiNativeWgpuReadbackPoll::Presented(pixels) => Ok(
            UiNativePresentationPortObservation::from_async_readback(pixels, cost),
        ),
        UiNativeWgpuReadbackPoll::Pending | UiNativeWgpuReadbackPoll::Indeterminate => {
            pending.retain_async_handoff();
            Err(UiNativePresentationPortFailure::ReadbackUnsettled(
                Box::new(pending),
            ))
        }
    }
}

fn evidence_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("worth-ui-retained-evidence-readback"),
        size: 512,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

#[allow(clippy::too_many_arguments)]
fn encode(
    graphics: &UiNativePresentationAccess,
    retained_view: wgpu::TextureView,
    surface_view: wgpu::TextureView,
    surface_pipeline: wgpu::RenderPipeline,
    surface_bind_group: wgpu::BindGroup,
    readback: &wgpu::Buffer,
    raster_vertex_buffer: Option<&wgpu::Buffer>,
    glyph_vertex_buffer: Option<&wgpu::Buffer>,
    atlas: Option<&crate::native::text_atlas::UiNativeTextAtlasGpuPages>,
    pipelines: &UiNativePresentationPipelines,
    plan: UiNativePresentationPortPlan,
) -> wgpu::CommandBuffer {
    let mut encoder = graphics
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("worth-ui-initial-presentation"),
        });
    draw_presentation_operations(
        graphics.device(),
        &mut encoder,
        &retained_view,
        raster_vertex_buffer,
        glyph_vertex_buffer,
        &plan.operations,
        atlas,
        pipelines,
        plan.clear_retained_target,
    );
    draw_retained_to_surface(
        &mut encoder,
        &surface_view,
        &surface_pipeline,
        &surface_bind_group,
    );
    copy_evidence_pixels(
        &mut encoder,
        graphics.retained_target(),
        readback,
        graphics.extent(),
    );
    encoder.finish()
}

fn raster_vertices(operations: &[UiNativeRasterOperation]) -> Vec<RasterVertex> {
    operations
        .iter()
        .filter_map(|operation| match operation {
            UiNativeRasterOperation::Clear(rect) => Some(rectangle_vertices(*rect, [0, 0, 0, 0])),
            UiNativeRasterOperation::FilledRect { rect, source_rgba8 } => {
                Some(rectangle_vertices(*rect, *source_rgba8))
            }
            UiNativeRasterOperation::Surface(surface) => {
                Some(rectangle_vertices(surface.rect(), [0, 0, 0, 0]))
            }
            UiNativeRasterOperation::Glyph(_) => None,
        })
        .flatten()
        .collect()
}

fn glyph_vertices(operations: &[UiNativeRasterOperation], extent: [u32; 2]) -> Vec<GlyphVertex> {
    operations
        .iter()
        .filter_map(|operation| match operation {
            UiNativeRasterOperation::Glyph(command) => {
                Some(super::super::text::glyph_vertices(*command, extent))
            }
            UiNativeRasterOperation::Clear(_)
            | UiNativeRasterOperation::FilledRect { .. }
            | UiNativeRasterOperation::Surface(_) => None,
        })
        .flatten()
        .collect()
}

fn encode_raster_vertices(vertices: &[RasterVertex]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vertices.len() * 24);
    for vertex in vertices {
        for value in vertex.position.into_iter().chain(vertex.color) {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
    }
    bytes
}

fn encode_glyph_vertices(vertices: &[GlyphVertex]) -> Vec<u8> {
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
