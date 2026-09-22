use super::{port, transaction_state::UiNativePendingExternalObligation, GPU_WAIT_DEADLINE};

pub(crate) struct UiNativePendingWgpuObligation {
    readback: wgpu::Buffer,
    submission: wgpu::SubmissionIndex,
    mapping: std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
    cost: worth_ui_host_contract::UiHostPresentationCostReport,
    presented: Option<port::UiNativePresentationPortObservation>,
    terminal_indeterminate: bool,
    device_generation: std::sync::Arc<crate::native::UiNativeDeviceGeneration>,
}

pub(crate) enum UiNativeWgpuReadbackPoll {
    Presented([[u8; 4]; 2]),
    Pending,
    Indeterminate,
}

impl UiNativePendingWgpuObligation {
    pub(crate) fn new(
        readback: wgpu::Buffer,
        submission: wgpu::SubmissionIndex,
        cost: worth_ui_host_contract::UiHostPresentationCostReport,
        device_generation: std::sync::Arc<crate::native::UiNativeDeviceGeneration>,
    ) -> Self {
        let (sender, mapping) = std::sync::mpsc::sync_channel(1);
        readback.map_async(wgpu::MapMode::Read, .., move |result| {
            let _ = sender.send(result);
        });
        Self {
            readback,
            submission,
            mapping,
            cost,
            presented: None,
            terminal_indeterminate: false,
            device_generation,
        }
    }

    pub(crate) fn poll_readback(&mut self) -> UiNativeWgpuReadbackPoll {
        if self.terminal_indeterminate {
            return UiNativeWgpuReadbackPoll::Indeterminate;
        }
        if self
            .device_generation
            .device()
            .poll(wgpu::PollType::Wait {
                submission_index: Some(self.submission.clone()),
                timeout: Some(GPU_WAIT_DEADLINE),
            })
            .is_err()
        {
            return UiNativeWgpuReadbackPoll::Pending;
        }
        match self
            .mapping
            .recv_timeout(std::time::Duration::from_millis(50))
        {
            Ok(Ok(())) => self.mapped_pixels(),
            Ok(Err(_)) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                self.terminal_indeterminate = true;
                UiNativeWgpuReadbackPoll::Indeterminate
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => UiNativeWgpuReadbackPoll::Pending,
        }
    }

    fn mapped_pixels(&mut self) -> UiNativeWgpuReadbackPoll {
        let bytes = self.readback.get_mapped_range(..);
        let pixels = bytes
            .get(..4)
            .and_then(|pixel| pixel.try_into().ok())
            .zip(bytes.get(256..260).and_then(|pixel| pixel.try_into().ok()));
        drop(bytes);
        self.readback.unmap();
        match pixels {
            Some((first, second)) => UiNativeWgpuReadbackPoll::Presented([first, second]),
            None => {
                self.terminal_indeterminate = true;
                UiNativeWgpuReadbackPoll::Indeterminate
            }
        }
    }

    pub(crate) fn retain_async_handoff(&mut self) {
        self.cost = self
            .cost
            .checked_add(
                worth_ui_host_contract::UiHostPresentationCostReport::from_adapter(
                    worth_ui_host_contract::UiHostPresentationCostInput {
                        asynchronous_handoffs: 1,
                        ..Default::default()
                    },
                ),
            )
            .expect("one retained native readback handoff fits the presentation cost domain");
    }
}

impl UiNativePendingExternalObligation for UiNativePendingWgpuObligation {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        let status = match self.poll_readback() {
            UiNativeWgpuReadbackPoll::Presented(pixels) => {
                self.presented = Some(port::UiNativePresentationPortObservation::from_async_readback(
                    pixels, self.cost,
                ));
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed
            }
            UiNativeWgpuReadbackPoll::Pending => {
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Pending
            }
            UiNativeWgpuReadbackPoll::Indeterminate => {
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::EffectsIndeterminate
            }
        };
        basis.observe(status)
    }

    fn take_presented_observation(&mut self) -> Option<port::UiNativePresentationPortObservation> {
        self.presented.take()
    }
}

#[cfg(test)]
pub(crate) fn prove_pending_readback_handoff() {
    let (device, queue, _) = crate::native::text_atlas::qualified_test_device();
    let source = evidence_source(&device);
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("worth-ui-readback-control-target"),
        size: 512,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("worth-ui-readback-control-copy"),
    });
    encoder.copy_buffer_to_buffer(&source, 0, &readback, 0, 512);
    let submission = queue.submit([encoder.finish()]);
    let device_lost = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let device_generation = std::sync::Arc::new(crate::native::UiNativeDeviceGeneration::new(
        1,
        crate::native::graphics::UiNativeBackendDeviceGenerationMechanics::new(
            device,
            queue,
            device_lost,
        ),
    ));
    let mut pending = UiNativePendingWgpuObligation::new(
        readback,
        submission,
        worth_ui_host_contract::UiHostPresentationCostReport::default(),
        device_generation,
    );
    pending.retain_async_handoff();
    let UiNativeWgpuReadbackPoll::Presented(pixels) = pending.poll_readback() else {
        panic!("one retained production map must reach its exact bytes");
    };
    assert_eq!(pixels, [[13, 29, 71, 255], [199, 5, 151, 17]]);
}

#[cfg(test)]
fn evidence_source(device: &wgpu::Device) -> wgpu::Buffer {
    let source = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("worth-ui-readback-control-source"),
        size: 512,
        usage: wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: true,
    });
    let mut mapped = source.get_mapped_range_mut(..);
    mapped.slice(..4).copy_from_slice(&[13, 29, 71, 255]);
    mapped.slice(256..260).copy_from_slice(&[199, 5, 151, 17]);
    drop(mapped);
    source.unmap();
    source
}
