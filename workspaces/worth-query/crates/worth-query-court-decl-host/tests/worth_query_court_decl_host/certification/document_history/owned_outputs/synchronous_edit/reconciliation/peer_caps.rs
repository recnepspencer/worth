//! The actual minimum cap of a persistent orbit requires full fact comparison.
use super::*;
use worth_query_host::facade::domain::OperationReads;
pub(super) fn minimum<B: ApplicationMutationBinding<Schema>>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, B>,
) -> Result<u64, HandlerExecutionDenial>
where
    CapNext: OperationReads<B::Operation>,
    Cap: OperationReads<B::Operation>,
{
    let seed = reader.resolve_entity(CapKey::at("CapKey"), "cap".to_owned())?;
    let cohort = reader
        .orbit(CapNext::reference(), &seed)
        .map_err(HandlerExecutionDenial::new)?;
    let mut cap = u64::MAX;
    for member in cohort.members() {
        cap = cap.min(reader.field(&member, CapValue::at("CapValue"))?.unwrap());
    }
    Ok(cap)
}
pub(crate) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Cap::reference(),
            WorthQueryApplicationEntityKey::new("cap").unwrap(),
        )
        .field(CapKey::at("CapKey"), "cap".to_owned())
        .field(CapValue::at("CapValue"), 5),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        CapNext::reference(),
        "cap-loop",
        WorthQueryApplicationEntityKey::new("cap").unwrap(),
        WorthQueryApplicationEntityKey::new("cap").unwrap(),
    ))
}
