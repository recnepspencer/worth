//! Explicit program-revision deployment grants, governed by Query capabilities.
use super::{BankSchema, Principal};
use worth_query_decl::facade::{
    application_schema::*, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_relation,
};
mod capability;
mod installation;
mod revocation;
pub use capability::*;
pub(crate) use installation::install;
pub use revocation::*;
worth_query_entity!(pub ProgramDeploymentScope for BankSchema);
worth_query_aspect!(pub ProgramDeploymentScopeFacts for BankSchema, ProgramDeploymentScope; identity = AspectIdentity(0x91764001), revision = AspectContractRevision(1),);
worth_query_field!(pub ProgramDeploymentSourceRevision for BankSchema, ProgramDeploymentScope, ProgramDeploymentScopeFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_entity!(pub ProgramDeploymentGrant for BankSchema);
worth_query_aspect!(pub ProgramDeploymentGrantFacts for BankSchema, ProgramDeploymentGrant; identity = AspectIdentity(0x91764002), revision = AspectContractRevision(1),);
worth_query_field!(pub ProgramDeploymentGrantIdentity for BankSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantSourceRevision for BankSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantAction for BankSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantPurpose for BankSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantStatus for BankSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: String => StringApplicationValueBinding, read_write, equality);
worth_query_field!(pub ProgramDeploymentGrantNotBefore for BankSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantNotAfter for BankSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub ProgramDeploymentGrantDepth for BankSchema, ProgramDeploymentGrant, ProgramDeploymentGrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub ProgramDeploymentGrantResource in BankSchema, ProgramDeploymentGrant => ProgramDeploymentScope; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ProgramDeploymentGrantParent in BankSchema, ProgramDeploymentGrant => ProgramDeploymentGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ProgramDeploymentGrantGrantor in BankSchema, Principal => ProgramDeploymentGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ProgramDeploymentGrantGrantee in BankSchema, Principal => ProgramDeploymentGrant; integrity = same_context_unbounded_retain_dangling);
