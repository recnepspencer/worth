//! Authority precedes source affinity, retention and admission mutation.
use super::*;
use crate::domain_computation::authorization::{BrandTag, WorthQueryAdmittedOperation};
use crate::domain_computation::primary_graph::application_runtime::WorthQueryApplicationRuntimeCore;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

mod binding;
mod pending;
mod preparation;

pub(super) struct CheckedSourceAdmission<'a, Schema, Operation, Input, Scope> {
    runtime: &'a crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    admission: &'a WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
}

pub(super) struct CheckedMutableSourceAdmission<'a, Schema, Operation, Input, Scope> {
    runtime: &'a crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
    admission: &'a mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
}

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[inline]
    pub(super) fn validate_source_admission<'a, Operation, Input, Scope>(
        &'a self,
        admission: &'a WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        subject: &str,
    ) -> Result<
        CheckedSourceAdmission<'a, Schema, Operation, Input, Scope>,
        WorthQuerySourceExpectationDenial,
    > {
        let (core, tag) = admission.reentry_parts();
        self.runtime.validate_source_admission(
            core,
            tag,
            &self.installed_schema.binding_identity(),
            subject,
        )?;
        Ok(CheckedSourceAdmission {
            runtime: self,
            admission,
        })
    }

    #[inline]
    pub(super) fn readmit_mutable_source_admission<'a, Operation, Input, Scope>(
        &'a self,
        admission: &'a mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        subject: &str,
    ) -> Result<
        CheckedMutableSourceAdmission<'a, Schema, Operation, Input, Scope>,
        WorthQuerySourceExpectationDenial,
    > {
        let (core, tag) = admission.reentry_parts();
        self.runtime.validate_source_admission(
            core,
            tag,
            &self.installed_schema.binding_identity(),
            subject,
        )?;
        Ok(CheckedMutableSourceAdmission {
            runtime: self,
            admission,
        })
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
        core.validate_projection_entry_with_tag(
            tag,
            self.authority_identity(),
            binding,
            &self.installed_operations,
        )
        .map_err(|denial| {
            WorthQuerySourceExpectationDenial::admission_authorization(subject, denial)
        })
    }
}
