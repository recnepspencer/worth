//! Permission withholding is distinct from a technical authorization failure.
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial as Denial;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as Kind;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as Refusal;
/// Only evaluated permission outcomes may withhold audit. Technical causes remain errors.
pub(super) fn permission_withheld(cause: &Denial) -> bool {
    !cause.causes().is_empty()
        && cause.causes().iter().all(|kind| {
            matches!(
                kind,
                Kind::Refused(
                    Refusal::CapabilityAuthorizationMissing
                        | Refusal::PermissionDenied
                        | Refusal::CapabilityGrantMissing
                        | Refusal::CapabilityExpired
                        | Refusal::PurposeMismatch
                        | Refusal::ScopeMismatch
                        | Refusal::ExplicitDenyRuleMatched
                        | Refusal::ConflictRuleMatched
                        | Refusal::SeparationOfDutyRuleMatched
                        | Refusal::DistinctActorRuleMatched
                )
            )
        })
}

use super::admission::admit;
use super::admission::WorthQueryWorkflowReadDenial as WorkflowReadAdmissionDenial;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationWorkflowReadBinding;
use worth_query_declaration::facade::application_program::ApplicationWorkflowReadRoute;
use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledWorkflowReadBinding;

/// Independently admit audit before its actor reader, matched to the subject access.
/// `None` means genuine permission withholding only; technical causes remain `Err`.
pub(super) fn admit_audit<Schema, Spec, Subject, Audit>(
    current: &WorthQuerySelectedProductOperation<'_, Schema>,
    target: &super::original_intent::OriginalWorkflowCandidate<'_, '_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    subject_binding: &WorthQueryInstalledWorkflowReadBinding<'_, Schema, Spec, Subject>,
    subject: &WorthQueryAdmittedApplicationCapabilityAccess<
        Schema,
        Subject::Capability,
        Subject::Operation,
        Subject::Input,
    >,
    audit_binding: &WorthQueryInstalledWorkflowReadBinding<'_, Schema, Spec, Audit>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    input: Audit::Input,
    request: &WorthQueryRequestScope,
) -> Result<
    Option<
        WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Audit::Capability,
            Audit::Operation,
            Audit::Input,
        >,
    >,
    WorkflowReadAdmissionDenial,
>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Subject: ApplicationWorkflowReadBinding<Schema>,
    Audit: ApplicationWorkflowReadBinding<Schema>,
{
    if target.subject != subject.resource_entity_id()
        || target.data.product().branch_identity() != current.product().branch_identity()
        || !std::ptr::eq(target.data.application(), current.application())
        || subject_binding.route() != ApplicationWorkflowReadRoute::Subject
        || audit_binding.route() != ApplicationWorkflowReadRoute::Audit
        || subject_binding.workflow().support_identity_bytes()
            != audit_binding.workflow().support_identity_bytes()
    {
        return Err(WorkflowReadAdmissionDenial::PolicyMismatch);
    }
    let audit = match admit(current, phase, audit_binding, principal, input, request) {
        Ok(access) => access,
        Err(WorkflowReadAdmissionDenial::Authorization(cause)) if permission_withheld(&cause) => {
            return Ok(None)
        }
        Err(cause) => return Err(cause),
    };
    if subject.resource_entity_id() != audit.resource_entity_id()
        || subject.runtime_authority() != audit.runtime_authority()
        || subject.graph_work_branch() != audit.graph_work_branch()
        || subject.principal_entity_id() != audit.principal_entity_id()
    {
        return Err(WorkflowReadAdmissionDenial::PolicyMismatch);
    }
    Ok(Some(audit))
}

/// No actor reader is reachable before the matched independent audit gate.
pub(super) fn observe<Schema, Spec, Subject, Audit>(
    current: &WorthQuerySelectedProductOperation<'_, Schema>,
    target: &super::original_intent::OriginalWorkflowCandidate<'_, '_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    subject_binding: &WorthQueryInstalledWorkflowReadBinding<'_, Schema, Spec, Subject>,
    subject: &WorthQueryAdmittedApplicationCapabilityAccess<
        Schema,
        Subject::Capability,
        Subject::Operation,
        Subject::Input,
    >,
    audit_binding: Option<&WorthQueryInstalledWorkflowReadBinding<'_, Schema, Spec, Audit>>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    input: Audit::Input,
    request: &WorthQueryRequestScope,

    history:&crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::ReadOnlyWorkflowHistory,
) -> Result<super::view::WorthQueryWorkflowAudit, WorkflowReadAdmissionDenial>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Subject: ApplicationWorkflowReadBinding<Schema>,
    Audit: ApplicationWorkflowReadBinding<Schema>,
{
    use super::view::WorthQueryWorkflowAudit;
    let Some(binding) = audit_binding else {
        return Ok(WorthQueryWorkflowAudit::Withheld);
    };
    let Some(_audit) = admit_audit(
        current,
        target,
        phase,
        subject_binding,
        subject,
        binding,
        principal,
        input,
        request,
    )?
    else {
        return Ok(WorthQueryWorkflowAudit::Withheld);
    };
    let execution = phase
        .execution_request_for(&current.application().product_runtime)
        .map_err(WorkflowReadAdmissionDenial::Phase)?;
    let graph = current
        .application()
        .runtime
        .primary_graph()
        .ok_or(WorkflowReadAdmissionDenial::BasisUnavailable)?;
    graph.integration_handle().with_runtime(&execution,|runtime| {
        crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::audit::observe(
            &execution,runtime,target.data.application_basis().snapshot_handle(),graph.layout().workflow(),target.instance,history)
    })
}
