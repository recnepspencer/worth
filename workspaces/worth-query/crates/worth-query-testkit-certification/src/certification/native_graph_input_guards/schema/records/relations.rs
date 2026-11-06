use super::*;

pub struct CurrentSnapshot;
impl Relation for CurrentSnapshot {
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, Some(1), None, Some(1), None, None);
    const NAME: &'static str = "CurrentSnapshot";
    type To = Snapshot;
}
impl CurrentSnapshot {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Row, Snapshot> {
        relation(InputModel, Row, Self)
    }
}
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Mapping, Principal> {
        relation(InputModel, Mapping, Self)
    }
}
pub struct ParentRows;
impl Relation for ParentRows {
    const NAME: &'static str = "ParentRows";
    type To = Row;
}
impl ParentRows {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Parent, Row> {
        relation(InputModel, Parent, Self)
    }
}
pub struct History;
impl Relation for History {
    const NAME: &'static str = "History";
    type To = Evidence;
}
impl History {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Row, Evidence> {
        relation(InputModel, Row, Self)
    }
}
pub struct Current;
impl Relation for Current {
    const NAME: &'static str = "Current";
    type To = Evidence;
}
impl Current {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Row, Evidence> {
        relation(InputModel, Row, Self)
    }
}
pub struct Consequences;
impl Relation for Consequences {
    const NAME: &'static str = "Consequences";
    type To = Consequence;
}
impl Consequences {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Evidence, Consequence> {
        relation(InputModel, Evidence, Self)
    }
}
pub struct SelectedParent;
impl Relation for SelectedParent {
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, Some(1), None, None, None, None);
    const NAME: &'static str = "SelectedParent";
    type To = Parent;
}
impl SelectedParent {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Row, Parent> {
        relation(InputModel, Row, Self)
    }
}
pub struct ParentJournal;
impl Relation for ParentJournal {
    const NAME: &'static str = "ParentJournal";
    type To = Journal;
}
impl ParentJournal {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Parent, Journal> {
        relation(InputModel, Parent, Self)
    }
}
pub struct JournalDetails;
impl Relation for JournalDetails {
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, Some(1), None, Some(1), None, None);
    const NAME: &'static str = "JournalDetails";
    type To = Detail;
}
impl JournalDetails {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Journal, Detail> {
        relation(InputModel, Journal, Self)
    }
}
pub struct GraphHistory;
impl Relation for GraphHistory {
    const NAME: &'static str = "GraphHistory";
    type To = GraphEvidence;
}
impl GraphHistory {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Row, GraphEvidence> {
        relation(InputModel, Row, Self)
    }
}
pub struct ScopeHistory;
impl Relation for ScopeHistory {
    const NAME: &'static str = "ScopeHistory";
    type To = ScopeEvidence;
}
impl ScopeHistory {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Row, ScopeEvidence> {
        relation(InputModel, Row, Self)
    }
}
pub struct RowCells;
impl Relation for RowCells {
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, None, None, Some(1), None, None);
    const NAME: &'static str = "RowCells";
    type To = Cell;
}
impl RowCells {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Row, Cell> {
        relation(InputModel, Row, Self)
    }
}
pub struct RowCellArchive;
impl Relation for RowCellArchive {
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, None, None, Some(1), None, None);
    const NAME: &'static str = "RowCellArchive";
    type To = Cell;
}
impl RowCellArchive {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Row, Cell> {
        relation(InputModel, Row, Self)
    }
}
pub struct CellDetails;
impl Relation for CellDetails {
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(None, None, None, Some(1), None, None);
    const NAME: &'static str = "CellDetails";
    type To = Detail;
}
impl CellDetails {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Cell, Detail> {
        relation(InputModel, Cell, Self)
    }
}
