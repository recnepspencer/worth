use super::*;

pub struct AuditFacts;
impl Aspect for AuditFacts {
    const NAME: &'static str = "AuditFacts";
    type Row = AuditFactsValue;
}
impl AuditFacts {
    pub fn reference() -> AspectSelection<InputModel, Audit, Self> {
        aspect(InputModel, Audit, Self)
    }
}
pub struct AuditFactsValue {
    pub audit_identity: String,
    pub audit_value: u64,
}
impl fragment::Record<AuditFactsValue> for InputModelBindings {
    fn record() -> impl Shape<AuditFactsValue, Self> {
        field("AuditIdentity", |row: &AuditFactsValue| {
            row.audit_identity.clone()
        })
        .read_only()
        .and("AuditValue", |row: &AuditFactsValue| row.audit_value)
        .read_only()
        .build(
            |audit_identity, audit_value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AuditFactsValue {
                    audit_identity,
                    audit_value,
                })
            },
        )
    }
}
pub struct Facts;
impl Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = FactsValue;
}
impl Facts {
    pub fn reference() -> AspectSelection<InputModel, Evidence, Self> {
        aspect(InputModel, Evidence, Self)
    }
}
pub struct FactsValue {
    pub signed_identity: String,
    pub actor: u64,
    pub provenance: ApplicationSigningGraphSubjectProvenance,
}
impl fragment::Record<FactsValue> for InputModelBindings {
    fn record() -> impl Shape<FactsValue, Self> {
        field("SignedIdentity", |row: &FactsValue| {
            row.signed_identity.clone()
        })
        .read_only()
        .and("Actor", |row: &FactsValue| row.actor)
        .read_only()
        .and("Provenance", |row: &FactsValue| row.provenance.clone())
        .build(
            |signed_identity, actor, provenance, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(FactsValue {
                    signed_identity,
                    actor,
                    provenance,
                })
            },
        )
    }
}
pub struct ConsequenceFacts;
impl Aspect for ConsequenceFacts {
    const NAME: &'static str = "ConsequenceFacts";
    type Row = ConsequenceFactsValue;
}
impl ConsequenceFacts {
    pub fn reference() -> AspectSelection<InputModel, Consequence, Self> {
        aspect(InputModel, Consequence, Self)
    }
}
pub struct ConsequenceFactsValue {
    pub record: ApplicationSigningSupersession,
}
impl fragment::Record<ConsequenceFactsValue> for InputModelBindings {
    fn record() -> impl Shape<ConsequenceFactsValue, Self> {
        field("Record", |row: &ConsequenceFactsValue| row.record.clone()).build(
            |record, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ConsequenceFactsValue { record })
            },
        )
    }
}
pub struct JournalFacts;
impl Aspect for JournalFacts {
    const NAME: &'static str = "JournalFacts";
    type Row = JournalFactsValue;
}
impl JournalFacts {
    pub fn reference() -> AspectSelection<InputModel, Journal, Self> {
        aspect(InputModel, Journal, Self)
    }
}
pub struct JournalFactsValue {
    pub journal_name: String,
    pub journal_revision: u64,
    pub journal_actor: u64,
}
impl fragment::Record<JournalFactsValue> for InputModelBindings {
    fn record() -> impl Shape<JournalFactsValue, Self> {
        field("JournalName", |row: &JournalFactsValue| {
            row.journal_name.clone()
        })
        .read_only()
        .and("JournalRevision", |row: &JournalFactsValue| {
            row.journal_revision
        })
        .read_only()
        .and("JournalActor", |row: &JournalFactsValue| row.journal_actor)
        .read_only()
        .build(
            |journal_name, journal_revision, journal_actor, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(JournalFactsValue {
                    journal_name,
                    journal_revision,
                    journal_actor,
                })
            },
        )
    }
}
pub struct GraphFacts;
impl Aspect for GraphFacts {
    const NAME: &'static str = "GraphFacts";
    type Row = GraphFactsValue;
}
impl GraphFacts {
    pub fn reference() -> AspectSelection<InputModel, GraphEvidence, Self> {
        aspect(InputModel, GraphEvidence, Self)
    }
}
pub struct GraphFactsValue {
    pub signed_value: u64,
    pub signed_revision: u64,
    pub signed_company: Option<String>,
    pub graph_actor: u64,
    pub graph_provenance: ApplicationSigningGraphSubjectProvenance,
}
impl fragment::Record<GraphFactsValue> for InputModelBindings {
    fn record() -> impl Shape<GraphFactsValue, Self> {
        field("SignedValue", |row: &GraphFactsValue| row.signed_value)
            .read_only()
            .and("SignedRevision", |row: &GraphFactsValue| {
                row.signed_revision
            })
            .read_only()
            .and("SignedCompany", |row: &GraphFactsValue| {
                row.signed_company.clone()
            })
            .read_only()
            .and("GraphActor", |row: &GraphFactsValue| row.graph_actor)
            .read_only()
            .and("GraphProvenance", |row: &GraphFactsValue| {
                row.graph_provenance.clone()
            })
            .build(
                |signed_value,
                 signed_revision,
                 signed_company,
                 graph_actor,
                 graph_provenance,
                 _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(GraphFactsValue {
                        signed_value,
                        signed_revision,
                        signed_company,
                        graph_actor,
                        graph_provenance,
                    })
                },
            )
    }
}
pub struct ScopeFacts;
impl Aspect for ScopeFacts {
    const NAME: &'static str = "ScopeFacts";
    type Row = ScopeFactsValue;
}
impl ScopeFacts {
    pub fn reference() -> AspectSelection<InputModel, ScopeEvidence, Self> {
        aspect(InputModel, ScopeEvidence, Self)
    }
}
pub struct ScopeFactsValue {
    pub scope_value: u64,
    pub scope_actor: u64,
    pub scope_provenance: ApplicationSigningSubjectProvenance,
}
impl fragment::Record<ScopeFactsValue> for InputModelBindings {
    fn record() -> impl Shape<ScopeFactsValue, Self> {
        field("ScopeValue", |row: &ScopeFactsValue| row.scope_value)
            .read_only()
            .and("ScopeActor", |row: &ScopeFactsValue| row.scope_actor)
            .read_only()
            .and("ScopeProvenance", |row: &ScopeFactsValue| {
                row.scope_provenance.clone()
            })
            .build(
                |scope_value, scope_actor, scope_provenance, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(ScopeFactsValue {
                        scope_value,
                        scope_actor,
                        scope_provenance,
                    })
                },
            )
    }
}
pub struct CellFacts;
impl Aspect for CellFacts {
    const NAME: &'static str = "CellFacts";
    type Row = CellFactsValue;
}
impl CellFacts {
    pub fn reference() -> AspectSelection<InputModel, Cell, Self> {
        aspect(InputModel, Cell, Self)
    }
}
pub struct CellFactsValue {
    pub cell_identity: u64,
    pub cell_kind: u64,
    pub cell_note: Option<Note>,
}
impl fragment::Record<CellFactsValue> for InputModelBindings {
    fn record() -> impl Shape<CellFactsValue, Self> {
        field("CellIdentity", |row: &CellFactsValue| row.cell_identity)
            .read_only()
            .and("CellKind", |row: &CellFactsValue| row.cell_kind)
            .read_only()
            .and("CellNote", |row: &CellFactsValue| row.cell_note.clone())
            .read_only()
            .build(
                |cell_identity, cell_kind, cell_note, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(CellFactsValue {
                        cell_identity,
                        cell_kind,
                        cell_note,
                    })
                },
            )
    }
}
pub struct DetailFacts;
impl Aspect for DetailFacts {
    const NAME: &'static str = "DetailFacts";
    type Row = DetailFactsValue;
}
impl DetailFacts {
    pub fn reference() -> AspectSelection<InputModel, Detail, Self> {
        aspect(InputModel, Detail, Self)
    }
}
pub struct DetailFactsValue {
    pub detail_label: String,
}
impl fragment::Record<DetailFactsValue> for InputModelBindings {
    fn record() -> impl Shape<DetailFactsValue, Self> {
        field("DetailLabel", |row: &DetailFactsValue| {
            row.detail_label.clone()
        })
        .read_only()
        .build(|detail_label, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(DetailFactsValue { detail_label })
        })
    }
}
