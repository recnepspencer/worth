//! One authored retention operation; output-only preservation is not a dummy write.
use super::{binding::BodyBinding, output::ScopeOutput, schema::*};
use worth_query_host::facade::declaration::application_schema::*;

pub(super) fn body<const DESTINATION: u64>(
) -> ApplicationOperationBody<DocumentRetentionSchema<DESTINATION>, SetRetention, SetRetentionInput>
{
    if DESTINATION == 31 {
        return SetRetention::reference()
            .body()
            .in_scope(Document::reference())
            .unwrap()
            .preserve_scope_output::<BodyBinding<DESTINATION>, ScopeOutput<DESTINATION>>()
            .unwrap()
            .finish()
            .unwrap();
    }
    let authored_destination = if matches!(DESTINATION, 24..=29) {
        20
    } else {
        DESTINATION
    };
    let body = SetRetention::reference::<DocumentRetentionSchema<DESTINATION>>()
        .body()
        .expect(DocumentRetentionField::reference(), 7)
        .unwrap();
    let body = if DESTINATION == 30 {
        body.in_scope(Document::reference()).unwrap()
    } else {
        body
    };
    let body = if DESTINATION >= 100 {
        body.assign_input(
            DocumentRetentionField::reference(),
            match DESTINATION {
                101 => "identity",
                102 => "missing",
                _ => "retention_days",
            },
        )
        .unwrap()
    } else {
        body.assign(DocumentRetentionField::reference(), authored_destination)
            .unwrap()
    };
    if matches!(DESTINATION, 20 | 21 | 23..=29 | 36) {
        let creation = body
            .create_in_scope(Audit::reference(), "audit")
            .unwrap()
            .initialize_input(AuditIdentityField::reference(), "identity")
            .unwrap()
            .initialize(AuditRetentionField::reference(), authored_destination)
            .unwrap();
        let creation = creation
            .link_from_scope(DocumentMembers::reference(), "audit-owner")
            .unwrap();
        let creation = if DESTINATION == 36 {
            creation
                .initialize_from_principal_identity::<BodyBinding<DESTINATION>, _, _, _, _, _>(
                    AuditActorField::reference(),
                )
                .unwrap()
        } else {
            creation
        };
        let creation = if DESTINATION == 23 {
            creation
                .initialize_from_scope(
                    DocumentAuditSourceField::reference(),
                    AuditPriorRetentionField::reference(),
                )
                .unwrap()
        } else {
            creation
                .initialize_from_scope(
                    DocumentRetentionField::reference(),
                    AuditPriorRetentionField::reference(),
                )
                .unwrap()
        };
        creation.finish().unwrap()
    } else {
        body.finish().unwrap()
    }
}
