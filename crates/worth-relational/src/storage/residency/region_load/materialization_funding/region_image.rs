//! Typed REG6 allocation preflight, before Serde or native columns allocate.
use super::super::super::region_image::{
    RegionAdjacencyKinds, RegionHeadImage, RegionHistoryImage, RegionHistoryRow,
};
use super::*;
use crate::durability::data::*;
use crate::identity::data::{KindId, PartitionId, RelationId, VersionId};
use crate::storage::{
    data::{RecordLifecycleState, RelationalFieldRevision},
    partition::{AdjacencySet, RegionRelationProjection},
    substrate::*,
};
use crate::symbols::data::Symbol;
use std::collections::BTreeSet;

fn revision_maps(raw: Raw<'_>, fields: bool, request: &ExecutionRequest<'_, '_>) -> Result<Quote> {
    let mut quote = Quote::default();
    let count = sequence(raw, request, |row| {
        if row.is_none() {
            return Ok(());
        }
        let count = pairs(row, request, |_, _| Ok(()))?;
        quote.add(if fields {
            Quote::bytes(
                tree::<(Symbol, Symbol), RelationalFieldRevision>(count)?,
                map::<(Symbol, Symbol), RelationalFieldRevision>(count)?,
            )
        } else {
            Quote::bytes(tree::<Symbol, u64>(count)?, map::<Symbol, u64>(count)?)
        })
    })?;
    quote.add(if fields {
        Quote::bytes(
            vector::<Option<BTreeMap<(Symbol, Symbol), RelationalFieldRevision>>>(count)?,
            column::<Option<FieldRevisions>>(count)?,
        )
    } else {
        Quote::bytes(
            vector::<BTreeMap<Symbol, u64>>(count)?,
            column::<AspectVersions>(count)?,
        )
    })?;
    Ok(quote)
}
fn diagnostics(raw: Raw<'_>, request: &ExecutionRequest<'_, '_>) -> Result<Quote> {
    let mut quote = Quote::default();
    let count = sequence(raw, request, |row| {
        let count = pairs(row, request, |_, value| {
            let n = text(value, request)?;
            quote.add(Quote::bytes(n, n))
        })?;
        quote.add(Quote::bytes(
            tree::<Symbol, String>(count)?,
            tree::<Symbol, String>(count)?,
        ))
    })?;
    quote.add(Quote::bytes(
        vector::<BTreeMap<Symbol, String>>(count)?,
        column::<StorageAllocation<BTreeMap<Symbol, String>>>(count)?,
    ))?;
    Ok(quote)
}
fn bitset(raw: Raw<'_>, request: &ExecutionRequest<'_, '_>) -> Result<Quote> {
    let mut quote = Quote::default();
    record(raw, &["words", "sparse_words"], request, |i, value| {
        let count = sequence(value, request, |_| Ok(()))?;
        quote.add(Quote::bytes(
            if i == 0 {
                vector::<u64>(count)?
            } else {
                vector::<(u64, u64)>(count)?
            },
            map::<usize, u64>(count)?,
        ))
    })?;
    Ok(quote)
}
fn arena(
    raw: Raw<'_>,
    entity: bool,
    contract: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    let mut quote = Quote::default();
    record(
        raw,
        &[
            "slots",
            "generations",
            "lifecycle",
            "kind_ids",
            "metadata_history",
            "created_at",
            "retired_at",
            "aspect_versions",
            "field_revisions",
            "extra",
            "diagnostics_enrichment",
            "branch_pins",
            "replay_pins",
            "snapshot_pins",
            "live_bitset",
            "reclaimable_bitset",
            "free_list",
        ],
        request,
        |i, value| {
            let next = match i {
                0 => scalars::<u64>(value, request)?,
                1 => {
                    let mut q = scalars::<u32>(value, request)?;
                    let count = sequence(value, request, |_| Ok(()))?;
                    q.native = sum(
                        q.native,
                        sum(
                            column::<PartitionId>(count)?,
                            sum(
                                column::<StorageAllocation<Option<RecordHistoryPrefix>>>(count)?,
                                map::<u64, usize>(count)?,
                            )?,
                        )?,
                    )?;
                    // Empty slots invokes the existing dense slot-directory fallback.
                    q.native = sum(q.native, column::<u64>(count)?)?;
                    q
                }
                2 => scalars::<RecordLifecycleState>(value, request)?,
                3 => scalars::<Option<KindId>>(value, request)?,
                4 => aspect_state::histories(value, entity, contract, request)?,
                5 => scalars::<VersionId>(value, request)?,
                6 => scalars::<Option<VersionId>>(value, request)?,
                7 => revision_maps(value, false, request)?,
                8 => revision_maps(value, true, request)?,
                9 => aspect_state::extra(value, entity, contract, request)?,
                10 => diagnostics(value, request)?,
                11..=13 => {
                    let count = sequence(value, request, |_| Ok(()))?;
                    Quote::bytes(vector::<u32>(count)?, map::<usize, u32>(count)?)
                }
                14 | 15 => bitset(value, request)?,
                16 => Quote::bytes(vector::<u64>(sequence(value, request, |_| Ok(()))?)?, 0),
                _ => return Err(damaged()),
            };
            quote.add(next)
        },
    )?;
    Ok(quote)
}
fn adjacency(raw: Raw<'_>, request: &ExecutionRequest<'_, '_>) -> Result<Quote> {
    let mut quote = Quote::default();
    let count = sequence(raw, request, |row| {
        record(
            row,
            &["slot", "relations", "structural_revisions"],
            request,
            |i, value| {
                if i == 1 {
                    let n = sequence(value, request, |_| Ok(()))?;
                    quote.add(Quote::bytes(
                        vector::<RelationId>(n)?,
                        map::<RelationId, ()>(n)?,
                    ))?;
                }
                if i == 2 {
                    let n = sequence(value, request, |_| Ok(()))?;
                    quote.add(Quote::bytes(
                        vector::<(KindId, VersionId)>(n)?,
                        map::<KindId, VersionId>(n)?,
                    ))?;
                }
                Ok(())
            },
        )
    })?;
    quote.add(Quote::bytes(
        vector::<DurableAdjacencyEntry>(count)?,
        map::<usize, AdjacencySet>(count)?,
    ))?;
    Ok(quote)
}
fn projection(raw: Raw<'_>, request: &ExecutionRequest<'_, '_>) -> Result<Quote> {
    let mut incidences = 0usize;
    let count = sequence(raw, request, |row| {
        record(
            row,
            &["relation", "kind", "outgoing", "incoming"],
            request,
            |i, value| {
                if i >= 2 {
                    let n = sequence(value, request, |_| Ok(()))?;
                    incidences = incidences.checked_add(n).ok_or_else(overflow)?;
                }
                Ok(())
            },
        )
    })?;
    projection_storage(count, incidences)
}
/// Preflight decoded, verified and retained claims, including peer memberships.
pub(in crate::storage::residency) fn projection_storage(
    claims: usize,
    incidences: usize,
) -> Result<Quote> {
    let sets = tree::<usize, ()>(incidences)?;
    let rows = vector::<RegionRelationProjection>(claims)?;
    let mut wire = sum(times(sets, 4)?, times(rows, 3)?)?;
    wire = sum(
        wire,
        times(tree::<RelationId, RegionRelationProjection>(claims)?, 2)?,
    )?;
    wire = sum(wire, tree::<usize, Vec<RegionRelationProjection>>(claims)?)?;
    wire = sum(
        wire,
        tree::<u64, BTreeMap<KindId, BTreeSet<RelationId>>>(incidences)?,
    )?;
    wire = sum(wire, tree::<KindId, BTreeSet<RelationId>>(incidences)?)?;
    wire = sum(wire, tree::<RelationId, ()>(incidences)?)?;
    let mut native = sum(rows, sets)?;
    native = sum(native, map::<usize, AdjacencySet>(incidences)?)?;
    native = sum(
        native,
        map::<KindId, SharedMap<RelationId, ()>>(incidences)?,
    )?;
    native = sum(native, map::<RelationId, ()>(incidences)?)?;
    Ok(Quote::bytes(wire, native))
}
fn kinds(raw: Raw<'_>, request: &ExecutionRequest<'_, '_>) -> Result<Quote> {
    let mut quote = Quote::default();
    let count = sequence(raw, request, |row| {
        record(row, &["slot", "kinds"], request, |i, value| {
            if i == 1 {
                let n = sequence(value, request, |pair| {
                    record(pair, &["kind", "ids"], request, |i, value| {
                        if i == 1 {
                            let n = sequence(value, request, |_| Ok(()))?;
                            quote.wire = sum(
                                quote.wire,
                                sum(vector::<RelationId>(n)?, tree::<RelationId, ()>(n)?)?,
                            )?;
                        }
                        Ok(())
                    })
                })?;
                quote.wire = sum(quote.wire, vector::<(KindId, Vec<RelationId>)>(n)?)?;
            }
            Ok(())
        })
    })?;
    quote.wire = sum(quote.wire, vector::<RegionAdjacencyKinds>(count)?)?;
    Ok(quote)
}
fn frontiers(raw: Raw<'_>, request: &ExecutionRequest<'_, '_>) -> Result<Quote> {
    let mut quote = Quote::default();
    record(raw, &["entities", "relations"], request, |_, rows| {
        let count = sequence(rows, request, |row| {
            record(row, &["slot", "frontier"], request, |i, value| {
                if i == 1 {
                    record(value, &["siblings", "last"], request, |i, value| {
                        if i == 0 {
                            let n = sequence(value, request, |_| Ok(()))?;
                            let bytes = vector::<HistoryFrontierSibling>(n)?;
                            quote
                                .add(Quote::bytes(bytes, sum(bytes, head_commitment_nodes(n)?)?))?;
                        }
                        Ok(())
                    })?;
                }
                Ok(())
            })
        })?;
        quote.wire = sum(quote.wire, vector::<(u64, HistoryAppendFrontier)>(count)?)?;
        quote.native = sum(
            quote.native,
            sum(
                times(shared::<RecordHistoryOpening>()?, count)?,
                head_commitment_nodes(count)?,
            )?,
        )?;
        Ok(())
    })?;
    Ok(quote)
}
pub(in crate::storage::residency) fn head(
    raw: Raw<'_>,
    contract: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    work(request, raw.0.len() as u64)?;
    let mut quote = Quote::bytes(
        sum(
            shared::<RegionHeadImage>()?,
            vector::<[usize; 2]>(16 * usize::BITS as usize)?,
        )?,
        0,
    );
    record(
        raw,
        &[
            "kind_projection",
            "history_kind_projection",
            "partition",
            "history_frontiers",
            "adjacency_kinds",
            "reverse_adjacency_kinds",
        ],
        request,
        |i, value| {
            quote.add(match i {
                0 | 1 => projection(value, request)?,
                2 => {
                    let mut q = Quote::default();
                    record(
                        value,
                        &[
                            "partition_id",
                            "entity_arena",
                            "relation_arena",
                            "adjacency",
                            "reverse_adjacency",
                        ],
                        request,
                        |i, value| {
                            if i != 0 {
                                q.add(if i <= 2 {
                                    arena(value, i == 1, contract, request)?
                                } else {
                                    adjacency(value, request)?
                                })?;
                            }
                            Ok(())
                        },
                    )?;
                    q
                }
                3 => frontiers(value, request)?,
                4 | 5 => kinds(value, request)?,
                _ => return Err(damaged()),
            })
        },
    )?;
    Ok(quote)
}
pub(in crate::storage::residency) fn history(
    raw: Raw<'_>,
    contract: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    work(request, raw.0.len() as u64)?;
    let mut quote = Quote::bytes(shared::<RegionHistoryImage>()?, 0);
    record(
        raw,
        &[
            "kind_projection",
            "adjacency_kinds",
            "reverse_adjacency_kinds",
            "entities",
            "relations",
        ],
        request,
        |i, value| {
            if i == 0 {
                quote.add(projection(value, request)?)?;
            } else if i <= 2 {
                quote.add(kinds(value, request)?)?;
            } else {
                let entity = i == 3;
                let count = sequence(value, request, |row| {
                    record(row, &["prefix", "earlier"], request, |i, value| {
                        if i == 1 {
                            quote.add(aspect_state::histories_element(
                                value, entity, contract, request,
                            )?)?;
                        }
                        Ok(())
                    })
                })?;
                quote.wire = sum(
                    quote.wire,
                    if i == 3 {
                        vector::<RegionHistoryRow<VersionedEntityMetadataImage>>(count)?
                    } else {
                        vector::<RegionHistoryRow<VersionedRelationMetadataImage>>(count)?
                    },
                )?;
            }
            Ok(())
        },
    )?;
    Ok(quote)
}
