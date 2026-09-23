//! The GPU bindings one frame's raster operations share.
//!
//! Every analytic surface reads its own record and every glyph samples its own
//! atlas page, but the records sit in one buffer and the pages are few. A frame
//! therefore creates one storage buffer and one bind group per distinct atlas
//! page, rather than one of each per operation: a scrolled viewport replays
//! thousands of operations over a handful of distinct resources.

use wgpu::util::DeviceExt as _;

use super::super::UiNativeRasterOperation;
use super::UiNativePresentationPipelines;
use crate::native::text_atlas::{UiNativeGpuAtlasKind, UiNativeTextAtlasGpuPages};

/// wgpu binds a storage range only where it starts on this alignment.
const STORAGE_BINDING_ALIGNMENT: usize = 256;

/// One atlas page bound for one glyph pipeline.
type GlyphPageKey = (UiNativeGpuAtlasKind, u32, bool);

/// What one frame's operations bind, addressed by operation order.
///
/// A bind group retains the resources it binds, so neither the packed surface
/// buffer nor an atlas page view needs a second owner here.
pub(super) struct UiNativeOperationBindings {
    surfaces: Vec<Option<wgpu::BindGroup>>,
    glyph_pages: Vec<wgpu::BindGroup>,
    glyph_page_of_operation: Vec<Option<usize>>,
}

impl UiNativeOperationBindings {
    pub(super) fn surface(&self, operation: usize) -> Option<&wgpu::BindGroup> {
        self.surfaces.get(operation).and_then(Option::as_ref)
    }

    pub(super) fn glyph(&self, operation: usize) -> Option<&wgpu::BindGroup> {
        let page = (*self.glyph_page_of_operation.get(operation)?)?;
        self.glyph_pages.get(page)
    }
}

pub(super) fn prepare(
    device: &wgpu::Device,
    operations: &[UiNativeRasterOperation],
    atlas: Option<&UiNativeTextAtlasGpuPages>,
    pipelines: &UiNativePresentationPipelines,
) -> UiNativeOperationBindings {
    let (surface_data, spans) = surface_storage(device, operations);
    let surfaces = surface_bind_groups(device, pipelines, surface_data.as_ref(), &spans);
    let (glyph_pages, glyph_page_of_operation) =
        glyph_bind_groups(device, operations, atlas, pipelines);
    UiNativeOperationBindings {
        surfaces,
        glyph_pages,
        glyph_page_of_operation,
    }
}

/// Pack every surface record into one buffer, each on a bindable boundary.
fn surface_storage(
    device: &wgpu::Device,
    operations: &[UiNativeRasterOperation],
) -> (Option<wgpu::Buffer>, Vec<Option<(u64, wgpu::BufferSize)>>) {
    let mut records = Vec::new();
    let mut spans = Vec::with_capacity(operations.len());
    for operation in operations {
        let UiNativeRasterOperation::Surface(surface) = operation else {
            spans.push(None);
            continue;
        };
        let record = surface.storage_bytes();
        let offset = records.len() as u64;
        let size = wgpu::BufferSize::new(record.len() as u64);
        records.extend_from_slice(&record);
        let bound = records.len().next_multiple_of(STORAGE_BINDING_ALIGNMENT);
        records.resize(bound, 0);
        spans.push(size.map(|size| (offset, size)));
    }
    let buffer = (!records.is_empty()).then(|| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("worth-ui-analytic-surface-data"),
            contents: &records,
            usage: wgpu::BufferUsages::STORAGE,
        })
    });
    (buffer, spans)
}

fn surface_bind_groups(
    device: &wgpu::Device,
    pipelines: &UiNativePresentationPipelines,
    surface_data: Option<&wgpu::Buffer>,
    spans: &[Option<(u64, wgpu::BufferSize)>],
) -> Vec<Option<wgpu::BindGroup>> {
    let Some(buffer) = surface_data else {
        return spans.iter().map(|_| None).collect();
    };
    let layout = pipelines.surface.get_bind_group_layout(0);
    spans
        .iter()
        .map(|span| {
            let (offset, size) = (*span)?;
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("worth-ui-analytic-surface-bind-group"),
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset,
                        size: Some(size),
                    }),
                }],
            }))
        })
        .collect()
}

fn glyph_bind_groups(
    device: &wgpu::Device,
    operations: &[UiNativeRasterOperation],
    atlas: Option<&UiNativeTextAtlasGpuPages>,
    pipelines: &UiNativePresentationPipelines,
) -> (Vec<wgpu::BindGroup>, Vec<Option<usize>>) {
    let layouts = [
        pipelines.alpha.get_bind_group_layout(0),
        pipelines.color.get_bind_group_layout(0),
    ];
    let mut keys: Vec<GlyphPageKey> = Vec::new();
    let mut pages = Vec::new();
    let mut page_of_operation = Vec::with_capacity(operations.len());
    for operation in operations {
        let UiNativeRasterOperation::Glyph(command) = operation else {
            page_of_operation.push(None);
            continue;
        };
        let intrinsic = super::super::text::source_is_intrinsic_color(*command);
        let key = (command.atlas_kind, command.atlas_page, intrinsic);
        let page = match keys.iter().position(|existing| *existing == key) {
            Some(page) => page,
            None => {
                let atlas = atlas.expect("admitted glyph command retains native atlas pages");
                let (view, _) = atlas
                    .page_view(command.atlas_kind, command.atlas_page)
                    .expect("admitted glyph command retains its exact atlas page");
                pages.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("worth-ui-glyph-atlas-bind-group"),
                    layout: &layouts[usize::from(intrinsic)],
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
                }));
                keys.push(key);
                keys.len() - 1
            }
        };
        page_of_operation.push(Some(page));
    }
    (pages, page_of_operation)
}
