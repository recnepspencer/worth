//! Consume complete captured metadata into its sealed ordinary manifest.
use super::*;

impl CapturedManifest {
    pub(crate) fn seal_ordinary(self) -> CapturedManifest<RelationalCheckpointManifest> {
        CapturedManifest {
            manifest: self.manifest.seal_ordinary(),
            component_holds: self.component_holds,
            members: self.members,
            canonical_publication: self.canonical_publication,
            canonical_payloads: self.canonical_payloads,
            envelope_cells: self.envelope_cells,
            publications: self.publications,
            cold_source: self.cold_source,
            guarded_members: self.guarded_members,
            history_members: self.history_members,
        }
    }
}
