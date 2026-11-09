use super::*;

#[derive(Clone, Copy)]
pub struct Snapshot;
impl Entity for Snapshot {
    const NAME: &'static str = "Snapshot";
    type Aspects = (SnapshotFacts,);
    type Relations = ();
}
impl Snapshot {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Mapping;
impl Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (MappingFacts,);
    type Relations = (MappingTarget,);
}
impl Mapping {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Parent;
impl Entity for Parent {
    const NAME: &'static str = "Parent";
    type Aspects = (ParentFacts,);
    type Relations = (ParentRows, ParentJournal);
}
impl Parent {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Row;
impl Entity for Row {
    const NAME: &'static str = "Row";
    type Aspects = (RowFacts,);
    type Relations = (
        CurrentSnapshot,
        History,
        Current,
        SelectedParent,
        GraphHistory,
        ScopeHistory,
        RowCells,
        RowCellArchive,
    );
}
impl Row {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Audit;
impl Entity for Audit {
    const NAME: &'static str = "Audit";
    type Aspects = (AuditFacts,);
    type Relations = ();
}
impl Audit {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Evidence;
impl Entity for Evidence {
    const NAME: &'static str = "Evidence";
    type Aspects = (Facts,);
    type Relations = (Consequences,);
}
impl Evidence {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Consequence;
impl Entity for Consequence {
    const NAME: &'static str = "Consequence";
    type Aspects = (ConsequenceFacts,);
    type Relations = ();
}
impl Consequence {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Journal;
impl Entity for Journal {
    const NAME: &'static str = "Journal";
    type Aspects = (JournalFacts,);
    type Relations = (JournalDetails,);
}
impl Journal {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct GraphEvidence;
impl Entity for GraphEvidence {
    const NAME: &'static str = "GraphEvidence";
    type Aspects = (GraphFacts,);
    type Relations = ();
}
impl GraphEvidence {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct ScopeEvidence;
impl Entity for ScopeEvidence {
    const NAME: &'static str = "ScopeEvidence";
    type Aspects = (ScopeFacts,);
    type Relations = ();
}
impl ScopeEvidence {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Cell;
impl Entity for Cell {
    const NAME: &'static str = "Cell";
    type Aspects = (CellFacts,);
    type Relations = (CellDetails,);
}
impl Cell {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Detail;
impl Entity for Detail {
    const NAME: &'static str = "Detail";
    type Aspects = (DetailFacts,);
    type Relations = ();
}
impl Detail {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(InputModel, Self)
    }
}
