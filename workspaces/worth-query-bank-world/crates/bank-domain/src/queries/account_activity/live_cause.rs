use worth_query_decl::facade::{
    application_query::ApplicationQueryLiveCauseBinding, worth_query_portable_type,
};

use crate::{
    model::{AccountId, PostingId},
    schema::{
        Account, AccountActivityEffect, AccountIdBinding, ActivityEvent, ActivityEventBinding,
        BankSchema, Posting, PostingIdBinding,
    },
};

use super::AccountActivityQuery;

pub struct AccountActivityLiveCause;
worth_query_portable_type!(AccountActivityLiveCause => "AccountActivityLiveCause");

impl ApplicationQueryLiveCauseBinding<BankSchema, AccountActivityQuery, Account, Posting>
    for AccountActivityLiveCause
{
    type Effect = AccountActivityEffect;
    type PayloadBinding = ActivityEventBinding;
    type ScopeIdentityBinding = AccountIdBinding;
    type TargetIdentityBinding = PostingIdBinding;

    fn effect() -> worth_query_decl::facade::application_schema::ApplicationEffectRef<
        BankSchema,
        Self::Effect,
        ActivityEvent,
    > {
        AccountActivityEffect::reference()
    }

    fn scope_identity(payload: &ActivityEvent) -> AccountId {
        payload.account
    }

    fn target_identity(payload: &ActivityEvent) -> PostingId {
        payload.posting
    }
}
