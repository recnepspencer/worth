use std::sync::OnceLock;

use crate::native::graphics::adapter_selection::{select_eligible_adapter, AdapterCandidate};
use crate::native::graphics::{qualified_backend, qualified_backends};

/// One graphics instance and one eligible adapter for the whole test process.
///
/// Tests must not each create and destroy their own instance. On Linux the
/// Vulkan loader crashed (SIGSEGV inside its debug-utils trampoline) when one
/// test named a pipeline while others were concurrently creating and
/// destroying instances; the same suite is green single-threaded. Sharing the
/// instance removes concurrent instance lifecycle from the suite entirely and
/// is also how the product runs: one instance, one adapter, many devices.
struct QualifiedTestGraphics {
    _instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    info: wgpu::AdapterInfo,
}

static QUALIFIED_TEST_GRAPHICS: OnceLock<QualifiedTestGraphics> = OnceLock::new();

fn qualified_test_graphics() -> &'static QualifiedTestGraphics {
    QUALIFIED_TEST_GRAPHICS.get_or_init(|| {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = qualified_backends();
        let instance = wgpu::Instance::new(descriptor);
        let observed = pollster::block_on(instance.enumerate_adapters(qualified_backends()))
            .into_iter()
            .map(|adapter| {
                let info = adapter.get_info();
                (
                    AdapterCandidate {
                        surface_supported: true,
                        device_type: info.device_type,
                        limits: adapter.limits(),
                        vendor: info.vendor,
                        device: info.device,
                        name: info.name,
                        driver_info: info.driver_info,
                    },
                    adapter,
                )
            })
            .collect();
        let cpu_adapter = crate::native_profile::WORTH_UI_NATIVE_SURFACE_PROFILE.cpu_adapter;
        let (_, adapter) = select_eligible_adapter(observed, cpu_adapter)
            .expect("qualified test requires a production-eligible adapter");
        let info = adapter.get_info();
        assert_eq!(info.backend, qualified_backend());
        assert_ne!(info.device_type, wgpu::DeviceType::Other);
        if info.device_type == wgpu::DeviceType::Cpu {
            assert_eq!(
                cpu_adapter,
                crate::native_profile::UiNativeCpuAdapterAdmission::Allow,
                "a software rasterizer serves tests only under a profile that admits it"
            );
        }
        QualifiedTestGraphics {
            _instance: instance,
            adapter,
            info,
        }
    })
}

/// The production-eligible adapter of the compiled profile's backend.
pub(crate) fn qualified_test_adapter() -> &'static wgpu::Adapter {
    &qualified_test_graphics().adapter
}

/// A fresh device on the shared qualified adapter, with the production limits.
pub(crate) fn qualified_test_device() -> (wgpu::Device, wgpu::Queue, wgpu::AdapterInfo) {
    let graphics = qualified_test_graphics();
    let required_limits =
        wgpu::Limits::downlevel_defaults().using_resolution(graphics.adapter.limits());
    let (device, queue) =
        pollster::block_on(graphics.adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("worth-ui-qualified-test-device"),
            required_features: wgpu::Features::empty(),
            required_limits,
            ..Default::default()
        }))
        .expect("qualified test requires a device");
    (device, queue, graphics.info.clone())
}
