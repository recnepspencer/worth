//! Static graph fixtures use native bootstrap; disputed changes use installed actions.
use super::{operator_identity::operator_identity, program, schema::*};
use worth_query_host::facade::{
    application_installation::{
        in_memory_program, WorthQueryInMemoryApplicationLimits, WorthQueryProgramApplicationRuntime,
    },
    declaration::authentication::WorthQueryPrincipalMappingStatus,
    primary_graph::{
        SignalConditionalEvaluationBudget, WorthQueryApplicationEntityKey as Key,
        WorthQueryApplicationEntitySeed as Entity, WorthQueryApplicationPrincipalKey,
        WorthQueryApplicationRelationSeed as Relation,
    },
    runtime,
};
pub type App = WorthQueryProgramApplicationRuntime<Schema, program::Program>;
pub fn install() -> App {
    let schema = program::schema();
    let limits = WorthQueryInMemoryApplicationLimits::new(
        crate::world_resources::world_resources(256, 4 * 1024 * 1024, 256),
        runtime::WorthQueryApplicationCandidateResourceProfile::physical_resources(
            200_000,
            128 * 1024 * 1024,
        )
        .unwrap(),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(5120, 2048, usize::MAX, 128)
            .unwrap(),
        SignalConditionalEvaluationBudget::development(),
    );
    in_memory_program(
        program::validated(&schema),
        schema,
        ((),),
        limits,
        |_phase, graph, installed| {
            graph.bind_principal(
                &installed
                    .principal_binding(PrincipalBinding::reference())
                    .unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for (id, company) in [
                ("parent-a", Some("company-a")),
                ("parent-b", Some("company-b")),
                ("parent-c", Some("company-a")),
                ("parent-absent", None),
                ("parent-overflow", None),
            ] {
                let entity = Entity::new(Parent::reference(), Key::new(id).unwrap())
                    .field(ParentIdentity::reference(), id.to_string())
                    .field(
                        ParentRevision::reference(),
                        if id == "parent-overflow" { u64::MAX } else { 1 },
                    );
                graph.bind_entity(if let Some(company) = company {
                    entity.field(Company::reference(), company.to_string())
                } else {
                    entity
                })?;
            }
            for row in [
                "row-a",
                "row-mismatch",
                "row-absent",
                "row-missing",
                "row-ambiguous",
                "row-sibling",
                "row-overflow",
            ] {
                graph.bind_entity(
                    Entity::new(Row::reference(), Key::new(row).unwrap())
                        .field(RowIdentity::reference(), row.to_string())
                        .field(RowValue::reference(), 7),
                )?;
            }
            for (parent, row) in [
                ("parent-a", "row-a"),
                ("parent-b", "row-mismatch"),
                ("parent-absent", "row-absent"),
                ("parent-a", "row-ambiguous"),
                ("parent-c", "row-ambiguous"),
                ("parent-a", "row-sibling"),
                ("parent-overflow", "row-overflow"),
            ] {
                graph.bind_relation(Relation::new(
                    ParentRows::reference(),
                    format!("{parent}-{row}"),
                    Key::new(parent).unwrap(),
                    Key::new(row).unwrap(),
                ))?;
            }
            Ok(())
        },
    )
    .unwrap()
}
