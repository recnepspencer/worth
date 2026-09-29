use serde::Serialize;
use worth_query_declaration::facade::application_operation::{
    application_value_identity, ApplicationValueIdentityDomain,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationValueEncodeDenial,
};

use super::WorthQueryApplicationIdempotencyBinding;

impl WorthQueryApplicationIdempotencyBinding {
    /// The single constructor for a commit whose effect program the host built
    /// itself, outside any mutation binding.
    ///
    /// Both identities derive from the canonical encoding of `key` and
    /// `intent`, each under its own domain and scoped to `Operation`, the
    /// operation the host's program commits under. The same value as key and as
    /// intent never shares an identity, a changed intent under a reused key is
    /// intent drift, and the same key and intent under two operations never
    /// replay each other. Mutation requests use `for_mutation_identities`
    /// instead, which also names the binding they run.
    pub fn for_host_commit<Schema, Operation, Key, Intent>(
        key: &Key,
        intent: &Intent,
    ) -> Result<Self, ApplicationValueEncodeDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Key: Serialize + ?Sized,
        Intent: Serialize + ?Sized,
    {
        Ok(Self::new(
            application_value_identity(
                ApplicationValueIdentityDomain::HostCommitKey,
                Operation::IDENTIFIER,
                key,
            )?
            .identity(),
            application_value_identity(
                ApplicationValueIdentityDomain::HostCommitIntent,
                Operation::IDENTIFIER,
                intent,
            )?
            .identity(),
        ))
    }
}
