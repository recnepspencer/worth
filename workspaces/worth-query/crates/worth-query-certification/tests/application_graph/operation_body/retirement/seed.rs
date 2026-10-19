//! Bootstrap data only: test mutations execute installed bodies, never host cleanup.
use super::{super::schema::*, declaration::*};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey as Key, WorthQueryApplicationEntitySeed as Entity,
    WorthQueryApplicationRelationSeed as Relation, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
};

pub fn seed<const M: u64>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema<M>>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    if !matches!(M, 120..=125) {
        return Ok(());
    }
    graph.bind_entity(
        Entity::new(Document::reference(), Key::new("outside-document").unwrap())
            .field(DocumentIdentityField::reference(), "outside".to_string())
            .field(DocumentRetentionField::reference(), 7)
            .field(DocumentAuditSourceField::reference(), 7),
    )?;
    let holders: &[(&str, &str, Option<u64>)] = if M == 123 {
        &[("holder-outside", "outside-document", Some(999))]
    } else if M == 121 {
        &[
            ("holder-a", "body-document", Some(101)),
            ("holder-b", "body-document", None),
            ("holder-c", "body-document", Some(303)),
            ("holder-outside", "outside-document", Some(999)),
        ]
    } else {
        &[
            ("holder-a", "body-document", Some(101)),
            ("holder-b", "body-document", None),
            ("holder-outside", "outside-document", Some(999)),
        ]
    };
    for (holder, document, selected) in holders {
        graph.bind_entity(
            Entity::new(Audit::reference(), Key::new(*holder).unwrap())
                .field(AuditIdentityField::reference(), holder.to_string())
                .field(AuditRetentionField::reference(), 7)
                .field(AuditPriorRetentionField::reference(), 7),
        )?;
        graph.bind_relation(Relation::new(
            OwnedRows::reference(),
            format!("owner-{holder}"),
            Key::new(*document).unwrap(),
            Key::new(*holder).unwrap(),
        ))?;
        if let Some(value) = selected {
            let snapshot = format!("snapshot-{value}");
            graph.bind_entity(
                Entity::new(Snapshot::reference(), Key::new(&snapshot).unwrap())
                    .field(SnapshotIdentity::reference(), *value),
            )?;
            graph.bind_relation(Relation::new(
                CurrentSelection::reference(),
                format!("current-{holder}"),
                Key::new(*holder).unwrap(),
                Key::new(&snapshot).unwrap(),
            ))?;
            if M == 122 && *holder == "holder-a" {
                graph.bind_relation(Relation::new(
                    AlternateSelection::reference(),
                    "alternate-a",
                    Key::new(*holder).unwrap(),
                    Key::new(&snapshot).unwrap(),
                ))?;
            }
        }
    }
    Ok(())
}
