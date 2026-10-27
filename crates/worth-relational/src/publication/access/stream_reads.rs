use crate::publication::cdc::data::{
    SubscriberResumeRequest, SubscriberStreamBatch, SubscriberStreamFailure,
};
use crate::publication::patch::data::{PatchStreamBatch, PatchStreamReadError, PatchStreamRequest};

use super::{PublicationPatchStreamAccess, PublicationSubscriberStreamAccess};

impl<'runtime> PublicationPatchStreamAccess<'runtime> {
    /// Read one bounded canonical batch under the caller's borrowed request.
    pub fn read(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: PatchStreamRequest,
    ) -> Result<PatchStreamBatch, PatchStreamReadError> {
        crate::publication::patch::read_patch_stream(execution, self.runtime, request)
    }
}

impl<'runtime> PublicationSubscriberStreamAccess<'runtime> {
    pub fn read(
        &self,
        request: SubscriberResumeRequest,
    ) -> Result<SubscriberStreamBatch, SubscriberStreamFailure> {
        crate::publication::cdc::access::read_subscriber_stream(self.runtime, request)
    }
}
