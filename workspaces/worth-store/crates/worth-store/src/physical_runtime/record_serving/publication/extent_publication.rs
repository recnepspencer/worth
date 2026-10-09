use worth_store_physical_format::{DurableExtentManifest, RecordArtifactFile};

use super::super::RecordWriteSource;

pub(in crate::physical_runtime::record_serving) struct ExtentDataPlan {
    pub(in crate::physical_runtime::record_serving) range:
        worth_store_physical_format::ExtentArenaRange,
    pub(in crate::physical_runtime::record_serving) artifact: RecordArtifactFile,
    pub(in crate::physical_runtime::record_serving) manifest: DurableExtentManifest,
    pub(in crate::physical_runtime::record_serving) source: Box<dyn RecordWriteSource>,
}
