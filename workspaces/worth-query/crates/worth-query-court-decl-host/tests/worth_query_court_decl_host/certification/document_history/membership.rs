use super::*;
use worth_query_host::facade::worth_query_portable_type;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationHistoryDenial, WorthQueryDocumentHistoryInstallationDenialKind,
    },
    declaration::application_history::ApplicationHistoryBinding,
};

struct UndeclaredHistory;
worth_query_portable_type!(UndeclaredHistory => "worth.query.document-retention.undeclared-history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for UndeclaredHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "UndeclaredDocumentHistory";
}

struct CopiedHistory;
worth_query_portable_type!(CopiedHistory => "worth.query.document-retention.history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for CopiedHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "DocumentRetentionHistory";
}

#[test]
fn attachment_requires_the_actual_installed_history_binding_and_returns_head_on_refusal() {
    let host = publish_on_first_program();
    let original = head(&host);
    let branch = original.branch();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let failure = host
        .with_host_advancement(|phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .attach_history::<UndeclaredHistory>(
                    &phase,
                    original,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap_err()
        })
        .unwrap();
    assert!(
        matches!(failure.denial(), WorthQueryApplicationHistoryDenial::Installation(denial)
        if denial.kind() == WorthQueryDocumentHistoryInstallationDenialKind::NotInstalled)
    );
    let (original, _) = failure.into_parts();
    assert_eq!(original.branch(), branch);
    let failure = host
        .with_host_advancement(|phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .attach_history::<CopiedHistory>(
                    &phase,
                    original,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap_err()
        })
        .unwrap();
    assert!(
        matches!(failure.denial(), WorthQueryApplicationHistoryDenial::Installation(denial)
        if denial.kind() == WorthQueryDocumentHistoryInstallationDenialKind::BindingMeaningChanged)
    );
    let (original, _) = failure.into_parts();
    assert_eq!(original.branch(), branch);
    let document = attach(&host, original, DOCUMENT_IDENTITY);
    assert_eq!(value(&host, &document), SEED_RETENTION);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn refused_attachment_preserves_the_head_for_a_lawful_target_and_model() {
    let host = publish_on_first_program();
    let original = head(&host);
    let branch = original.branch();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let failure = host
        .with_host_advancement(|phase| {
            host.runtime()
                .request(&principal, &scope)
                .attach_history::<RetentionHistory>(
                    &phase,
                    original,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap_err()
        })
        .unwrap();
    assert!(matches!(
        failure.denial(),
        WorthQueryApplicationHistoryDenial::RequestTargetMismatch
    ));
    let (original, _) = failure.into_parts();
    let failure = host
        .with_host_advancement(|phase| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .attach_history::<RetentionHistory>(
                    &phase,
                    original,
                    DocumentRetentionRead {
                        identity: "absent-document".to_owned(),
                    },
                )
                .unwrap_err()
        })
        .unwrap();
    assert!(matches!(failure.denial(), WorthQueryApplicationHistoryDenial::Query(
        worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ScopeResolution(_))));
    let (original, _) = failure.into_parts();
    assert_eq!(original.branch(), branch);
    let document = attach(&host, original, DOCUMENT_IDENTITY);
    assert_eq!(value(&host, &document), SEED_RETENTION);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    assert!(host.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn history_definition_validation_rejects_malformed_portable_membership_with_a_valid_twin() {
    use worth_query_host::facade::declaration::application_history::{
        ApplicationHistoryDefinitionDenial as Denial, ErasedApplicationHistoryDefinition,
    };
    let host = publish_on_first_program();
    let installed = host
        .runtime()
        .installed_schema()
        .installed_history::<RetentionHistory>()
        .unwrap();
    let valid = installed.definition().parts().clone();
    let mut bad_name = valid.clone();
    bad_name.identity = "invalid history".to_owned();
    let mut bad_access = valid.clone();
    bad_access.access = worth_query_host::facade::declaration::application_query::ApplicationQueryBindingReference::from_untrusted_parts({
        let mut parts = valid.access.parts().clone(); parts.query_name.clear(); parts
    });
    let mut missing = valid.clone();
    missing.operations.clear();
    let mut blank_label = valid.clone();
    blank_label.operations[0].label = " ".to_owned();
    let mut duplicate = valid.clone();
    duplicate.operations.push(valid.operations[0].clone());
    for (parts, expected) in [
        (bad_name, Denial::InvalidIdentity),
        (bad_access, Denial::InvalidAccess),
        (missing, Denial::MissingOperations),
        (blank_label, Denial::InvalidOperation),
        (duplicate, Denial::DuplicateOperation),
    ] {
        assert_eq!(
            ErasedApplicationHistoryDefinition::from_untrusted_parts(parts).validate(),
            Err(expected)
        );
    }
    assert_eq!(
        ErasedApplicationHistoryDefinition::from_untrusted_parts(valid).validate(),
        Ok(())
    );
}
