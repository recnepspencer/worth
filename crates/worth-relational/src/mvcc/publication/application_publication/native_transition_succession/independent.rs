//! Independent succession comes from the current route's actual canonical coordinate.
use super::*;
use crate::execution::request_stage::checkpoint;

impl RelationalRuntime {
    /// Reissue an independent route's original publication custody at current C.
    /// Payload currentness belongs to the caller's verification; native successor
    /// installation still checks the full predecessor and relation correspondence.
    pub fn readmit_current_independent_output_successor(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        running_producer: &str,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
        let foreign = || {
            TransactionCommitError::publication(
                crate::publication::data::PublicationError::authored_retirement(
                    crate::publication::data::AuthoredRetirementRefusal::ForeignCoupledOwner,
                    crate::transactions::data::RecordRef::Entity(meaning.source_scope),
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
        let Some((envelope, origin)) =
            selection::select(self, execution, installed, basis, meaning)?
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
            runtime_instance_id: self.runtime_instance_id(),
            authority: installed
                .authority
                .seal_required_output_publication_at(origin.commit.commit_id.0, ordinal),
            evidence: crate::publication::ownership::PublicationEvidence::Canonical(envelope),
            origin,
            readmitted_at: Some(basis.clone()),
        }))
    }
}
