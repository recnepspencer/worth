//! Actual graph reads, including the previous round's temporary retirement rows.
use super::*;
use std::collections::BTreeMap;
use worth_query_host::facade::domain::OperationReads;
type Target<E> = WorthQueryInvariantMutationTarget<Schema, E>;
pub(super) struct Member {
    pub key: String,
    pub row: Target<FirstOutput>,
    pub source: Target<SecondOutput>,
    pub source_key: String,
    pub children: Vec<Retired>,
}
pub(super) struct Retired {
    pub key: String,
    pub row: Target<ChildOutput>,
}
pub(super) struct Prepared {
    pub model: String,
    pub members: BTreeMap<u64, Member>,
    pub retired: Vec<Retired>,
}
pub(super) fn member_key(model: &str, ordinal: u64) -> String {
    format!("reconciled:member:{model}:{ordinal}")
}
pub(super) fn source_key(model: &str, ordinal: u64) -> String {
    format!("reconciled:source:{model}:{ordinal}")
}
pub(super) fn capture<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
    source: &WorthQueryInvariantEntityIdentity<Schema, Model>,
    next_count: u64,
) -> Result<Prepared, HandlerExecutionDenial>
where
    FirstOf: OperationReads<B::Operation>,
    FirstOutput: OperationReads<B::Operation>,
    ChildOfFirst: OperationReads<B::Operation>,
    ChildOutput: OperationReads<B::Operation>,
    ReconciledSourceOf: OperationReads<B::Operation>,
    SecondOf: OperationReads<B::Operation>,
    SecondOutput: OperationReads<B::Operation>,
{
    let model = reader.field(source, ModelKey::at("ModelKey"))?.unwrap();
    let prefix = format!("reconciled:member:{model}:");
    let mut members = BTreeMap::new();
    let mut retired = Vec::new();
    for edge in reader
        .relations_to(FirstOf::reference(), source)
        .map_err(HandlerExecutionDenial::new)?
    {
        reader
            .entity(edge.from(), FirstOutput::reference())
            .map_err(HandlerExecutionDenial::new)?;
        let key = reader
            .field(edge.from(), FirstKey::at("FirstKey"))?
            .unwrap();
        reader.field(edge.from(), FirstValue::at("FirstValue"))?;
        if let Some(ordinal) = key.strip_prefix(&prefix) {
            let ordinal = ordinal
                .parse::<u64>()
                .expect("fixture-owned member ordinal");
            let sources = reader
                .relations_from(ReconciledSourceOf::reference(), edge.from())
                .map_err(HandlerExecutionDenial::new)?;
            let [source_edge] = sources.as_slice() else {
                panic!("each generated member has one stable source");
            };
            reader
                .entity(source_edge.to(), SecondOutput::reference())
                .map_err(HandlerExecutionDenial::new)?;
            let source_key = reader
                .field(source_edge.to(), SecondKey::at("SecondKey"))?
                .unwrap();
            reader.field(source_edge.to(), SecondValue::at("SecondValue"))?;
            let owners = reader
                .relations_from(SecondOf::reference(), source_edge.to())
                .map_err(HandlerExecutionDenial::new)?;
            let [owner] = owners.as_slice() else {
                panic!("one model owns this source relation");
            };
            assert_eq!(owner.to().entity_id(), source.entity_id());
            let source = reader.mutation_target(source_edge.to())?;
            let children = if ordinal >= next_count {
                capture_children(reader, edge.from())?
            } else {
                Vec::new()
            };
            assert!(members
                .insert(
                    ordinal,
                    Member {
                        key,
                        row: reader.mutation_target(edge.from())?,
                        source,
                        source_key,
                        children
                    }
                )
                .is_none());
        } else if key == format!("reconciled:root:{model}") {
            for row in capture_children(reader, edge.from())? {
                assert!(
                    row.key.starts_with("reconciled:retired:"),
                    "only this producer's retirement facts belong to the root"
                );
                retired.push(row);
            }
        }
    }
    Ok(Prepared {
        model,
        members,
        retired,
    })
}

fn capture_children<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
    source: &WorthQueryInvariantEntityIdentity<Schema, FirstOutput>,
) -> Result<Vec<Retired>, HandlerExecutionDenial>
where
    ChildOfFirst: OperationReads<B::Operation>,
    ChildOutput: OperationReads<B::Operation>,
{
    let mut children = Vec::new();
    for link in reader
        .relations_from(ChildOfFirst::reference(), source)
        .map_err(HandlerExecutionDenial::new)?
    {
        reader
            .entity(link.to(), ChildOutput::reference())
            .map_err(HandlerExecutionDenial::new)?;
        let key = reader.field(link.to(), ChildKey::at("ChildKey"))?.unwrap();
        reader.field(link.to(), ChildValue::at("ChildValue"))?;
        children.push(Retired {
            key,
            row: reader.mutation_target(link.to())?,
        });
    }
    Ok(children)
}
