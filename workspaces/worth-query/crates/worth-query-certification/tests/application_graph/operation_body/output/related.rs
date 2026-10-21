//! ExactlyOne output refuses two lawful targets before producing any receipt.
use super::*;
#[path = "related/binding.rs"]
pub(in super::super) mod binding;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestMutationDenial as RequestDenial,
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryApplicationAttemptDenial as Attempt,
        WorthQueryApplicationAttemptDenialKind as Kind, WorthQueryPrimaryGraphBootstrap,
        WorthQueryPrimaryGraphInstallationDenial,
    },
};

pub struct RelatedOutput<const MODE: u64>;
pub struct RelatedOutputs<const MODE: u64>;
impl<const MODE: u64> ApplicationMutationOutputContract<DocumentRetentionSchema<MODE>>
    for RelatedOutputs<MODE>
{
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<RelatedOutput<MODE> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
impl<const MODE: u64> WorthQueryApplicationOutputRole for RelatedOutput<MODE> {
    type Schema = DocumentRetentionSchema<MODE>;
    type Contract = RelatedOutputs<MODE>;
    type Entity = Audit;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "selected-audit";
}

pub(in super::super) fn seed_members<const MODE: u64>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema<MODE>>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let members: &[&str] = match MODE {
        25..=27 | 29 | 37 => &["body-member"],
        38 => &["body-member", "second-member"],
        _ => &[],
    };
    for member in members {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Audit::reference(),
                WorthQueryApplicationEntityKey::new(*member).unwrap(),
            )
            .field(
                AuditIdentityField::reference(),
                if *member == "body-member" {
                    "member"
                } else {
                    member
                }
                .to_string(),
            )
            .field(AuditRetentionField::reference(), 7)
            .field(AuditPriorRetentionField::reference(), 7),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            DocumentMembers::reference(),
            format!("{member}-edge"),
            WorthQueryApplicationEntityKey::new("body-document").unwrap(),
            WorthQueryApplicationEntityKey::new(*member).unwrap(),
        ))?;
    }
    Ok(())
}

#[test]
fn related_output_of_one_target_is_preserved_without_a_dummy_write() {
    let single = install::<37>();
    let scope = request_scope();
    let principal = authenticate_operator(single.runtime().installed_schema(), &scope);
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.into(),
        retention_days: 99,
        ..Default::default()
    };
    let Outcome::Committed { receipt, .. } = single
        .request(&principal, &scope)
        .advanced()
        .mutate(binding::RelatedIntent(input.clone()))
        .without_source()
        .idempotency(&370)
        .execute_in_program(&single, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    else {
        panic!("one target")
    };
    let selected = receipt
        .outputs_of::<RelatedOutputs<37>>()
        .unwrap()
        .entity::<RelatedOutput<37>>()
        .unwrap()
        .entity_id();
    assert_eq!(
        read_retention(&single),
        7,
        "output-only body has no dummy writes"
    );
    let Outcome::AlreadyCommitted(replay) = single
        .request(&principal, &scope)
        .advanced()
        .mutate(binding::RelatedIntent(input.clone()))
        .without_source()
        .idempotency(&370)
        .execute_in_program(&single, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    else {
        panic!("replay")
    };
    assert_eq!(
        replay
            .outputs_of::<RelatedOutputs<37>>()
            .unwrap()
            .entity::<RelatedOutput<37>>()
            .unwrap()
            .entity_id(),
        selected
    );
}

#[test]
fn related_output_requires_one_target_not_an_arbitrary_member_of_two() {
    let ambiguous = install::<38>();
    let scope = request_scope();
    let principal = authenticate_operator(ambiguous.runtime().installed_schema(), &scope);
    let attempt = ambiguous
        .request(&principal, &scope)
        .advanced()
        .mutate(binding::RelatedIntent(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.into(),
            retention_days: 99,
            ..Default::default()
        }))
        .without_source()
        .idempotency(&380)
        .execute_in_program(&ambiguous, ExecutionAllocationPolicy::SystemAllocation);
    let Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause))) = attempt
    else {
        panic!("two targets must not produce a receipt")
    };
    let cause = cause.downcast::<Attempt>().unwrap();
    assert_eq!(cause.kind(), Kind::AmbiguousRelation);
    assert_eq!(cause.subject(), "DocumentMembers");
    assert_eq!(read_retention(&ambiguous), 7);
    for identity in ["member", "second-member"] {
        let result = ambiguous
            .request(&principal, &scope)
            .query(audit_query::AuditRead {
                identity: identity.into(),
            })
            .execute()
            .unwrap();
        let row = &result.rows()[0];
        assert_eq!(row.identity, identity);
        assert_eq!((row.retention_days, row.prior_retention_days), (7, 7));
    }
}
