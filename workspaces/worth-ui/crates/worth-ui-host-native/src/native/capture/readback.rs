use std::sync::mpsc::{Receiver, TryRecvError};

#[derive(Clone, Copy)]
pub(super) struct UiNativeReadbackLayout {
    dimensions: [u32; 2],
    tight_row_bytes: u32,
    padded_row_bytes: u32,
    allocation_bytes: u64,
}

pub(super) struct UiNativeReadback {
    buffer: wgpu::Buffer,
    submission: wgpu::SubmissionIndex,
    mapping: Receiver<Result<(), wgpu::BufferAsyncError>>,
    layout: UiNativeReadbackLayout,
}

pub(super) enum UiNativeReadbackPoll {
    Pending(UiNativeReadback),
    Captured(Box<[u8]>),
    ArtifactIndeterminate,
    PhysicalCompletionIndeterminate(UiNativeReadback),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiNativeDevicePollPosture {
    SubmissionSettled,
    Pending,
    PhysicalCompletionIndeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiNativeMappingPosture {
    Ready,
    Pending,
    ArtifactIndeterminate,
}

impl UiNativeReadbackLayout {
    pub(super) fn bounded(dimensions: [u32; 2], request_limit: u64) -> Option<Self> {
        let tight_row_bytes = dimensions[0].checked_mul(4)?;
        let tight_bytes = u64::from(tight_row_bytes).checked_mul(u64::from(dimensions[1]))?;
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row_bytes = tight_row_bytes
            .checked_add(alignment - 1)?
            .checked_div(alignment)?
            .checked_mul(alignment)?;
        let allocation_bytes = u64::from(padded_row_bytes).checked_mul(u64::from(dimensions[1]))?;
        (dimensions.iter().all(|value| *value > 0)
            && tight_bytes <= request_limit
            && allocation_bytes
                <= u64::from(crate::UiNativeMechanicsCapacities::QUALIFIED.readback_bytes))
        .then_some(Self {
            dimensions,
            tight_row_bytes,
            padded_row_bytes,
            allocation_bytes,
        })
    }

    pub(super) const fn dimensions(self) -> [u32; 2] {
        self.dimensions
    }

    pub(super) const fn tight_row_bytes(self) -> u32 {
        self.tight_row_bytes
    }

    pub(super) const fn allocation_bytes(self) -> u64 {
        self.allocation_bytes
    }

    pub(super) fn canonical_byte_len(self) -> usize {
        let bytes = u64::from(self.tight_row_bytes)
            .checked_mul(u64::from(self.dimensions[1]))
            .expect("the validated readback layout retains an exact canonical byte count");
        usize::try_from(bytes)
            .expect("the qualified 16 MiB readback bound fits the supported native target")
    }
}

impl UiNativeReadback {
    pub(super) fn begin(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        retained_target: &wgpu::Texture,
        layout: UiNativeReadbackLayout,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("worth-ui-visual-capture-readback"),
            size: layout.allocation_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("worth-ui-visual-capture-copy"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: retained_target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(layout.padded_row_bytes),
                    rows_per_image: Some(layout.dimensions[1]),
                },
            },
            wgpu::Extent3d {
                width: layout.dimensions[0],
                height: layout.dimensions[1],
                depth_or_array_layers: 1,
            },
        );
        let submission = queue.submit([encoder.finish()]);
        let (sender, mapping) = std::sync::mpsc::sync_channel(1);
        buffer.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = sender.send(result);
        });
        Self {
            buffer,
            submission,
            mapping,
            layout,
        }
    }

    pub(super) fn poll(self, device: &wgpu::Device) -> UiNativeReadbackPoll {
        match classify_device_poll(device.poll(wgpu::PollType::Wait {
            submission_index: Some(self.submission.clone()),
            timeout: Some(std::time::Duration::ZERO),
        })) {
            UiNativeDevicePollPosture::SubmissionSettled => self.poll_mapping(),
            UiNativeDevicePollPosture::Pending => UiNativeReadbackPoll::Pending(self),
            UiNativeDevicePollPosture::PhysicalCompletionIndeterminate => {
                UiNativeReadbackPoll::PhysicalCompletionIndeterminate(self)
            }
        }
    }

    pub(super) fn poll_recovery(self, device: &wgpu::Device) -> UiNativeReadbackPoll {
        match classify_device_poll(device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::ZERO),
        })) {
            UiNativeDevicePollPosture::SubmissionSettled => self.poll_mapping(),
            UiNativeDevicePollPosture::Pending
            | UiNativeDevicePollPosture::PhysicalCompletionIndeterminate => {
                UiNativeReadbackPoll::PhysicalCompletionIndeterminate(self)
            }
        }
    }

    fn poll_mapping(self) -> UiNativeReadbackPoll {
        match classify_mapping(self.mapping.try_recv()) {
            UiNativeMappingPosture::Ready => self.canonicalize(),
            UiNativeMappingPosture::ArtifactIndeterminate => {
                UiNativeReadbackPoll::ArtifactIndeterminate
            }
            UiNativeMappingPosture::Pending => UiNativeReadbackPoll::Pending(self),
        }
    }

    fn canonicalize(self) -> UiNativeReadbackPoll {
        let mapped = self.buffer.get_mapped_range(..);
        let canonical = canonical_bytes(&mapped, self.layout);
        drop(mapped);
        self.buffer.unmap();
        match canonical {
            Some(bytes) => UiNativeReadbackPoll::Captured(bytes),
            None => UiNativeReadbackPoll::ArtifactIndeterminate,
        }
    }
}

fn classify_device_poll(
    observation: Result<wgpu::PollStatus, wgpu::PollError>,
) -> UiNativeDevicePollPosture {
    match observation {
        Ok(_) => UiNativeDevicePollPosture::SubmissionSettled,
        Err(wgpu::PollError::Timeout) => UiNativeDevicePollPosture::Pending,
        Err(wgpu::PollError::WrongSubmissionIndex(_, _)) => {
            UiNativeDevicePollPosture::PhysicalCompletionIndeterminate
        }
    }
}

fn classify_mapping(
    observation: Result<Result<(), wgpu::BufferAsyncError>, TryRecvError>,
) -> UiNativeMappingPosture {
    match observation {
        Ok(Ok(())) => UiNativeMappingPosture::Ready,
        Err(TryRecvError::Empty) => UiNativeMappingPosture::Pending,
        Ok(Err(_)) | Err(TryRecvError::Disconnected) => {
            UiNativeMappingPosture::ArtifactIndeterminate
        }
    }
}

fn canonical_bytes(mapped: &[u8], layout: UiNativeReadbackLayout) -> Option<Box<[u8]>> {
    (u64::try_from(mapped.len()).ok()? == layout.allocation_bytes()).then_some(())?;
    let tight_len = usize::try_from(layout.tight_row_bytes).ok()?;
    let padded_len = usize::try_from(layout.padded_row_bytes).ok()?;
    let row_count = usize::try_from(layout.dimensions[1]).ok()?;
    let mut bytes = Vec::with_capacity(tight_len.checked_mul(row_count)?);
    for row in 0..row_count {
        let start = row.checked_mul(padded_len)?;
        bytes.extend_from_slice(mapped.get(start..start.checked_add(tight_len)?)?);
    }
    Some(bytes.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_bytes, classify_device_poll, classify_mapping, UiNativeDevicePollPosture,
        UiNativeMappingPosture, UiNativeReadback, UiNativeReadbackLayout, UiNativeReadbackPoll,
    };

    #[test]
    fn external_readback_failures_have_distinct_resource_postures() {
        assert_eq!(
            classify_device_poll(Err(wgpu::PollError::Timeout)),
            UiNativeDevicePollPosture::Pending
        );
        assert_eq!(
            classify_device_poll(Err(wgpu::PollError::WrongSubmissionIndex(9, 7))),
            UiNativeDevicePollPosture::PhysicalCompletionIndeterminate
        );
        assert_eq!(
            classify_device_poll(Ok(wgpu::PollStatus::WaitSucceeded)),
            UiNativeDevicePollPosture::SubmissionSettled
        );
        assert_eq!(
            classify_mapping(Ok(Err(wgpu::BufferAsyncError))),
            UiNativeMappingPosture::ArtifactIndeterminate
        );
        assert_eq!(
            classify_mapping(Err(std::sync::mpsc::TryRecvError::Disconnected)),
            UiNativeMappingPosture::ArtifactIndeterminate
        );
        assert_eq!(
            classify_mapping(Err(std::sync::mpsc::TryRecvError::Empty)),
            UiNativeMappingPosture::Pending
        );
    }

    #[test]
    fn malformed_mapped_shape_cannot_become_a_canonical_artifact() {
        let layout = UiNativeReadbackLayout::bounded([3, 2], 24).unwrap();
        assert!(canonical_bytes(&[0; 511], layout).is_none());
    }

    #[test]
    fn production_readback_removes_gpu_row_padding_without_changing_rgba_bytes() {
        let (device, queue, _) = crate::native::text_atlas::qualified_test_device();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("worth-ui-capture-canonicalization-source"),
            size: wgpu::Extent3d {
                width: 3,
                height: 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::native::graphics::qualified_target_format(),
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let expected = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ];
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &expected,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(12),
                rows_per_image: Some(2),
            },
            wgpu::Extent3d {
                width: 3,
                height: 2,
                depth_or_array_layers: 1,
            },
        );
        let layout = UiNativeReadbackLayout::bounded([3, 2], 24).unwrap();
        let mut readback = UiNativeReadback::begin(&device, &queue, &texture, layout);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match readback.poll(&device) {
                UiNativeReadbackPoll::Captured(bytes) => {
                    assert_eq!(bytes.as_ref(), expected);
                    break;
                }
                UiNativeReadbackPoll::Pending(pending) => readback = pending,
                UiNativeReadbackPoll::ArtifactIndeterminate
                | UiNativeReadbackPoll::PhysicalCompletionIndeterminate(_) => {
                    panic!("readback became indeterminate")
                }
            }
            assert!(std::time::Instant::now() < deadline, "readback timed out");
            std::thread::yield_now();
        }
    }
}
