//! ExactlyOne output refuses two lawful targets before producing any receipt.
pub use super::*;
#[path = "related/binding.rs"]
pub mod binding;
pub use worth_query_host::facade::{
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

pub fn seed_members<const MODE: u64>(
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
