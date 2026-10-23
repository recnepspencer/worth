//! Unrelated scopes hold genuine source/reversal history before the measured source.
use super::super::super::source_delivery::correction as commands;
use super::*;
use crate::checkpoint_recovery::computation_partition::entry_correction::EntryCorrection;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntitySeed as Seed, WorthQueryApplicationRelationSeed as Relation,
};

pub(super) const ENTRY_BASE: u64 = 1_000_000;
pub(super) fn scope(index: usize) -> String {
    format!("locality-source-{index}")
}
pub(super) fn seed(graph: &mut Graph, count: usize) {
    for index in 0..count {
        let scope = scope(index);
        let names = [scope.clone(), format!("{scope}:b"), format!("{scope}:c")];
        for (name, (x, y)) in names.iter().zip([(1, 1), (2, 1), (1, 2)]) {
            graph
                .bind_entity(
                    Seed::new(Body::reference(), facts::key(name))
                        .field(BodyKey::reference(), name.clone())
                        .field(PositionX::reference(), length(x))
                        .field(PositionY::reference(), length(y))
                        .field(Length::reference(), length(1)),
                )
                .unwrap();
        }
        for at in 0..3 {
            graph
                .bind_relation(Relation::new(
                    PlanarSuccessor::reference(),
                    format!("{scope}:edge:{at}"),
                    facts::key(&names[at]),
                    facts::key(&names[(at + 1) % 3]),
                ))
                .unwrap();
        }
        let set = format!("{scope}:entries");
        facts::seed_set(graph, &set, -0.0);
        facts::seed_owned_set(graph, &scope, &set);
        let entry = format!("{scope}:entry");
        facts::seed_entry(
            graph,
            &entry,
            &crate::checkpoint_recovery::computation_partition::execution::entry(
                ENTRY_BASE + index as u64,
                0,
                2.0,
            ),
        );
        facts::seed_member(graph, &set, &entry);
        super::super::super::courtroom::correction_grant::seed_grant(
            graph,
            &scope,
            &format!("{scope}:grant"),
            1,
        );
    }
}
pub(super) fn history(app: &correction::App, request: &Request<'_, '_, '_>, count: usize) {
    for index in 0..count {
        let key = 0x9190_0000 + 2 * index as u64;
        let source = commands::source(
            app,
            request,
            EntryCorrection {
                scope_key: scope(index),
                entry: ENTRY_BASE + index as u64,
                mask: 2.0_f64.to_bits() ^ 3.0_f64.to_bits(),
                expected: None,
            },
            key,
        );
        commands::reverse(app, request, &source, &(key + 1).to_string());
    }
}
