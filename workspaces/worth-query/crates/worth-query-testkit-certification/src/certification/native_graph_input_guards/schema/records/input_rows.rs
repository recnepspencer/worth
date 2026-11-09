use super::*;

pub struct SnapshotFacts;
impl Aspect for SnapshotFacts {
    const NAME: &'static str = "SnapshotFacts";
    type Row = SnapshotFactsValue;
}
impl SnapshotFacts {
    pub fn reference() -> AspectSelection<InputModel, Snapshot, Self> {
        aspect(InputModel, Snapshot, Self)
    }
}
pub struct SnapshotFactsValue {
    pub snapshot_identity: String,
    pub snapshot_value: u64,
    pub snapshot_note: Option<Note>,
    pub snapshot_label: Option<String>,
    pub snapshot_kind: Option<String>,
}
impl fragment::Record<SnapshotFactsValue> for InputModelBindings {
    fn record() -> impl Shape<SnapshotFactsValue, Self> {
        field("SnapshotIdentity", |row: &SnapshotFactsValue| {
            row.snapshot_identity.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("SnapshotValue", |row: &SnapshotFactsValue| {
            row.snapshot_value
        })
        .equality_queryable(true)
        .read_only()
        .and("SnapshotNote", |row: &SnapshotFactsValue| {
            row.snapshot_note.clone()
        })
        .equality_queryable(true)
        .and("SnapshotLabel", |row: &SnapshotFactsValue| {
            row.snapshot_label.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("SnapshotKind", |row: &SnapshotFactsValue| {
            row.snapshot_kind.clone()
        })
        .equality_queryable(true)
        .read_only()
        .build(
            |snapshot_identity,
             snapshot_value,
             snapshot_note,
             snapshot_label,
             snapshot_kind,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SnapshotFactsValue {
                    snapshot_identity,
                    snapshot_value,
                    snapshot_note,
                    snapshot_label,
                    snapshot_kind,
                })
            },
        )
    }
}
pub struct MappingFacts;
impl Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingFactsValue;
}
impl MappingFacts {
    pub fn reference() -> AspectSelection<InputModel, Mapping, Self> {
        aspect(InputModel, Mapping, Self)
    }
}
pub struct MappingFactsValue {
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingFactsValue> for InputModelBindings {
    fn record() -> impl Shape<MappingFactsValue, Self> {
        field("ExternalIdentity", |row: &MappingFactsValue| {
            row.external_identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |row: &MappingFactsValue| {
            row.mapping_status
        })
        .equality_queryable(true)
        .build(
            |external_identity, mapping_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingFactsValue {
                    external_identity,
                    mapping_status,
                })
            },
        )
    }
}
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}
impl PrincipalFacts {
    pub fn reference() -> AspectSelection<InputModel, Principal, Self> {
        aspect(InputModel, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity: u64,
}
impl fragment::Record<PrincipalFactsValue> for InputModelBindings {
    fn record() -> impl Shape<PrincipalFactsValue, Self> {
        field("PrincipalIdentity", |row: &PrincipalFactsValue| {
            row.principal_identity
        })
        .equality_queryable(true)
        .read_only()
        .build(|principal_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalFactsValue { principal_identity })
        })
    }
}
pub struct ParentFacts;
impl Aspect for ParentFacts {
    const NAME: &'static str = "ParentFacts";
    type Row = ParentFactsValue;
}
impl ParentFacts {
    pub fn reference() -> AspectSelection<InputModel, Parent, Self> {
        aspect(InputModel, Parent, Self)
    }
}
pub struct ParentFactsValue {
    pub parent_identity: String,
    pub company: Option<String>,
    pub parent_revision: u64,
}
impl fragment::Record<ParentFactsValue> for InputModelBindings {
    fn record() -> impl Shape<ParentFactsValue, Self> {
        field("ParentIdentity", |row: &ParentFactsValue| {
            row.parent_identity.clone()
        })
        .equality_queryable(true)
        .unique()
        .read_only()
        .and("Company", |row: &ParentFactsValue| row.company.clone())
        .equality_queryable(true)
        .and("ParentRevision", |row: &ParentFactsValue| {
            row.parent_revision
        })
        .equality_queryable(true)
        .build(
            |parent_identity, company, parent_revision, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ParentFactsValue {
                    parent_identity,
                    company,
                    parent_revision,
                })
            },
        )
    }
}
pub struct RowFacts;
impl Aspect for RowFacts {
    const NAME: &'static str = "RowFacts";
    type Row = RowFactsValue;
}
impl RowFacts {
    pub fn reference() -> AspectSelection<InputModel, Row, Self> {
        aspect(InputModel, Row, Self)
    }
}
pub struct RowFactsValue {
    pub row_identity: String,
    pub row_value: u64,
}
impl fragment::Record<RowFactsValue> for InputModelBindings {
    fn record() -> impl Shape<RowFactsValue, Self> {
        field("RowIdentity", |row: &RowFactsValue| {
            row.row_identity.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("RowValue", |row: &RowFactsValue| row.row_value)
        .equality_queryable(true)
        .build(
            |row_identity, row_value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(RowFactsValue {
                    row_identity,
                    row_value,
                })
            },
        )
    }
}
