//! A second member, a third and a fourth model (PR C C4 r2-r5 courts).
use super::*;

/// Member `n` (`value`) owned by model `a`, model `c` (9) and model `d` (4).
pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    value: u64,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Member::reference(),
            WorthQueryApplicationEntityKey::new("n").unwrap(),
        )
        .field(MemberKey::at("MemberKey"), "n".to_owned())
        .field(MemberValue::at("MemberValue"), value),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        ModelMember::reference(),
        "owns:a:n",
        WorthQueryApplicationEntityKey::new("a").unwrap(),
        WorthQueryApplicationEntityKey::new("n").unwrap(),
    ))?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Model::reference(),
            WorthQueryApplicationEntityKey::new("c").unwrap(),
        )
        .field(ModelKey::at("ModelKey"), "c".to_owned())
        .field(ModelValue::at("ModelValue"), 9),
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Model::reference(),
            WorthQueryApplicationEntityKey::new("d").unwrap(),
        )
        .field(ModelKey::at("ModelKey"), "d".to_owned())
        .field(ModelValue::at("ModelValue"), 4),
    )?;
    Ok(())
}
