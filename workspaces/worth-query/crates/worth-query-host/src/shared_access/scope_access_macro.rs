//! `worth_query_scope_access!`: one declaration owns a schema's stock scope
//! grants, read/watch capabilities, governed disclosure and grant/revoke
//! actions, all over typed vocabulary.

/// Why a stock scope grant refused before any effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryScopeGrantRefusal {
    /// The grantee's principal mapping is disabled.
    GranteeMappingDisabled,
    /// The grantee's mapping links no principal.
    GranteeUnmapped,
    /// The grant identifier is not a valid entity key.
    InvalidGrantKey,
}

impl std::fmt::Display for WorthQueryScopeGrantRefusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::GranteeMappingDisabled => "the grantee's principal mapping is disabled",
            Self::GranteeUnmapped => "the grantee's mapping links no principal",
            Self::InvalidGrantKey => "the grant identifier is not a valid entity key",
        })
    }
}

impl std::error::Error for WorthQueryScopeGrantRefusal {}

/// Declares a schema's stock scope access in module `module`.
///
/// ```text
/// worth_query_scope_access! {
///     pub mod project_access for Schema {
///         context: DesktopContext,
///         scope: Inbox => InboxId = InboxFacts::inbox_id(),
///         principal: Owner => OwnerFacts::owner_id(),
///         mapping: Mapping {
///             identity: MappingFacts::external_identity(),
///             status: MappingFacts::mapping_status(),
///         } => MappingOwner,
///         reads: [InboxFacts::inbox_id(), InboxFacts::inbox_name()],
///     }
/// }
/// ```
///
/// The scope key is a `String` field; `principal` names the principal
/// binding's identity field; `mapping` names the external-identity mapping and
/// its relation to the principal; `reads` lists the scope fields a governed
/// read or watch may observe. The module contains:
///
/// - `ScopeAccessMembers`: the grant records, capability contexts, read/watch
///   operations and capability contracts. Add it to the contribution members.
/// - `GrantScopeAccess`/`GrantScopeAccessBinding`/`ScopeAccessGranted` with
///   `ScopeGrantHandler`, and `RevokeScopeAccess`/`RevokeScopeAccessBinding`/
///   `ScopeAccessRevoked` with `ScopeRevokeHandler`: register both as
///   contribution `action` members (each declares its pure operation).
/// - `disclosure()` and `scope_view()`: the governed disclosure of a shared
///   query, so `disclose_field_by(field, scope_view(), influence)` names no
///   classification or disclosure string.
///
/// Every grant dimension is typed: access is [`super::ScopeAccess`], action,
/// purpose and disclosure are [`super::ScopeGrantAction`],
/// [`super::ScopeGrantPurpose`] and [`super::ScopeGrantDisclosure`]. Stock
/// item names are fixed, so a schema declares one stock scope access.
#[macro_export]
macro_rules! worth_query_scope_access {
    (
        $vis:vis mod $module:ident for $Schema:ident {
            context: $Context:ident,
            scope: $Scope:ident => $ScopeKey:ident = $ScopeFacts:ident :: $scope_key:ident (),
            principal: $Principal:ident => $PrincipalFacts:ident :: $principal_id:ident (),
            mapping: $Mapping:ident {
                identity: $IdentityFacts:ident :: $mapping_identity:ident (),
                status: $StatusFacts:ident :: $mapping_status:ident () $(,)?
            } => $MappingTarget:ident,
            reads: [$($ReadFacts:ident :: $read:ident ()),+ $(,)?] $(,)?
        }
    ) => {
        $vis mod $module {
            #![allow(unused_imports, dead_code)]
            use super::*;
            use $crate::facade::declaration::application_capability::*;
            use $crate::facade::declaration::application_schema::{fragment::*, *};
            use $crate::facade::shared_access::{
                ScopeAccess, ScopeAccessBinding, ScopeGrantAction, ScopeGrantActionBinding,
                ScopeGrantDisclosure, ScopeGrantDisclosureBinding, ScopeGrantPurpose,
                ScopeGrantPurposeBinding,
            };

            $crate::__worth_query_scope_access_schema!($Schema, $Scope, $Principal);
            $crate::__worth_query_scope_access_contracts!(
                $Schema, $Scope, $Principal, $ScopeFacts::$scope_key,
                [$($ReadFacts::$read),+]
            );
            $crate::__worth_query_scope_access_actions!(
                $Schema, $Context, $Scope, $ScopeKey, $ScopeFacts::$scope_key,
                $Principal, $PrincipalFacts::$principal_id,
                $Mapping, $IdentityFacts::$mapping_identity, $StatusFacts::$mapping_status,
                $MappingTarget
            );
        }
    };
}
