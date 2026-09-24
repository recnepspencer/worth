//! Canonical account entity population for primary-graph fixtures.

use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap;

pub(super) struct AccountSeedSpec<'a> {
    pub(super) key: &'a str,
    pub(super) status: &'a str,
    pub(super) label: &'a str,
    pub(super) note: Option<&'a str>,
}

pub(super) fn bind_account(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    spec: AccountSeedSpec<'_>,
) {
    let mut seed = WorthQueryApplicationEntitySeed::new(
        Account::reference(),
        WorthQueryApplicationEntityKey::new(spec.key).unwrap(),
    )
    .field(AccountIdentity::reference(), spec.key.to_owned())
    .field(AccountStatus::reference(), spec.status.to_string())
    .field(AccountMembershipTag::reference(), "member".to_string())
    .field(AccountLabel::reference(), spec.label.to_string());
    if let Some(note) = spec.note {
        seed = seed.field(AccountNote::reference(), note.to_string());
    }
    bootstrap.bind_entity(seed).unwrap();
}
