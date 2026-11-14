//! Authenticated original absence kept separately from an uncaptured field.
use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::InstalledPreImageLocus;
use worth_relational::facade::{identity::KindId, transactions::RecordRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct RetainedChangedFieldOutput {
    pub(in crate::domain_computation) output:
        worth_query_declaration::facade::application_aftermath::PortableChangedFieldOutput,
    pub(in crate::domain_computation) target: worth_relational::facade::identity::EntityId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum RetainedPreImageAbsenceKind {
    Field,
    Aspect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct RetainedPreImageAbsence {
    pub(in crate::domain_computation) locus: InstalledPreImageLocus,
    pub(in crate::domain_computation) target: RecordRef,
    pub(in crate::domain_computation) entity_kind: KindId,
    pub(in crate::domain_computation) locator: AspectFieldLocator,
    pub(in crate::domain_computation) kind: RetainedPreImageAbsenceKind,
}
