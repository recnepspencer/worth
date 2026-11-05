//! Authority precedes source affinity, retention and admission mutation.
use super::*;
use crate::domain_computation::authorization::{BrandTag, WorthQueryAdmittedOperation};

mod binding;
mod pending;
mod preparation;

pub(super) struct CheckedSourceAdmission<'a, Schema, Operation, Input, Scope> {
    runtime: &'a dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
    admission: &'a WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
}

pub(super) struct CheckedMutableSourceAdmission<'a, Schema, Operation, Input, Scope> {
    runtime: &'a dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
    admission: &'a mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
}

pub(super) trait PreparedExpectationAdmissionParentOps<
    Schema: worth_query_installation::facade::ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    #[inline]
    fn validate_source_admission<'a, Operation, Input, Scope>(
        &'a self,
        admission: &'a WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        subject: &str,
    ) -> Result<
        CheckedSourceAdmission<'a, Schema, Operation, Input, Scope>,
        WorthQuerySourceExpectationDenial,
    > {
        let (core, tag) = admission.reentry_parts();
        validate_source_admission(self.application_query_host(), core, tag, subject)?;
        Ok(CheckedSourceAdmission {
            runtime: self.application_query_host(),
            admission,
        })
    }

    #[inline]
    fn readmit_mutable_source_admission<'a, Operation, Input, Scope>(
        &'a self,
        admission: &'a mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        subject: &str,
    ) -> Result<
        CheckedMutableSourceAdmission<'a, Schema, Operation, Input, Scope>,
        WorthQuerySourceExpectationDenial,
    > {
        let (core, tag) = admission.reentry_parts();
        validate_source_admission(self.application_query_host(), core, tag, subject)?;
        Ok(CheckedMutableSourceAdmission {
            runtime: self.application_query_host(),
            admission,
        })
    }
}

impl<
        Schema: worth_query_installation::facade::ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > PreparedExpectationAdmissionParentOps<Schema> for HostRuntime
{
}

fn validate_source_admission<Schema: worth_query_installation::facade::ApplicationSchema>(
    host: &(dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema> + '_),
    core: &WorthQueryAdmittedOperation,
    tag: &BrandTag,
    subject: &str,
) -> Result<(), WorthQuerySourceExpectationDenial> {
    core.validate_projection_entry_with_tag(
        tag,
        host.authority_identity(),
        &host.installed_schema().binding_identity(),
        host.installed_operations(),
    )
    .map_err(|denial| WorthQuerySourceExpectationDenial::admission_authorization(subject, denial))
}
