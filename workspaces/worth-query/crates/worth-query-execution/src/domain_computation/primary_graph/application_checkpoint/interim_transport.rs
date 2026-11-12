//! Captured transport authority is separate from imported checkpoint DATA.
use super::super::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use super::*;
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    CapturedNativeInterimAdmission, CapturedNativeInterimOrigin,
};
use worth_runtime_world::facade::{CapturedWorldInterimAdmission, CapturedWorldInterimOrigin};

#[derive(Debug, Clone)]
pub struct WorthQueryInterimCheckpointOrigin {
    source_application: worth_foundational::CanonicalDigestId,
    pub(in crate::domain_computation::primary_graph) native: CapturedNativeInterimOrigin,
    pub(in crate::domain_computation::primary_graph) world: CapturedWorldInterimOrigin,
}
impl WorthQueryApplicationCheckpoint {
    /// Retain the actual issuing owners' immutable transport proof separately
    /// from bytes. Imported byte DATA never reconstructs this proof.
    pub fn capture_origin(&self) -> Option<WorthQueryInterimCheckpointOrigin> {
        self.origin.clone()
    }

    pub fn with_capture_origin(
        mut self,
        origin: WorthQueryInterimCheckpointOrigin,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        self.origin = Some(origin);
        self.preflight_interim(request)
            .map_err(CheckpointDecodeDenial::into_installation_denial)?;
        Ok(self)
    }
    pub(in crate::domain_computation::primary_graph) fn preflight_interim(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), CheckpointDecodeDenial> {
        let mut answer = None;
        let stage = request.run_native_with_checkpoint(|_| {
            answer = Some((|| {
                let work = u64::try_from(self.bytes().len())
                    .map_err(|_| CheckpointDecodeDenial::capacity())?;
                request
                    .run_native_with_checkpoint(|context| {
                        if context.has_accounting() {
                            context.checkpoint_cost(work, work)
                        } else {
                            context.poll()
                        }
                    })
                    .map_err(CheckpointDecodeDenial::execution)?
                    .0
                    .map_err(|cause| {
                        CheckpointDecodeDenial::execution(
                            worth_execution::WorkCeilingDenial::Stopped(cause),
                        )
                    })?;
                let sections = header::sections(self.bytes())?;
                let origin = self.origin.as_ref().ok_or_else(missing)?;
                origin
                    .native
                    .validate_transport(self.bytes(), request)
                    .map_err(native_refusal)?;
                origin
                    .world
                    .validate_transport(self.bytes(), request)
                    .map_err(world_refusal)?;
                if sections.native_image.is_empty() || sections.world_originals.is_empty() {
                    return Err(missing());
                }
                Ok(())
            })())
        });
        match (stage, answer) {
            (Ok(_), Some(answer)) => answer,
            (Ok(_), None) => unreachable!("the interim preflight produced its answer"),
            (Err(cause), Some(Err(inner))) if worth_relational::facade::transactions::RelationalExecutionDenialCause::from(cause)
                == worth_relational::facade::transactions::RelationalExecutionDenialCause::NestedStopped => Err(inner),
            (Err(cause), _) => Err(CheckpointDecodeDenial::execution(cause)),
        }
    }
    pub(super) fn encode_captured(
        native: worth_relational::facade::durability::RelationalNativeCheckpoint,
        mut admission: CapturedNativeInterimAdmission,
        world: CapturedWorldInterimAdmission,
        publication: &super::super::WorthQueryPrimaryGraphPublication,
        accepted_outputs: &[super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity],
        request: &ExecutionRequest<'_, '_>,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<
        (Self, WorthQueryApplicationCheckpointSectionBytes),
        WorthQueryCheckpointCaptureDenial,
    > {
        let image = admission
            .emit_interim_image(world.registry_revision(), world.body_digest(), request)
            .map_err(|cause| WorthQueryCheckpointCaptureDenial::AdmissionImage(Box::new(cause)))?;
        let native_len = native.bytes().len();
        let image_start = HEADER_BYTES
            .checked_add(native_len)
            .ok_or(WorthQueryCheckpointCaptureDenial::OriginalAuthorityUnavailable)?;
        let world_start = image_start
            .checked_add(image.as_bytes().len())
            .ok_or(WorthQueryCheckpointCaptureDenial::OriginalAuthorityUnavailable)?;
        let world_end = world_start
            .checked_add(world.bytes().len())
            .ok_or(WorthQueryCheckpointCaptureDenial::OriginalAuthorityUnavailable)?;
        let (mut checkpoint, sizes) = Self::encode_frame(
            native,
            image.as_bytes(),
            world.bytes(),
            publication,
            accepted_outputs,
            policy,
        )?;
        let native = admission
            .seal_transport(
                checkpoint.bytes.clone(),
                HEADER_BYTES..image_start,
                image_start..world_start,
                request,
            )
            .map_err(|cause| WorthQueryCheckpointCaptureDenial::AdmissionImage(Box::new(cause)))?;
        let world = world
            .seal_transport(
                checkpoint.bytes.clone(),
                world_start..world_end,
                &native,
                request,
            )
            .map_err(|cause| WorthQueryCheckpointCaptureDenial::WorldOriginals(Box::new(cause)))?;
        checkpoint.origin = Some(WorthQueryInterimCheckpointOrigin {
            source_application: application_binding(publication),
            native,
            world,
        });
        Ok((checkpoint, sizes))
    }
}

pub(in crate::domain_computation::primary_graph) fn application_binding(
    publication: &super::super::WorthQueryPrimaryGraphPublication,
) -> worth_foundational::CanonicalDigestId {
    let mut hash = Sha256::new();
    hash.update(b"worth.query.interim.application/1");
    hash.update(publication.binding_identity().package_identity().bytes());
    hash.update(publication.binding_identity().schema_identity().bytes());
    hash.update(publication.bootstrap_commit_id().0.to_be_bytes());
    worth_foundational::CanonicalDigestId::new(hash.finalize().into())
}
fn missing() -> CheckpointDecodeDenial {
    CheckpointDecodeDenial::installation(Denial::new(
        Kind::InterimCheckpointCaptureOriginMissing,
        "interim checkpoint restoration requires its actual capture origin",
    ))
}
pub(in crate::domain_computation::primary_graph) fn native_refusal(
    cause: worth_relational::facade::durability::RelationalCheckpointAdmissionRefusal,
) -> CheckpointDecodeDenial {
    use worth_relational::facade::durability::RelationalCheckpointAdmissionRefusal as R;
    match cause {
        R::Request(cause) => CheckpointDecodeDenial::native_execution(
            cause,
            "interim capture admission stopped".into(),
        ),
        R::Allocation(cause) => CheckpointDecodeDenial::installation(
            super::super::bootstrap::map_checkpoint_recovery_stop(
                worth_relational::facade::durability::RelationalRecoveryStop::Allocation(cause),
            ),
        ),
        R::Buffer(cause) => backing(cause),
        R::Native(cause) => CheckpointDecodeDenial::installation(
            super::super::bootstrap::map_checkpoint_recovery_stop(
                worth_relational::facade::durability::RelationalRecoveryStop::Durability(cause),
            ),
        ),
        R::Content(cause) => CheckpointDecodeDenial::native_execution(
            cause.into_execution_cause(),
            "interim capture content refused".into(),
        ),
        R::MissingCaptureOrigin | R::MissingArtifact => missing(),
        R::PredecessorFormat { .. }
        | R::UnsupportedVersion(_)
        | R::InvalidVersion
        | R::Malformed
        | R::BindingMismatch(_)
        | R::AuthenticationMismatch(_)
        | R::MissingRootImage
        | R::InvalidProjectedMeaning
        | R::OwnerUnavailable => CheckpointDecodeDenial::installation(Denial::new(
            Kind::InterimCheckpointCaptureOriginMismatch,
            format!("interim Native capture refused: {cause:?}"),
        )),
        R::CapacityOverflow => CheckpointDecodeDenial::capacity(),
    }
}
pub(in crate::domain_computation::primary_graph) fn world_refusal(
    cause: worth_runtime_world::facade::WorldInterimCheckpointRefusal,
) -> CheckpointDecodeDenial {
    use worth_runtime_world::facade::WorldInterimCheckpointRefusal as R;
    match cause {
        R::Request(cause) => CheckpointDecodeDenial::execution(cause),
        R::Native(cause) => native_refusal(cause),
        R::Backing(cause) => backing(cause),
        R::Component(cause) => component_refusal(cause),
        R::MissingCaptureOrigin => missing(),
        R::OwnerUnavailable
        | R::CaptureChanged
        | R::BindingMismatch
        | R::AuthenticationMismatch
        | R::ReceiverAlreadyBound
        | R::Malformed => CheckpointDecodeDenial::installation(Denial::new(
            Kind::InterimCheckpointCaptureOriginMismatch,
            format!("interim World capture refused: {cause:?}"),
        )),
        R::AllocationFailed => CheckpointDecodeDenial::allocator(),
        R::SizeOverflow => CheckpointDecodeDenial::capacity(),
    }
}
fn component_refusal(
    cause: worth_relational::facade::branch::RelationalRecordedComponentRefusal,
) -> CheckpointDecodeDenial {
    use worth_relational::facade::branch::RelationalRecordedComponentRefusal as R;
    match cause {
        R::Request(cause) => CheckpointDecodeDenial::native_execution(
            cause,
            "interim Native component admission stopped".into(),
        ),
        R::Source(
            worth_relational::facade::durability::RelationalRecoveryStop::AdmissionImage(cause),
        ) => native_refusal(cause),
        R::Source(worth_relational::facade::durability::RelationalRecoveryStop::Content(cause)) => {
            CheckpointDecodeDenial::native_execution(
                cause.into_execution_cause(),
                "interim component content admission refused".into(),
            )
        }
        R::Source(cause) => CheckpointDecodeDenial::installation(
            super::super::bootstrap::map_checkpoint_recovery_stop(cause),
        ),
        R::PayloadBacking(cause) => backing(cause),
        R::Basis(_)
        | R::Frame(_)
        | R::MissingPerformedCommit { .. }
        | R::CanonicalMismatch { .. }
        | R::PayloadEncoding { .. } => CheckpointDecodeDenial::installation(Denial::new(
            Kind::InterimCheckpointReceiverMismatch,
            format!("interim Native component refused: {cause:?}"),
        )),
    }
}
fn backing(cause: worth_execution::ExecutionRequestArrayDenial) -> CheckpointDecodeDenial {
    use worth_execution::ExecutionRequestArrayDenial as R;
    match cause {
        R::Request(cause) => CheckpointDecodeDenial::execution(cause),
        R::Allocation(cause) => match cause.classify() {
            worth_execution::ExecutionAllocationDenialClass::Request(cause) => {
                CheckpointDecodeDenial::execution(cause)
            }
            worth_execution::ExecutionAllocationDenialClass::Physical(_) => {
                CheckpointDecodeDenial::allocator()
            }
        },
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static>
    super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub(in crate::domain_computation::primary_graph) fn restore_interim_registry(
        &self,
        origin: &WorthQueryInterimCheckpointOrigin,
        continuation: Option<
            &worth_relational::facade::durability::NativeInterimBootstrapContinuation,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let receiver = self
            .runtime
            .primary_provider
            .graph
            .with_runtime(request, |runtime| {
                runtime
                    .durability_authority()
                    .admit_interim_receiver(&origin.native, request)
            })
            .map_err(|cause| native_refusal(cause).into_installation_denial())?;
        let product = &self.runtime.product_runtime;
        let lease = product
            .admit_product_occurrence(product.default_occurrence)
            .map_err(|cause| {
                Denial::new(
                    Kind::InterimCheckpointReceiverMismatch,
                    format!("interim World receiver refused: {cause:?}"),
                )
            })?;
        let port = product.interim_checkpoint_port().ok_or_else(|| {
            Denial::new(
                Kind::InterimCheckpointReceiverMismatch,
                "interim checkpoint cannot restore a directory Home",
            )
        })?;
        port.restore(
            request,
            lease.observation(),
            &receiver,
            &origin.native,
            &origin.world,
            continuation,
            // The continuation joins the authenticated source to the actual
            // acknowledged successor. Ordinary restore still requires the
            // current publication's exact application identity.
            if continuation.is_some() {
                origin.source_application
            } else {
                application_binding(self.publication())
            },
        )
        .map_err(|cause| world_refusal(cause).into_installation_denial())
    }
}
