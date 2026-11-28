//! W10C source provenance of a parent-scoped creation: field-level
//! diagnostics refuse the root-path disclosure shape after both evidence
//! capabilities are admitted, with a typed `Unsupported` refusal rather than
//! a permission or capacity one, and the governed receipt stays readable.
use crate::time::controlled_at;

use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    audit::{
        WorthQueryGovernedReadAccess, WorthQuerySourceProvenanceDenial,
        WorthQuerySourceProvenanceRequest,
    },
    WorthQueryRefusalCategory, WorthQueryUserFacingRefusal,
};

use super::grants::SiteRequest;
use super::observers::home;
use super::program::caller;
use super::provenance::{
    AssignRoleProvenance, AssignedGrantBinding, EvidenceCapability, InspectEvidenceOp,
};
use super::schema::{AssignRole, QmsSchema};
use super::{at, temp_home_path};

type Evidence =
    WorthQueryGovernedReadAccess<QmsSchema, EvidenceCapability, InspectEvidenceOp, SiteRequest>;

fn evidence() -> Evidence {
    WorthQueryGovernedReadAccess::new(
        EvidenceCapability::reference(),
        InspectEvidenceOp::reference(),
        SiteRequest {
            site: "north".to_owned(),
        },
    )
}

#[test]
fn a_parent_scoped_creation_refuses_field_detail_by_shape_and_keeps_its_receipt() {
    let path = temp_home_path("provenance-root-path");
    let clock = controlled_at(at(2026, 11, 15, 12, 0));
    let home = home(&path, &clock);
    let alice = caller(&home, "alice");
    let session = home.session(alice.principal.clone());
    let (receipt, ()) = session
        .run(AssignRole {
            site: "north".to_owned(),
            member: 2,
            grant: "bob-review".to_owned(),
        })
        .unwrap_or_else(|denial| panic!("alice grants bob: {denial:?}"))
        .into_parts();

    // Both evidence capabilities admit alice; the inspection itself refuses
    // the Site ← GrantSite ← RoleGrant disclosure shape.
    let refused = session
        .source_provenance::<AssignRoleProvenance>(
            &receipt,
            WorthQuerySourceProvenanceRequest {
                metadata: evidence(),
                diagnostic: evidence(),
            },
        )
        .expect_err("field diagnostics do not admit a root-path disclosure");
    assert!(
        matches!(refused, WorthQuerySourceProvenanceDenial::Inspection(_)),
        "{refused:?}"
    );
    assert!(refused.is_unsupported_field_shape(), "{refused:?}");
    assert_eq!(
        refused.refusal_category(),
        WorthQueryRefusalCategory::Unsupported
    );
    assert!(!refused.retryable());

    // The documented alternative: the governed receipt, reference only.
    let runtime = home.runtime().runtime();
    let request = runtime
        .request(&alice.principal, &alice.scope)
        .on_branch(session.branch());
    let admitted = runtime
        .with_application_advancement(&alice.scope, |phase| {
            request.admit_read_access_in_advancement::<AssignedGrantBinding, _, _, _>(
                &phase,
                &evidence(),
            )
        })
        .unwrap()
        .unwrap_or_else(|denial| panic!("alice holds quality evidence: {denial:?}"));
    let reference = request
        .inspect_source::<AssignRoleProvenance>(&receipt)
        .why_performed()
        .reference_only()
        .execute_governed(admitted)
        .unwrap_or_else(|denial| panic!("the receipt is governed-readable: {denial:?}"));
    assert_eq!(reference.source_commit(), receipt.commit_reference());
}
