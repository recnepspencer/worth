use super::{
    binding::UiProjectionBindingAuthority, UiCollectionProjectionBinding,
    UiProjectionBindingStopReceipt, UiScalarProjectionBinding,
};

#[must_use]
#[derive(Debug)]
pub enum UiScalarProjectionBindingAdmission {
    Ready(UiScalarProjectionBinding),
    Unavailable(crate::UiProjectionUnavailableReceipt),
    Stopped(UiProjectionBindingStopReceipt),
}

#[must_use]
#[derive(Debug)]
pub enum UiCollectionProjectionBindingAdmission {
    Ready(UiCollectionProjectionBinding),
    Stopped(UiProjectionBindingStopReceipt),
}

pub(crate) fn admit_scalar_registration(
    registration: crate::UiScalarProjectionRegistration,
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> UiScalarProjectionBindingAdmission {
    let (view, requirement) = registration.into_parts();
    let (installed_domain, view_identity) = view.into_parts();
    let attempt_identity = installed_domain.authority_identity().clone();
    let runtime_provenance = installed_domain.runtime_provenance();
    if requirement.native_family() != crate::UiProjectionNativeFamily::Text {
        return UiScalarProjectionBindingAdmission::Stopped(
            UiProjectionBindingStopReceipt::initial(
                super::UiProjectionBindingStopKind::NativeFamilyMismatch,
                attempt_identity,
                "milestone 3.13 admits only direct native text scalar projection",
            ),
        );
    }
    let operation = installed_domain.scalar_text_operation_reference();
    let gateway = match operation.enter_attempt(workspace) {
        Ok(gateway) => gateway,
        Err(denial) => {
            return UiScalarProjectionBindingAdmission::Stopped(attempt_stop(
                attempt_identity,
                denial,
            ));
        }
    };
    match gateway.prepare_consumer(requirement.selected_field()) {
        Ok(prepared) => {
            UiScalarProjectionBindingAdmission::Ready(UiScalarProjectionBinding::admitted(
                requirement,
                view_identity,
                UiProjectionBindingAuthority::query_issued(
                    attempt_identity,
                    runtime_provenance,
                    operation,
                ),
                prepared,
            ))
        }
        Err(
            crate::application_binding::WorthUiScalarTextConsumerPreparationDenial::ConsumerContract(
                worth_query::facade::domain::WorthQueryConsumerProjectionContractDenial::Compatibility(
                    denial,
                ),
            ),
        ) => match denial.evidence_identity() {
            Ok(identity) => UiScalarProjectionBindingAdmission::Unavailable(
                crate::UiProjectionUnavailableReceipt::query_issued(
                    crate::UiProjectionUnavailableKind::Unsupported,
                    identity.clone(),
                ),
            ),
            Err(cause) => UiScalarProjectionBindingAdmission::Stopped(
                UiProjectionBindingStopReceipt::initial(
                    super::UiProjectionBindingStopKind::EvidenceIdentity(cause),
                    attempt_identity,
                    "Query could not derive the consumer compatibility identity",
                ),
            ),
        },
        Err(denial) => UiScalarProjectionBindingAdmission::Stopped(preparation_stop(
            attempt_identity,
            denial,
        )),
    }
}

pub(crate) fn admit_collection_registration(
    registration: crate::UiCollectionProjectionRegistration,
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> UiCollectionProjectionBindingAdmission {
    let (view, requirement) = registration.into_parts();
    let (installed_domain, view_identity) = view.into_parts();
    let attempt_identity = installed_domain.authority_identity().clone();
    let runtime_provenance = installed_domain.runtime_provenance();
    if requirement.native_family() != crate::UiProjectionNativeFamily::Text {
        return UiCollectionProjectionBindingAdmission::Stopped(
            UiProjectionBindingStopReceipt::initial(
                super::UiProjectionBindingStopKind::NativeFamilyMismatch,
                attempt_identity,
                "milestone 3.13 admits only direct native text collection projection",
            ),
        );
    }
    let operation = installed_domain.collection_text_operation_reference();
    let gateway = match operation.enter_attempt(workspace) {
        Ok(gateway) => gateway,
        Err(denial) => {
            return UiCollectionProjectionBindingAdmission::Stopped(attempt_stop(
                attempt_identity,
                denial,
            ));
        }
    };
    match gateway.prepare_consumer(&requirement) {
        Ok(prepared) => {
            UiCollectionProjectionBindingAdmission::Ready(UiCollectionProjectionBinding::admitted(
                requirement,
                view_identity,
                UiProjectionBindingAuthority::query_issued(
                    attempt_identity,
                    runtime_provenance,
                    operation,
                ),
                prepared,
            ))
        }
        Err(denial) => UiCollectionProjectionBindingAdmission::Stopped(
            collection_preparation_stop(attempt_identity, denial),
        ),
    }
}

fn attempt_stop(
    attempt_identity: worth_query::facade::runtime::WorthQueryEvidenceIdentity,
    denial: crate::WorthUiQueryOperationAttemptDenial,
) -> UiProjectionBindingStopReceipt {
    use crate::WorthUiQueryOperationAttemptDenial as Denial;
    let (kind, summary) = match denial {
        Denial::Installation(_) => (
            super::UiProjectionBindingStopKind::MissingInstalledView,
            "the installed Query view is unavailable",
        ),
        Denial::InstalledDomainAuthorityMismatch => (
            super::UiProjectionBindingStopKind::WrongWorld,
            "the projection registration belongs to a different Query world",
        ),
        Denial::OperatingWorld(_) => (
            super::UiProjectionBindingStopKind::RebindRequired,
            "Query denied entry to the current operating world",
        ),
    };
    UiProjectionBindingStopReceipt::initial(kind, attempt_identity, summary)
}

fn preparation_stop(
    attempt_identity: worth_query::facade::runtime::WorthQueryEvidenceIdentity,
    denial: crate::application_binding::WorthUiScalarTextConsumerPreparationDenial,
) -> UiProjectionBindingStopReceipt {
    use crate::application_binding::WorthUiScalarTextConsumerPreparationDenial as Denial;
    let (kind, summary) = match denial {
        Denial::Binding(denial) => match denial.kind() {
            worth_query::facade::domain::WorthQueryOperationBindingDenialKind::DomainAuthority => (
                super::UiProjectionBindingStopKind::WrongWorld,
                denial.detail().to_owned(),
            ),
            worth_query::facade::domain::WorthQueryOperationBindingDenialKind::OperationNotInstalled => (
                super::UiProjectionBindingStopKind::MissingInstalledView,
                denial.detail().to_owned(),
            ),
            _ => (
                super::UiProjectionBindingStopKind::SchemaMismatch,
                denial.detail().to_owned(),
            ),
        },
        Denial::ConsumerContract(_) => (
            super::UiProjectionBindingStopKind::LifecycleMismatch,
            "Query consumer support does not satisfy the declared lifecycle".to_owned(),
        ),
        Denial::NativeRequest(denial) => super::scalar_native_request_stop::scalar_native_request_stop(denial),
    };
    UiProjectionBindingStopReceipt::initial(kind, attempt_identity, summary)
}

fn collection_preparation_stop(
    attempt_identity: worth_query::facade::runtime::WorthQueryEvidenceIdentity,
    denial: crate::application_binding::WorthUiCollectionTextConsumerPreparationDenial,
) -> UiProjectionBindingStopReceipt {
    use crate::application_binding::WorthUiCollectionTextConsumerPreparationDenial as Denial;
    let (kind, summary) = match denial {
        Denial::Binding(denial) => match denial.kind() {
            worth_query::facade::domain::WorthQueryOperationBindingDenialKind::DomainAuthority => (
                super::UiProjectionBindingStopKind::WrongWorld,
                denial.detail().to_owned(),
            ),
            worth_query::facade::domain::WorthQueryOperationBindingDenialKind::OperationNotInstalled => (
                super::UiProjectionBindingStopKind::MissingInstalledView,
                denial.detail().to_owned(),
            ),
            _ => (
                super::UiProjectionBindingStopKind::SchemaMismatch,
                denial.detail().to_owned(),
            ),
        },
        Denial::ConsumerContract(denial) => (
            super::UiProjectionBindingStopKind::LifecycleMismatch,
            format!(
                "Query consumer support does not satisfy the live collection lifecycle: {denial:?}"
            ),
        ),
        Denial::ProjectionShapeMismatch => (
            super::UiProjectionBindingStopKind::ShapeMismatch,
            "the installed Query operation is not a collection".to_owned(),
        ),
        Denial::RowIdentityMismatch => (
            super::UiProjectionBindingStopKind::RowIdentityMismatch,
            "the declared row identity does not match Query's collection contract".to_owned(),
        ),
        Denial::NativeRequest(denial) => match denial {
            crate::application_binding::WorthUiCollectionTextNativeRequestDenial::TextNativeFamilyMismatch => (
                super::UiProjectionBindingStopKind::NativeFamilyMismatch,
                "the selected Query native field is not text".to_owned(),
            ),
            crate::application_binding::WorthUiCollectionTextNativeRequestDenial::ApplicationItemKeyNativeFamilyMismatch => (
                super::UiProjectionBindingStopKind::NativeFamilyMismatch,
                "the application item-key field is not unsigned64".to_owned(),
            ),
            crate::application_binding::WorthUiCollectionTextNativeRequestDenial::ProjectionRequest(denial) => (
                super::UiProjectionBindingStopKind::SchemaMismatch,
                format!(
                    "Query rejected the selected field from contract {}: {:?}",
                    denial.contract_key().as_str(),
                    denial.kind()
                ),
            ),
            crate::application_binding::WorthUiCollectionTextNativeRequestDenial::SelectionMismatch(denial) => (
                super::UiProjectionBindingStopKind::SchemaMismatch,
                format!(
                    "Query rejected the selected field key for this exact contract: {:?}",
                    denial.kind()
                ),
            ),
        },
    };
    UiProjectionBindingStopReceipt::initial(kind, attempt_identity, summary)
}
