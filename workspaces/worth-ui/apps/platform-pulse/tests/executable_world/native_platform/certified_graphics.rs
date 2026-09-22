//! The graphics record the certified lane expects the product to report.
//!
//! The courtroom used to spell this record by hand (`Dx12`, `PreMultiplied`,
//! a discrete-or-integrated adapter), which certified one vendor's answer and
//! could never be true on a second platform. Every axis here is read from the
//! certified profile manifest instead, so the expectation is the qualified
//! record itself and a lane that certifies under a different profile derives
//! a different, equally exact, expectation.
use super::qualified_record::{certified_manifest, manifest_string, QUALIFIED_PROFILE_MANIFESTS};

/// The manifest's wgpu backend-selection spelling beside the spelling the
/// host renders into the graphics receipt (`format!("{:?}", wgpu::Backend)`).
/// Closed: an unknown selection is a missing row, never a guess.
const BACKEND_SPELLINGS: [(&str, &str); 2] =
    [("Backends::DX12", "Dx12"), ("Backends::VULKAN", "Vulkan")];

/// The `adapter_order` tokens that name a device type, beside the spelling
/// the host renders (`format!("{:?}", wgpu::DeviceType)`). The remaining
/// tokens (`vendor-id`, `device-id`, ...) are tie-breakers, not types.
const DEVICE_TYPE_SPELLINGS: [(&str, &str); 4] = [
    ("discrete", "DiscreteGpu"),
    ("integrated", "IntegratedGpu"),
    ("virtual", "VirtualGpu"),
    ("cpu", "Cpu"),
];

const BACKEND_AXIS: &str = "runtime_backend_selection";
const ADAPTER_ORDER_AXIS: &str = "adapter_order";
const CPU_ADAPTER_AXIS: &str = "cpu_adapter";
const SURFACE_FORMAT_AXIS: &str = "surface_format";
const PRESENT_MODE_AXIS: &str = "present_mode";
const COMPOSITE_ALPHA_AXIS: &str = "composite_alpha";
const TARGET_FORMAT_AXIS: &str = "target_format";

pub(crate) struct CertifiedGraphicsRecord {
    backend: &'static str,
    admitted_device_types: Vec<&'static str>,
    surface_format: String,
    present_mode: String,
    composite_alpha: String,
    target_format: String,
}

impl CertifiedGraphicsRecord {
    pub(crate) fn from_certified_manifest() -> Self {
        Self::from_manifest(certified_manifest())
    }

    fn from_manifest(manifest: &str) -> Self {
        let axis = |key: &str| {
            manifest_string(manifest, key)
                .unwrap_or_else(|| panic!("the qualified record declares {key}"))
        };
        let selection = axis(BACKEND_AXIS);
        let backend = BACKEND_SPELLINGS
            .iter()
            .find(|(spelling, _)| *spelling == selection)
            .map(|(_, rendered)| *rendered)
            .unwrap_or_else(|| panic!("{selection:?} is not a known backend selection"));
        let order = axis(ADAPTER_ORDER_AXIS);
        let admitted_device_types = order
            .split(';')
            .filter_map(|token| {
                DEVICE_TYPE_SPELLINGS
                    .iter()
                    .find(|(spelling, _)| *spelling == token)
                    .map(|(_, rendered)| *rendered)
            })
            .collect();
        Self {
            backend,
            admitted_device_types,
            surface_format: axis(SURFACE_FORMAT_AXIS),
            present_mode: axis(PRESENT_MODE_AXIS),
            composite_alpha: axis(COMPOSITE_ALPHA_AXIS),
            target_format: axis(TARGET_FORMAT_AXIS),
        }
    }

    pub(crate) fn backend(&self) -> &str {
        self.backend
    }

    /// Whether the record admits an adapter of this rendered device type.
    pub(crate) fn admits_device_type(&self, rendered: &str) -> bool {
        self.admitted_device_types.contains(&rendered)
    }

    pub(crate) fn surface_format(&self) -> &str {
        &self.surface_format
    }

    pub(crate) fn present_mode(&self) -> &str {
        &self.present_mode
    }

    pub(crate) fn composite_alpha(&self) -> &str {
        &self.composite_alpha
    }

    pub(crate) fn target_format(&self) -> &str {
        &self.target_format
    }
}

/// Every qualified record derives, so an unknown backend or an empty adapter
/// order fails here for the profile that carries it, not in a lane that
/// happens to certify under it.
#[test]
fn every_qualified_record_derives_a_graphics_expectation() {
    for (identity, manifest) in QUALIFIED_PROFILE_MANIFESTS {
        let record = CertifiedGraphicsRecord::from_manifest(manifest);
        assert!(
            !record.admitted_device_types.is_empty(),
            "{identity} admits no device type"
        );
        assert!(!record.surface_format().is_empty(), "{identity}");
    }
}

/// `cpu_adapter` and the `cpu` row of `adapter_order` are two spellings of
/// one fact; a record that allows a software adapter it never ranks (or the
/// reverse) would admit an adapter the product cannot select.
#[test]
fn cpu_admission_and_adapter_order_agree_in_every_qualified_record() {
    for (identity, manifest) in QUALIFIED_PROFILE_MANIFESTS {
        let record = CertifiedGraphicsRecord::from_manifest(manifest);
        let admission = manifest_string(manifest, CPU_ADAPTER_AXIS)
            .unwrap_or_else(|| panic!("{identity} declares {CPU_ADAPTER_AXIS}"));
        let ranks_cpu = record.admits_device_type("Cpu");
        match admission.as_str() {
            "allow" => assert!(ranks_cpu, "{identity} allows a cpu adapter it never ranks"),
            "deny" => assert!(!ranks_cpu, "{identity} ranks a cpu adapter it denies"),
            other => panic!("{identity}: {other:?} is not a cpu adapter admission"),
        }
    }
}

#[test]
fn the_derivation_reads_each_axis_exactly() {
    let manifest = "runtime_backend_selection = \"Backends::VULKAN\"\n\
                    adapter_order = \"discrete;integrated;virtual;cpu;vendor-id\"\n\
                    surface_format = \"Bgra8UnormSrgb\"\n\
                    present_mode = \"Fifo\"\n\
                    composite_alpha = \"Opaque\"\n\
                    target_format = \"Rgba8UnormSrgb\"\n";
    let record = CertifiedGraphicsRecord::from_manifest(manifest);
    assert_eq!(record.backend(), "Vulkan");
    assert!(record.admits_device_type("Cpu"));
    assert!(record.admits_device_type("IntegratedGpu"));
    assert!(!record.admits_device_type("Other"));
    assert!(!record.admits_device_type("vendor-id"));
    assert_eq!(record.composite_alpha(), "Opaque");
    assert_eq!(record.target_format(), "Rgba8UnormSrgb");
}
