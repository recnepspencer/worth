//! Complete required writer coverage and capture only the authored stage spans.
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use super::super::{custody, RelationalAuthoredCoupledOwners, RequiredOutputPublication};
use super::refusals::reject;
use super::retained_assembly::{CoupledAuthoredStagePart, CoupledPublicationPart};
use super::source_effects::RelationalCoupledAuthoredSourceEffects;
use crate::authority::mutation::FoundationalPatchFragment;
use crate::execution::request_stage::checkpoint;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;

pub(super) fn complete(
    execution: &ExecutionRequest<'_, '_>,
    owners: &RelationalAuthoredCoupledOwners,
    parts: &[CoupledPublicationPart],
    stages: &[CoupledAuthoredStagePart],
    fragments: &[FoundationalPatchFragment],
    authored_prefix: std::ops::Range<usize>,
    authored_records: &mut Vec<RecordRef>,
) -> Result<RelationalCoupledAuthoredSourceEffects, TransactionCommitError> {
    if let Some(missing) = owners.stages.get(stages.len()) {
        return Err(reject(Cause::MissingAuthoredStage, *missing));
    }
    // Select actual authored appends before final patch composition. Producer
    // fragments between authored stages never become authored source effects.
    let source_effects = RelationalCoupledAuthoredSourceEffects::capture(
        execution,
        fragments,
        std::iter::once(authored_prefix).chain(stages.iter().map(|stage| stage.fragments.clone())),
    )?;
    for stage in stages {
        authored_records.extend_from_slice(&stage.records);
    }
    execution.run_native_with_checkpoint(|context| {
        for expected in owners.required_owners() {
            for part in parts {
                let meaning = part.publication.producing_meaning().expect("admitted producer route");
                checkpoint(context, (meaning.family.len() + expected.origin.meaning.family.len() + 65) as u64)
                    .map_err(custody::execution_stop)?;
            }
            if !parts.iter().any(|part| matches!(part.publication.owned(),
                Some(RequiredOutputPublication::Preserve { meaning, .. }) if expected.origin.meaning.same_route(meaning))) {
                return Err(reject(Cause::MissingCoupledOwner, expected.origin.meaning.source_scope));
            }
        }
        Ok(())
    }).map_err(custody::execution_stop)?.0?;
    Ok(source_effects)
}
