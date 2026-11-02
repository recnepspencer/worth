//! Authority precedes source affinity, retention and admission mutation.
use super::*;
use crate::domain_computation::authorization::{BrandTag, WorthQueryAdmittedOperation};
use crate::domain_computation::primary_graph::application_runtime::WorthQueryApplicationRuntimeCore;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[inline]
    pub(in crate::domain_computation::primary_graph) fn validate_source_admission<
        Operation,
        Input,
        Scope,
    >(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        subject: &str,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        let (core, tag) = admission.reentry_parts();
        self.runtime.validate_source_admission(
            core,
            tag,
            &self.installed_schema.binding_identity(),
            subject,
        )
    }
}

impl WorthQueryApplicationRuntimeCore {
    fn validate_source_admission(
        &self,
        core: &WorthQueryAdmittedOperation,
        tag: &BrandTag,
        binding: &ApplicationSchemaBindingIdentity,
        subject: &str,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        core.validate_installed_entry_with_tag(
            tag,
            self.authority_identity(),
            binding,
            &self.installed_operations,
        )
        .and_then(|()| core.validate_current_authority())
        .map_err(|denial| {
            WorthQuerySourceExpectationDenial::admission_authorization(subject, denial)
        })
    }
}
