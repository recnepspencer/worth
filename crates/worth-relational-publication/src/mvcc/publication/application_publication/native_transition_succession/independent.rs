//! Independent succession comes from the current route's actual canonical coordinate.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::group_selection::CanonicalCommitEnvelopeBodyOwnerPort as _;
use super::*;
use contracts::execution::request_stage::checkpoint;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;

pub fn readmit_current_independent_output_successor(
    inputs: RetirementHistoryInputs<'_>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    running_producer: &str,
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
    let foreign = || {
        TransactionCommitError::publication(
            contracts::publication::data::publication_error::PublicationError::authored_retirement(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::ForeignCoupledOwner,
                contracts::transactions::data::primitives::RecordRef::Entity(meaning.source_scope),
            ),
        )
    };
    if installed
        .source_scope_ownership
        .scope_kind(running_producer)
        .is_some()
    {
        return Err(foreign());
    }
    let Some((envelope, origin)) = selection::select(inputs, execution, installed, basis, meaning)?
    else {
        return Ok(None);
    };
    let ordinal = execution
        .run_native_with_checkpoint(|context| {
            checkpoint(
                context,
                (origin.producer.len() + running_producer.len() + 1) as u64,
            )
            .map_err(custody::execution_stop)?;
            if origin.source_scope_owned
                || installed
                    .source_scope_ownership
                    .scope_kind(&origin.producer)
                    .is_some()
                || origin
                    .issuer
                    .as_ref()
                    .is_some_and(|issuer| !issuer.same_publication_owner(&installed.authority))
            {
                return Err(foreign());
            }
            for (ordinal, selected) in envelope.output_origins().enumerate() {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                if Arc::ptr_eq(selected, &origin) {
                    return Ok(ordinal);
                }
            }
            Err(origin_denial(
                "selected independent publication lost its canonical coordinate",
            ))
        })
        .map_err(custody::execution_stop)?
        .0?;
    Ok(Some(RelationalRequiredOutputPublication {
        runtime_instance_id: inputs.projection.runtime_identity,
        authority: installed
            .authority
            .seal_required_output_publication_at(origin.commit.commit_id.0, ordinal),
        evidence: worth_relational_branch::internal_port::publication::ownership::evidence::PublicationEvidence::Canonical(envelope),
        origin,
        readmitted_at: Some(basis.clone()),
    }))
}
