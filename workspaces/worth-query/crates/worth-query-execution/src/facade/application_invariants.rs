//! Installed domain rules and bounded proposed-state access for application hosts.

pub use crate::domain_computation::primary_graph::application_invariant::{
    CustomInvariantInterruption, WorthQueryApplicationInvariantContext,
    WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantEntityBinding,
    WorthQueryApplicationInvariantExecutionError, WorthQueryApplicationInvariantFieldBinding,
    WorthQueryApplicationInvariantPreparationError, WorthQueryApplicationInvariantReadView,
    WorthQueryApplicationInvariantRelation, WorthQueryApplicationInvariantRelationBinding,
    WorthQueryApplicationInvariantRule, WorthQueryApplicationInvariantScopePlanner,
    WorthQueryApplicationInvariantVerdict, WorthQueryEmptyInvariantRefusal,
    WorthQueryInvariantAccessDenial, WorthQueryInvariantAccessDenialKind,
    WorthQueryInvariantRefusalDecodeDenial,
};
pub use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionWork;
pub use crate::domain_computation::primary_graph::{
    WorthQueryApplicationInvariantFactories, WorthQueryApplicationInvariantSchemaResolver,
};
pub use worth_foundational::facade::AspectValue;
pub use worth_relational::facade::identity::{EntityId, RelationId};
/// Original proposed or committed endpoints carried by relation-law denials.
pub use worth_relational::facade::transactions::EntityReference;

/// Relational owns the cooperative budget supplied to installed application rules.
pub use worth_relational::facade::runtime::CustomInvariantLeaseBudget;
