//! `worth_query_principal_registry!`: one declaration owns how runtime users
//! map to principals: the account record whose identity is `unique` by
//! construction, the principal binding and its context, and ability-gated
//! onboarding and status actions over an open Home.

mod actions_macro;
mod records_macro;

/// Why a registry onboarding or status change refused before any effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrincipalRegistryRefusal {
    /// An account already maps this external identity.
    IdentityAlreadyOnboarded,
    /// A principal already holds this principal identity.
    PrincipalIdentityTaken,
    /// No account maps this external identity.
    IdentityNotOnboarded,
    /// The identity is too long to key its account and principal.
    InvalidAccountKey,
}

impl WorthQueryPrincipalRegistryRefusal {
    const ALL: [Self; 4] = [
        Self::IdentityAlreadyOnboarded,
        Self::PrincipalIdentityTaken,
        Self::IdentityNotOnboarded,
        Self::InvalidAccountKey,
    ];

    /// The stable domain refusal code a caller matches.
    pub const fn code(self) -> &'static str {
        match self {
            Self::IdentityAlreadyOnboarded => "principal_identity_already_onboarded",
            Self::PrincipalIdentityTaken => "principal_identity_taken",
            Self::IdentityNotOnboarded => "principal_identity_not_onboarded",
            Self::InvalidAccountKey => "principal_account_key_invalid",
        }
    }

    pub const fn message(self) -> &'static str {
        match self {
            Self::IdentityAlreadyOnboarded => "An account already maps this identity.",
            Self::PrincipalIdentityTaken => "A principal already holds this principal identity.",
            Self::IdentityNotOnboarded => "No account maps this identity.",
            Self::InvalidAccountKey => "This identity cannot key an account.",
        }
    }

    /// The registry refusal a run's domain refusal carries, if it is one.
    pub fn of(refusal: &crate::refusal_summary::WorthQueryDomainRefusal) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.code() == refusal.code())
    }

    /// The handler outcome reporting this refusal to the caller unchanged.
    #[doc(hidden)]
    pub fn into_handler_result<T>(
        self,
    ) -> worth_query_execution::facade::primary_graph::HandlerResult<
        T,
        worth_query_execution::facade::primary_graph::WorthQueryOperationBodyDenial,
    > {
        worth_query_execution::facade::primary_graph::HandlerResult::domain_refusal(
            self.code(),
            self.message(),
        )
    }
}

impl std::fmt::Display for WorthQueryPrincipalRegistryRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for WorthQueryPrincipalRegistryRefusal {}

/// Declares a schema's runtime principal registry in module `module`.
///
/// ```text
/// worth_query_principal_registry! {
///     pub mod accounts for Schema {
///         context: Desktop {
///             idempotency: u64 = worth_query_identity!(NS, idempotency),
///             denial: BodyDenial,
///         },
///         principal: Operator => OperatorId = OperatorFacts::operator_id(),
///         directory: Site => SiteId = SiteFacts::site_id(),
///         onboarding: ManageOperators,
///     }
/// }
/// ```
///
/// `principal` names the application's principal entity and its identity
/// field (equality-queryable, read-only). `directory` names the entity whose
/// key scopes every onboarding: who may onboard into it is the `onboarding`
/// ability, which must be scoped to that entity and have an installed policy.
/// The module contains:
///
/// - `PrincipalAccount` with `PrincipalAccountFacts { external_identity,
///   status }`, `PrincipalAccountTarget` (account → principal) and
///   `PrincipalDirectory` (principal → the directory it joined). The
///   identity field is declared `unique`, so two accounts can never map one
///   identity: a racing duplicate is refused at commit, not discovered later
///   as an ambiguous principal.
/// - `Registry`, the installed principal binding, and the `context` named
///   above, which implements `ApplicationBindingContext` for every action,
///   query and eligibility question of the schema.
/// - `OnboardPrincipal` (`OnboardPrincipalBinding`, `PrincipalOnboarded`,
///   `OnboardPrincipalHandler`): creates the principal with its identity and
///   an enabled account in one commit.
/// - `SetPrincipalStatus` (`SetPrincipalStatusBinding`,
///   `PrincipalStatusSet`, `SetPrincipalStatusHandler`): enables or disables
///   an account; a disabled identity no longer authenticates or acts.
/// - `RegistryMembers`: records, binding and onboarding requirements. Add it
///   to the contribution members and both actions as contribution `action`s.
///
/// Each account belongs to one directory: status changes name it and refuse
/// `IdentityNotOnboarded` for an account of any other directory, exactly as
/// for an unknown identity, so one directory's operators learn nothing about
/// another's accounts. External identities stay globally unique, so
/// onboarding one already mapped anywhere refuses `IdentityAlreadyOnboarded`.
/// Seed accounts with `seed_principal(graph, installed, directory, key,
/// principal, identity, status)`: `bind_principal` plus the same directory
/// enrollment onboarding commits.
/// Refusals reach the caller as domain refusals; recover them with
/// [`WorthQueryPrincipalRegistryRefusal::of`].
/// The principal entity is created with only its identity field: its other
/// aspects must be optional or written by a later action.
#[macro_export]
macro_rules! worth_query_principal_registry {
    (
        $vis:vis mod $module:ident for $Schema:ident {
            context: $Context:ident {
                idempotency: $Key:ty = $key_identity:expr,
                denial: $Denial:ty $(,)?
            },
            principal: $Principal:ident => $PrincipalKey:ident =
                $PrincipalFacts:ident :: $principal_id:ident (),
            directory: $Directory:ident => $DirectoryKey:ident =
                $DirectoryFacts:ident :: $directory_id:ident (),
            onboarding: $Onboarding:ident $(,)?
        }
    ) => {
        $vis mod $module {
            #![allow(unused_imports, dead_code)]
            use super::*;
            use $crate::facade::declaration::application_schema::{fragment::*, *};
            use $crate::facade::declaration::authentication::{
                WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
                WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
            };

            $crate::__worth_query_principal_registry_records!(
                $Schema, $Context, $Key, $key_identity, $Denial,
                $Principal, $PrincipalKey, $PrincipalFacts::$principal_id,
                $Directory, $Onboarding
            );
            $crate::__worth_query_principal_registry_actions!(
                $Schema, $Context,
                $Principal, $PrincipalKey, $PrincipalFacts::$principal_id,
                $Directory, $DirectoryKey, $DirectoryFacts::$directory_id
            );
        }
    };
}
