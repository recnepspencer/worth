//! Crate-boundary internal ports for `WorthQueryApplicationCheckpoint` and
//! `WorthQueryInterimCheckpointOrigin`. Never exported on
//! `worth-query-execution::facade` or Host.

use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::CanonicalDigestId;
use worth_relational::facade::durability::{
    CapturedNativeInterimAdmission, CapturedNativeInterimOrigin,
    NativeInterimBootstrapContinuation, RecoveredRelationalRuntimeAuthority,
    RelationalNativeCheckpoint,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_world::facade::{CapturedWorldInterimAdmission, CapturedWorldInterimOrigin};

use super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity;
use super::super::WorthQueryPrimaryGraphPublication;
use super::{
    CheckpointDecodeDenial, DecodedApplicationCheckpoint, WorthQueryApplicationCheckpoint,
    WorthQueryApplicationCheckpointSectionBytes, WorthQueryCheckpointCaptureDenial,
    WorthQueryCheckpointCapturePolicy, WorthQueryInterimCheckpointOrigin,
};

pub trait InterimCheckpointOriginInternalPort {
    fn source_application(&self) -> &CanonicalDigestId;
    fn native(&self) -> &CapturedNativeInterimOrigin;
    fn world(&self) -> &CapturedWorldInterimOrigin;
}

impl InterimCheckpointOriginInternalPort for WorthQueryInterimCheckpointOrigin {
    fn source_application(&self) -> &CanonicalDigestId {
        &self.source_application
    }

    fn native(&self) -> &CapturedNativeInterimOrigin {
        &self.native
    }

    fn world(&self) -> &CapturedWorldInterimOrigin {
        &self.world
    }
}

pub trait ApplicationCheckpointInternalPort: Sized {
    fn from_admitted_parts(
        bytes: ExecutionImmutableBytes,
        origin: Option<WorthQueryInterimCheckpointOrigin>,
    ) -> Self;
    fn bytes_immutable(&self) -> &ExecutionImmutableBytes;
    fn origin_ref(&self) -> Option<&WorthQueryInterimCheckpointOrigin>;
    fn decode_with_execution(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial>;
    fn decode(self) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial>;
    fn preflight_interim(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), CheckpointDecodeDenial>;
    fn encode_captured(
        native: RelationalNativeCheckpoint,
        admission: CapturedNativeInterimAdmission,
        world: CapturedWorldInterimAdmission,
        publication: &WorthQueryPrimaryGraphPublication,
        accepted_outputs: &[WorthQueryAcceptedOutputCheckpointIdentity],
        request: &ExecutionRequest<'_, '_>,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<
        (Self, WorthQueryApplicationCheckpointSectionBytes),
        WorthQueryCheckpointCaptureDenial,
    >;
    #[allow(clippy::too_many_arguments)]
    fn capture_native_transition(
        runtime: &mut RelationalRuntime,
        recovered: &mut RecoveredRelationalRuntimeAuthority,
        original: &WorthQueryInterimCheckpointOrigin,
        publication: &WorthQueryPrimaryGraphPublication,
        outputs: &[WorthQueryAcceptedOutputCheckpointIdentity],
        request: &ExecutionRequest<'_, '_>,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<(Self, NativeInterimBootstrapContinuation), WorthQueryCheckpointCaptureDenial>;
}

impl ApplicationCheckpointInternalPort for WorthQueryApplicationCheckpoint {
    fn from_admitted_parts(
        bytes: ExecutionImmutableBytes,
        origin: Option<WorthQueryInterimCheckpointOrigin>,
    ) -> Self {
        Self::from_admitted_parts(bytes, origin)
    }

    fn bytes_immutable(&self) -> &ExecutionImmutableBytes {
        &self.bytes
    }

    fn origin_ref(&self) -> Option<&WorthQueryInterimCheckpointOrigin> {
        self.origin.as_ref()
    }

    fn decode_with_execution(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial> {
        Self::decode_with_execution(self, execution)
    }

    fn decode(self) -> Result<DecodedApplicationCheckpoint, CheckpointDecodeDenial> {
        Self::decode(self)
    }

    fn preflight_interim(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), CheckpointDecodeDenial> {
        Self::preflight_interim(self, request)
    }

    fn encode_captured(
        native: RelationalNativeCheckpoint,
        admission: CapturedNativeInterimAdmission,
        world: CapturedWorldInterimAdmission,
        publication: &WorthQueryPrimaryGraphPublication,
        accepted_outputs: &[WorthQueryAcceptedOutputCheckpointIdentity],
        request: &ExecutionRequest<'_, '_>,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<
        (Self, WorthQueryApplicationCheckpointSectionBytes),
        WorthQueryCheckpointCaptureDenial,
    > {
        Self::encode_captured(
            native,
            admission,
            world,
            publication,
            accepted_outputs,
            request,
            policy,
        )
    }

    fn capture_native_transition(
        runtime: &mut RelationalRuntime,
        recovered: &mut RecoveredRelationalRuntimeAuthority,
        original: &WorthQueryInterimCheckpointOrigin,
        publication: &WorthQueryPrimaryGraphPublication,
        outputs: &[WorthQueryAcceptedOutputCheckpointIdentity],
        request: &ExecutionRequest<'_, '_>,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<(Self, NativeInterimBootstrapContinuation), WorthQueryCheckpointCaptureDenial> {
        Self::capture_native_transition(
            runtime,
            recovered,
            original,
            publication,
            outputs,
            request,
            policy,
        )
    }
}
