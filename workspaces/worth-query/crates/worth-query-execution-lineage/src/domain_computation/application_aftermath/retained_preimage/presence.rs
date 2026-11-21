//! Authenticated original absence kept separately from an uncaptured field.
use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::InstalledPreImageLocus;
use worth_relational::facade::{identity::KindId, transactions::RecordRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedOutputFields {
    pub output: worth_query_declaration::facade::application_aftermath::PortablePreImageFieldOutput,
    pub target: worth_relational::facade::identity::EntityId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetainedPreImageAbsenceKind {
    Field,
    Aspect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedPreImageAbsence {
    pub locus: InstalledPreImageLocus,
    pub target: RecordRef,
    pub entity_kind: KindId,
    pub locator: AspectFieldLocator,
    pub kind: RetainedPreImageAbsenceKind,
}
