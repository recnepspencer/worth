//! Frozen consumer windows and layer-specific version admission.

use worth_foundational::facade::{BoundaryProtocolCompatibilityWindow, BoundaryProtocolVersion};
use worth_query_installation::facade::WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION;

use crate::application_program::WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION;
use crate::envelope::WORTH_QUERY_PACKAGE_RELEASE_ENVELOPE_PROTOCOL_VERSION;
use crate::protocol::WORTH_QUERY_PACKAGE_ARCHIVE_PROTOCOL_VERSION;
use crate::record::WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION;

use super::WorthQueryPackageArchiveCompatibilityDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPackageArchiveProtocolLayer {
    ReleaseEnvelope,
    Archive,
    Manifest,
    RecordFrame,
    ApplicationProgramDescription,
}

/// Reader support windows for the independently versioned archive layers.
///
/// Callers may inspect this profile but cannot construct or widen it. Adding a
/// supported reader version requires changing the archive implementation and
/// this archive-owned declaration together.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPackageArchiveCompatibilityProfile {
    release_envelope: BoundaryProtocolCompatibilityWindow,
    archive: BoundaryProtocolCompatibilityWindow,
    manifest: BoundaryProtocolCompatibilityWindow,
    record_frame: BoundaryProtocolCompatibilityWindow,
    application_program_description: BoundaryProtocolCompatibilityWindow,
}

impl WorthQueryPackageArchiveCompatibilityProfile {
    pub const CURRENT: Self = Self {
        release_envelope: exact_window(WORTH_QUERY_PACKAGE_RELEASE_ENVELOPE_PROTOCOL_VERSION),
        archive: exact_window(WORTH_QUERY_PACKAGE_ARCHIVE_PROTOCOL_VERSION),
        manifest: exact_window(WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION.get()),
        record_frame: exact_window(WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION),
        application_program_description: exact_window(
            WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION,
        ),
    };

    pub const fn release_envelope_window(self) -> BoundaryProtocolCompatibilityWindow {
        self.release_envelope
    }

    pub const fn archive_window(self) -> BoundaryProtocolCompatibilityWindow {
        self.archive
    }

    pub const fn manifest_window(self) -> BoundaryProtocolCompatibilityWindow {
        self.manifest
    }

    pub const fn record_frame_window(self) -> BoundaryProtocolCompatibilityWindow {
        self.record_frame
    }

    pub const fn application_program_description_window(
        self,
    ) -> BoundaryProtocolCompatibilityWindow {
        self.application_program_description
    }

    pub(crate) fn admit(
        self,
        layer: WorthQueryPackageArchiveProtocolLayer,
        observed_version: u16,
    ) -> Result<(), WorthQueryPackageArchiveCompatibilityDenial> {
        let window = self.window(layer);
        let Ok(produced) = BoundaryProtocolVersion::try_new(u32::from(observed_version)) else {
            return Err(WorthQueryPackageArchiveCompatibilityDenial::invalid_zero(
                layer, window,
            ));
        };
        window.admit(produced).map(|_| ()).map_err(|unsupported| {
            WorthQueryPackageArchiveCompatibilityDenial::unsupported(
                layer,
                observed_version,
                window,
                unsupported,
            )
        })
    }

    const fn window(
        self,
        layer: WorthQueryPackageArchiveProtocolLayer,
    ) -> BoundaryProtocolCompatibilityWindow {
        match layer {
            WorthQueryPackageArchiveProtocolLayer::ReleaseEnvelope => self.release_envelope,
            WorthQueryPackageArchiveProtocolLayer::Archive => self.archive,
            WorthQueryPackageArchiveProtocolLayer::Manifest => self.manifest,
            WorthQueryPackageArchiveProtocolLayer::RecordFrame => self.record_frame,
            WorthQueryPackageArchiveProtocolLayer::ApplicationProgramDescription => {
                self.application_program_description
            }
        }
    }
}

const fn exact_window(version: u16) -> BoundaryProtocolCompatibilityWindow {
    let version = BoundaryProtocolVersion::new(version as u32);
    BoundaryProtocolCompatibilityWindow::inclusive(version, version)
}
