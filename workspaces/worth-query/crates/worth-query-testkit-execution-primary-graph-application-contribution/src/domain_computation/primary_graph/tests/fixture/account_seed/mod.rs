//! Canonical account entity population for primary-graph fixtures.

use crate::domain_computation::primary_graph::tests::fixture::*;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap;

pub struct AccountSeedSpec<'a> {
    pub key: &'a str,
    pub status: &'a str,
    pub label: &'a str,
    pub note: Option<&'a str>,
}

pub fn bind_account(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    spec: AccountSeedSpec<'_>,
) {
    let mut seed = WorthQueryApplicationEntitySeed::new(
        Account::reference(),
        WorthQueryApplicationEntityKey::new(spec.key).unwrap(),
    )
    .field(AccountIdentity::at("AccountIdentity"), spec.key.to_owned())
    .field(AccountStatus::at("AccountStatus"), spec.status.to_string())
    .field(
        AccountMembershipTag::at("AccountMembershipTag"),
        "open".to_string(),
    )
    .field(AccountLabel::at("AccountLabel"), spec.label.to_string());
    if let Some(note) = spec.note {
        seed = seed.field(AccountNote::at("AccountNote"), note.to_string());
    }
    bootstrap.bind_entity(seed).unwrap();
}
