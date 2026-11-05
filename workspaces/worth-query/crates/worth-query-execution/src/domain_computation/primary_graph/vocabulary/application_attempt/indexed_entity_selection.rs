//! Moved whole from `domain_computation::primary_graph::application_attempt::fact::indexed_entity_selection`: the items below and every impl they own.
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexedSelectionReobserveDenial {
    UnexpectedFact,
    Lookup(BoundedEntityFieldLookupDenialKind),
    Overflow,
    WorkBudgetExceeded,
}
