use crate::native::text_atlas::{UiNativeTextAtlasTransactionPlan, UiNativeTextAtlasUpload};
use crate::native::UiNativeHostState;

use super::text_atlas_upload::{
    submit_correlated_upload, CorrelatedGpuUploadObservation, CorrelatedUploadInput,
    UiNativeTextAtlasUploadPort,
};

pub(super) struct QualifiedDx12UploadPort {
    device: wgpu::Device,
    queue: wgpu::Queue,
    settle_immediately: bool,
}

impl QualifiedDx12UploadPort {
    pub(super) fn new() -> Self {
        Self::with_settlement(true)
    }

    pub(super) fn defer_settlement(&mut self) {
        self.settle_immediately = false;
    }

    fn with_settlement(settle_immediately: bool) -> Self {
        let (device, queue, info) = crate::native::text_atlas::qualified_test_device();
        assert_eq!(info.backend, crate::native::graphics::qualified_backend());
        Self {
            device,
            queue,
            settle_immediately,
        }
    }

    pub(super) fn complete(&self) {
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(crate::native::presentation::GPU_WAIT_DEADLINE),
            })
            .expect("qualified DX12 evidence must observe physical completion");
    }
}

impl UiNativeTextAtlasUploadPort for QualifiedDx12UploadPort {
    fn upload(
        &mut self,
        state: &mut UiNativeHostState,
        plan: &UiNativeTextAtlasTransactionPlan,
        uploads: &[UiNativeTextAtlasUpload],
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
    ) -> CorrelatedGpuUploadObservation {
        let mut observation = submit_correlated_upload(CorrelatedUploadInput {
            gpu: &mut state.text_atlas_gpu,
            resources: &mut state.resources,
            device: &self.device,
            queue: &self.queue,
            plan,
            uploads,
            basis,
        });
        if self.settle_immediately
            && state
                .text_atlas_gpu
                .as_ref()
                .is_some_and(|gpu| gpu.transaction_pending(plan.transaction_identity()))
        {
            self.complete();
            observation.signal = state
                .text_atlas_gpu
                .as_mut()
                .and_then(|gpu| {
                    gpu.poll_transaction_observation(
                        &mut state.resources,
                        plan.transaction_identity(),
                    )
                })
                .expect("qualified DX12 upload must return its exact correlated completion");
        }
        observation
    }
}
