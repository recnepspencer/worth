use std::marker::PhantomData;

use super::super::historical_authority::{
    WorthQueryApplicationHistoricalBasis, WorthQueryApplicationHistoricalRead,
    WorthQueryApplicationHistoricalReadSource, WorthQueryApplicationHistoricalRetention,
};
use super::validation::{denial, validate_truth_view_request};
use crate::domain_computation::primary_graph::{
    application_query::{
        WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    },
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(crate) fn admit_application_historical_basis(
        &self,
        read: WorthQueryApplicationHistoricalRead,
        request: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryApplicationHistoricalBasis<Schema>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        validate_truth_view_request(request)?;
        let basis = self.admit_historical_execution_basis(read.into_source())?;
        let lease = super::super::admission::register_basis(self, basis)?;
        validate_truth_view_request(request)?;
        Ok(WorthQueryApplicationHistoricalBasis {
            lease,
            _schema: PhantomData,
        })
    }

    fn admit_historical_execution_basis(
        &self,
        source: WorthQueryApplicationHistoricalReadSource,
    ) -> Result<
        worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        match source {
            WorthQueryApplicationHistoricalReadSource::ApplicationCommit {
                provider_runtime_instance_id,
                commit,
                descriptor,
                retention,
            } => {
                if provider_runtime_instance_id != descriptor.runtime_instance_id()
                    || provider_runtime_instance_id
                        != self.relational_branch_identity.runtime_instance_id()
                    || commit.branch_id != *descriptor.branch_id()
                {
                    return Err(denial(
                        WorthQueryApplicationQueryAdmissionDenialKind::ForeignHistoricalReceipt,
                        "application commit receipt",
                    ));
                }
                let retention = match retention {
                    WorthQueryApplicationHistoricalRetention::OwnerLifecycle => self
                        .primary_provider
                        .retained_application_commit_basis(&commit),
                }
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable,
                        "historical receipt basis retention has expired",
                    )
                })?;
                if retention.descriptor() != &descriptor {
                    return Err(denial(
                        WorthQueryApplicationQueryAdmissionDenialKind::ForeignHistoricalReceipt,
                        "application commit receipt",
                    ));
                }
                let basis = self
                    .primary_provider
                    .graph
                    .with_runtime(|runtime| {
                        runtime.readmit_retained_branch_basis(&descriptor, retention.lease())
                    })
                    .map_err(super::super::map_basis_denial)?;
                let selected_commit = self.primary_provider.graph.with_runtime(|runtime| {
                    runtime
                        .history()
                        .branch_head_for_observation(&basis.observation())
                        .ok()
                        .flatten()
                });
                if selected_commit.as_ref() != Some(&commit) {
                    return Err(denial(
                        WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable,
                        "historical receipt does not match its exact Relational basis",
                    ));
                }
                Ok(basis)
            }
        }
    }
}
