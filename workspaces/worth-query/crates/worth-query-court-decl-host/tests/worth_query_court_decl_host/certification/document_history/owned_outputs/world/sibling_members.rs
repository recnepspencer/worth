//! A further model `z` and its members; no court edit touches them.
use super::*;

/// Model `z` (0) owning members `s0`, `s1`, ... (each 5). Nothing when `count` is 0.
pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    count: usize,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    if count == 0 {
        return Ok(());
    }
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Model::reference(),
            WorthQueryApplicationEntityKey::new("z").unwrap(),
        )
        .field(ModelKey::at("ModelKey"), "z".to_owned())
        .field(ModelValue::at("ModelValue"), 0),
    )?;
    for index in 0..count {
        let key = format!("s{index}");
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Member::reference(),
                WorthQueryApplicationEntityKey::new(key.clone()).unwrap(),
            )
            .field(MemberKey::at("MemberKey"), key.clone())
            .field(MemberValue::at("MemberValue"), 5),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ModelMember::reference(),
            format!("owns:z:{key}"),
            WorthQueryApplicationEntityKey::new("z").unwrap(),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
        ))?;
    }
    Ok(())
}
