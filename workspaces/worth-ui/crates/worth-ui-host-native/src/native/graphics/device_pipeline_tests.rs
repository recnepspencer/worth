use super::*;

#[test]
fn presentation_pipelines_are_reused_only_within_their_device_generation() {
    let adapter = crate::native::text_atlas::qualified_test_adapter();
    let prepare = |identity| {
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("native pipeline proof requires a device");
        UiNativeDeviceGeneration::new(
            identity,
            UiNativeBackendDeviceGenerationMechanics::new(
                device,
                queue,
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
            ),
        )
    };
    let first = prepare(1);
    assert!(first.presentation_pipelines.get().is_none());
    let cold = std::time::Instant::now();
    let pipelines = first.presentation_pipelines();
    let cold_time = cold.elapsed();
    let warm = std::time::Instant::now();
    for _ in 0..512 {
        assert!(std::ptr::eq(pipelines, first.presentation_pipelines()));
    }
    eprintln!(
        "pipeline compilation: {cold_time:?}; 512 retained accesses: {:?}",
        warm.elapsed()
    );
    let successor = prepare(2);
    assert!(successor.presentation_pipelines.get().is_none());
    assert!(!std::ptr::eq(pipelines, successor.presentation_pipelines()));
    assert!(std::ptr::eq(pipelines, first.presentation_pipelines()));
}
