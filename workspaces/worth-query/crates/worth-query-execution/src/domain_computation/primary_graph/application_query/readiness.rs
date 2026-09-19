use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
};

use super::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// Descriptive readiness of one installed primary-graph application runtime.
///
/// This snapshot carries no query, mutation, basis, or installation authority.
/// A host may use its token for optimistic transport preconditions, but the
/// owning Query runtime must still perform ordinary admission and currentness
/// checks before executing an application operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphApplicationReadinessSnapshot {
    schema_binding: ApplicationSchemaBindingIdentity,
    basis_identity: WorthQueryApplicationBasisIdentity,
    basis_token: String,
}

impl<Schema>
    crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    /// Inspects the current installed application basis without allowing its
    /// owner-issued lease to escape the Query boundary.
    pub fn inspect_application_readiness(
        &self,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationReadinessSnapshot,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let application = self.application();
        let basis_identity = self.application_basis().identity().clone();
        let schema_binding = application.installed_schema().binding_identity();
        let basis_token = basis_token(
            &application.application_readiness_schema_token,
            basis_identity.runtime_instance_id(),
            basis_identity.descriptor(),
        );
        Ok(WorthQueryPrimaryGraphApplicationReadinessSnapshot {
            schema_binding,
            basis_identity,
            basis_token,
        })
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Renders the transport precondition for the exact basis observed by an
    /// application query result.
    pub fn application_basis_token(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
    ) -> Result<String, WorthQueryApplicationQueryAdmissionDenial> {
        self.require_owned_basis(
            basis.runtime_instance_id(),
            basis.branch_id(),
            "application query basis",
        )?;
        Ok(basis_token(
            &self.application_readiness_schema_token,
            basis.runtime_instance_id(),
            basis.descriptor(),
        ))
    }

    /// Renders the transport precondition established by an exact committed
    /// application receipt without rediscovering a newer global head.
    pub fn application_commit_basis_token(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<String, WorthQueryApplicationQueryAdmissionDenial> {
        let descriptor = receipt.basis_descriptor();
        self.require_owned_basis(
            receipt.provider_runtime_instance_id(),
            descriptor.branch_id(),
            "application commit basis",
        )?;
        Ok(basis_token(
            &self.application_readiness_schema_token,
            receipt.provider_runtime_instance_id(),
            descriptor,
        ))
    }

    fn require_owned_basis(
        &self,
        runtime_instance_id: u64,
        branch_id: &worth_relational::facade::history::BranchId,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        if runtime_instance_id != self.relational_branch_identity.runtime_instance_id()
            || branch_id != self.relational_branch_identity.branch_id()
        {
            return Err(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
                subject,
            ));
        }
        Ok(())
    }
}

impl WorthQueryPrimaryGraphApplicationReadinessSnapshot {
    pub fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }

    pub fn basis_identity(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.basis_identity
    }

    pub fn basis_token(&self) -> &str {
        &self.basis_token
    }
}

fn basis_token(
    schema_token: &str,
    runtime_instance_id: u64,
    descriptor: &worth_relational::facade::branch::RelationalBranchBasisDescriptor,
) -> String {
    format!(
        "basis:query-primary-graph-v2:{}:{}:{}:{}",
        runtime_instance_id,
        descriptor.truth_version().as_u64(),
        descriptor.root_identity(),
        schema_token,
    )
}
