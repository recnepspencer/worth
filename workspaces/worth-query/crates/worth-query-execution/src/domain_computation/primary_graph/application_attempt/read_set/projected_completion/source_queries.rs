//! Consuming projected completion accepts only this reader's issued Source values.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::managed_source_read::ManagedSourceReadKey;
use crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind;
impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationReadAttempt<
        '_,
        '_,
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    pub(super) fn observe_source_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: &crate::domain_computation::primary_graph::application_attempt::SourcePublicationRead,
    ) -> Result<
        (
            crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::InstalledScopePosition,
            WorthQueryApplicationObservedFact,
        ),
        WorthQueryApplicationAttemptDenial,
    > {
        self.lease.handle().with_runtime(execution, |_runtime| {
            if !read.admits(self.admission.admission_identity()) {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                    self.admission.operation(),
                ));
            }
            // This exact observation was issued by the same original reader;
            // keys cannot recreate it or admit a second Query read here.
            let observation = self
                .expected_facts
                .as_ref()
                .and_then(|expected| expected.source_publication(read))
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                        self.admission.operation(),
                    )
                })?;
            Ok((
                crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::InstalledScopePosition::SourceQuery,
                WorthQueryApplicationObservedFact::SourcePublication {
                    observation: std::sync::Arc::clone(observation),
                },
            ))
        })
    }
    pub(super) fn observe_source_authorization(
        &self,
        key: &crate::domain_computation::primary_graph::vocabulary::application_attempt::source_read_authorization::SourceReadAuthorizationKey,
    ) -> Result<
        (
            crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::InstalledScopePosition,
            WorthQueryApplicationObservedFact,
        ),
        WorthQueryApplicationAttemptDenial,
    > {
        let observation = self
            .expected_facts
            .as_ref()
            .and_then(|expected| {
                expected.source_authorization(
                    key,
                    self.admission.admission_identity(),
                    &self.admission.retain_graph_contract(),
                    self.lease.snapshot(),
                )
            })
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                    self.admission.operation(),
                )
            })?;
        Ok((
            crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::InstalledScopePosition::SourceQuery,
            WorthQueryApplicationObservedFact::SourceReadAuthorization {
                observation: std::sync::Arc::clone(observation),
            },
        ))
    }
    pub(super) fn observe_managed_source(
        &self,
        key: &ManagedSourceReadKey,
    ) -> Result<
        (
            crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::InstalledScopePosition,
            WorthQueryApplicationObservedFact,
        ),
        WorthQueryApplicationAttemptDenial,
    > {
        let observation = self
            .expected_facts
            .as_ref()
            .and_then(|expected| {
                expected.managed_source(
                    key,
                    self.admission.admission_identity(),
                    &self.admission.retain_graph_contract(),
                    self.lease.snapshot(),
                )
            })
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                    self.admission.operation(),
                )
            })?;
        Ok((
            crate::domain_computation::primary_graph::vocabulary::application_attempt::scope_bindings::InstalledScopePosition::SourceQuery,
            WorthQueryApplicationObservedFact::ManagedIndexSource {
                observation: std::sync::Arc::clone(observation),
            },
        ))
    }
}
