//! Fresh admission requires the original owner's actual initialization step.
use super::*;
use crate::durability::source_initial_inputs::RelationalInitialInputsRefusal;

pub(super) fn ensure_original_initialization(
    runtime: &crate::runtime::RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
    source: &Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    protection: &RelationalDurableReadProtection,
) -> Result<bool, RelationalFreshSourceAdmissionStop> {
    let original = runtime
        .history
        .capture_original_branch_schema_inputs(request)
        .map_err(|cause| match cause {
            crate::runtime::OriginalBranchSchemaInputsDenial::Request(cause) => {
                Stop::Request(cause.into())
            }
            crate::runtime::OriginalBranchSchemaInputsDenial::SizeOverflow => Stop::Request(
                crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
            ),
            crate::runtime::OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch => {
                refused("initial input prefix differs")
            }
        })?;
    let durable = protection
        .read_initial_inputs_owned(source, request, 1 << 30)
        .map_err(crate::durability::checkpoint_history::capture::owned_stop)?;
    match (&original.accepted_initial_inputs, durable) {
        (Some(original), Some(bytes)) => {
            let expected = original
                .artifact
                .as_ref()
                .ok_or_else(|| refused("accepted original initialization has no frame"))?;
            if bytes.len() != expected.as_bytes().len() {
                return Err(refused("initialization source differs from its producer").into());
            }
            for (actual, original) in bytes
                .bytes()
                .chunks(64 * 1024)
                .zip(expected.as_bytes().chunks(64 * 1024))
            {
                request
                    .consult()
                    .map_err(|cause| Stop::Request(cause.into()))?;
                if actual != original {
                    return Err(refused("initialization source differs from its producer").into());
                }
            }
            Ok(false)
        }
        (None, None) if original.installation.is_none() => {
            let image = crate::durability::source_initial_inputs::capture::capture(
                &runtime.config().history.main_branch,
                &original,
                None,
                request,
            )
            .map_err(inputs_stop)?;
            let medium = runtime
                .durability
                .medium()
                .ok_or_else(|| refused("fresh initialization medium detached"))?;
            let committed = medium
                .publish_initial_inputs(image, request)
                .map_err(RelationalFreshSourceAdmissionStop::Publication)?;
            committed.consume(|publication| {
                runtime
                    .history
                    .canonical_checkpoint_gate()
                    .retain_accepted_initial_inputs(publication.image)
            });
            Ok(true)
        }
        (None, None) | (None, Some(_)) | (Some(_), None) => {
            Err(refused("fresh installation lacks its own durable original inputs").into())
        }
    }
}
fn inputs_stop(cause: RelationalInitialInputsRefusal) -> Stop {
    match cause {
        RelationalInitialInputsRefusal::Request(cause) => Stop::Request(cause),
        RelationalInitialInputsRefusal::Native(cause) => cause,
        cause => refused_detail(format!("initialization artifact rejected: {cause:?}")),
    }
}
fn refused_detail(detail: String) -> Stop {
    DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail).into()
}
