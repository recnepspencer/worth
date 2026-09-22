//! Read-only adapter/output provenance for the exact task monitor.
//! Enumeration does not select a device or authorize a capture fallback.
use winsafe::prelude::*;

pub(super) fn describe(target_monitor: usize) -> Result<String, String> {
    let factory = winsafe::CreateDXGIFactory1().map_err(|error| error.to_string())?;
    let mut enumeration_default = None;
    let mut matches = Vec::new();
    for (adapter_index, adapter) in factory.EnumAdapters1().enumerate() {
        if adapter_index >= 32 {
            return Err("adapter diagnostic bound exceeded".into());
        }
        let adapter = adapter.map_err(|error| error.to_string())?;
        let desc = adapter.GetDesc().map_err(|error| error.to_string())?;
        let identity = format!(
            "adapter[{adapter_index}] {} vendor={:#06x} device={:#06x} luid={}:{}",
            desc.Description(),
            desc.VendorId,
            desc.DeviceId,
            desc.AdapterLuid.high_part(),
            desc.AdapterLuid.low_part(),
        );
        if adapter_index == 0 {
            enumeration_default = Some(identity.clone());
        }
        for (output_index, output) in adapter.EnumOutputs().enumerate() {
            if output_index >= 64 {
                return Err("output diagnostic bound exceeded".into());
            }
            let output = output.map_err(|error| error.to_string())?;
            let desc = output.GetDesc().map_err(|error| error.to_string())?;
            if desc.Monitor.ptr() as usize == target_monitor {
                let rect = desc.DesktopCoordinates;
                matches.push(format!(
                    "{identity} output[{output_index}] {} attached={} desktop=[{},{},{},{}] rotation={:?}",
                    desc.DeviceName(), desc.AttachedToDesktop(),
                    rect.left, rect.top, rect.right, rect.bottom, desc.Rotation,
                ));
            }
        }
    }
    // Adapter zero is DXGI's enumeration default, not an assertion about the
    // D3D device selected under an external per-executable GPU preference.
    Ok(format!(
        "target HMONITOR={target_monitor:#x}; enumeration default={}; exact target outputs={matches:?}; backend=D3D11 hardware/default adapter + DuplicateOutput1 BGRA8",
        enumeration_default.as_deref().unwrap_or("none"),
    ))
}
