use std::collections::BTreeMap;
use std::sync::Arc;

use crate::boundary::errors::WorthSignalJsError;
use crate::expression::model::SignalValue;

use super::super::super::aspects::{
    aspect_mask_from_list, checked_grid_cells, defaulted_produced_aspects, initial_aspect_version,
};
use super::super::super::debug::wasm_debug;
use super::super::super::keyed_families::{object_number_field, parse_tile_key};
use super::super::super::state::{CatalogEntry, DenseGridFamily, StoredSource};
use super::super::super::{RuntimeCore, DEFAULT_ASPECT};
use crate::recipe::model::KeyedSetValue;

impl RuntimeCore {
    pub(in crate::runtime::core) fn ensure_dense_rgba_grid(
        &mut self,
        family_id: &str,
        width: u32,
        height: u32,
    ) -> Result<Arc<DenseGridFamily>, WorthSignalJsError> {
        if let Some(existing) = self.dense_grids.get(family_id) {
            if existing.width != width || existing.height != height {
                return Err(WorthSignalJsError::invalid_input(format!(
                    "dense grid family `{family_id}` was initialized as {}x{} and cannot become {width}x{height}",
                    existing.width, existing.height
                )));
            }
            return Ok(existing.clone());
        }

        let started_at = self.runtime.time_sampler().monotonic();
        wasm_debug(format!(
            "[worth-signals-wasm] dense-grid:init family={family_id} size={}x{} cells={}",
            width,
            height,
            checked_grid_cells(width, height)?
        ));

        let initial = {
            let store = self.lock_store()?;
            let family = store.source_families.get(family_id).ok_or_else(|| {
                WorthSignalJsError::invalid_input(format!("unknown source family `{family_id}`"))
            })?;
            (
                family.spec.initial.clone(),
                defaulted_produced_aspects(family.spec.produces_aspects.as_deref()),
            )
        };
        let (initial, produced_aspects) = initial;

        let grid_cells = checked_grid_cells(width, height)?;
        let mut ids = Vec::with_capacity(grid_cells);
        let mut nodes = Vec::with_capacity(grid_cells);
        let mut key_to_index = BTreeMap::new();
        let mut pending_sources = Vec::with_capacity(grid_cells);

        for index in 0..grid_cells {
            let x = index % (width as usize);
            let y = index / (width as usize);
            let key = format!("{x},{y}");
            let id = crate::runtime::core::keyed_families::composite_keyed_id(family_id, &key);
            if let Some(existing) = self.catalog.get(&id) {
                ids.push(id.clone());
                nodes.push(existing.node);
                key_to_index.insert(key, index);
                continue;
            }

            let node = self
                .runtime
                .graph_mut()
                .node()
                .produces_aspects(aspect_mask_from_list(&produced_aspects))
                .build();
            self.catalog.insert(
                id.clone(),
                CatalogEntry {
                    node,
                    produced_aspects: produced_aspects.clone(),
                },
            );
            self.nodes_by_id.insert(node, id.clone());
            ids.push(id.clone());
            nodes.push(node);
            key_to_index.insert(key, index);
            pending_sources.push((id, initial.clone()));

            if index > 0 && index % 10_000 == 0 {
                wasm_debug(format!(
                    "[worth-signals-wasm] dense-grid:init progress family={family_id} built={index}"
                ));
            }
        }

        if !pending_sources.is_empty() {
            let mut store = self.lock_store()?;
            for (id, value) in pending_sources {
                store.sources.insert(
                    id,
                    StoredSource {
                        value,
                        version: initial_aspect_version(&produced_aspects),
                    },
                );
            }
        }

        let family = Arc::new(DenseGridFamily {
            width,
            height,
            ids,
            nodes,
            key_to_index,
            produced_aspects,
        });
        self.dense_grids
            .insert(family_id.to_owned(), family.clone());
        wasm_debug(format!(
            "[worth-signals-wasm] dense-grid:ready family={family_id} elapsed_ms={:.1}",
            self.perf_elapsed_ms(started_at)
        ));
        Ok(family)
    }

    pub fn seed_keyed_grid_coords(
        &mut self,
        family_id: &str,
        width: u32,
        height: u32,
    ) -> Result<(), WorthSignalJsError> {
        if let Some(existing) = self.dense_grids.get(family_id) {
            if existing.width == width && existing.height == height {
                wasm_debug(format!(
                    "[worth-signals-wasm] dense-grid:coords family={family_id} size={}x{} reused",
                    width, height
                ));
                return Ok(());
            }
            return Err(WorthSignalJsError::invalid_input(format!(
                "dense grid family `{family_id}` was initialized as {}x{} and cannot become {width}x{height}",
                existing.width, existing.height
            )));
        }

        let started_at = self.runtime.time_sampler().monotonic();
        let grid_cells = checked_grid_cells(width, height)?;
        let mut ids = Vec::with_capacity(grid_cells);
        let mut nodes = Vec::with_capacity(grid_cells);
        let mut key_to_index = BTreeMap::new();
        let mut pending_sources = Vec::with_capacity(grid_cells);

        for row in 0..height {
            for column in 0..width {
                let index = (row as usize) * (width as usize) + (column as usize);
                let key = format!("tile-{column}-{row}");
                let id = crate::runtime::core::keyed_families::composite_keyed_id(family_id, &key);
                if let Some(existing) = self.catalog.get(&id) {
                    ids.push(id.clone());
                    nodes.push(existing.node);
                    key_to_index.insert(key, index);
                    continue;
                }

                let node = self.runtime.graph_mut().node().build();
                self.catalog.insert(
                    id.clone(),
                    CatalogEntry {
                        node,
                        produced_aspects: vec![DEFAULT_ASPECT],
                    },
                );
                self.nodes_by_id.insert(node, id.clone());
                ids.push(id.clone());
                nodes.push(node);
                key_to_index.insert(key, index);
                pending_sources.push((
                    id,
                    SignalValue::Object(vec![
                        ("column".to_owned(), SignalValue::Number(column as f64)),
                        ("row".to_owned(), SignalValue::Number(row as f64)),
                    ]),
                ));
            }
        }

        if !pending_sources.is_empty() {
            let mut store = self.lock_store()?;
            for (id, value) in pending_sources {
                store.sources.insert(
                    id,
                    StoredSource {
                        value,
                        version: initial_aspect_version(&[DEFAULT_ASPECT]),
                    },
                );
            }
        }

        self.dense_grids.insert(
            family_id.to_owned(),
            Arc::new(DenseGridFamily {
                width,
                height,
                ids,
                nodes,
                key_to_index,
                produced_aspects: vec![DEFAULT_ASPECT],
            }),
        );
        wasm_debug(format!(
            "[worth-signals-wasm] dense-grid:coords family={family_id} size={}x{} elapsed_ms={:.1}",
            width,
            height,
            self.perf_elapsed_ms(started_at)
        ));
        Ok(())
    }

    pub(in crate::runtime::core) fn try_fast_seed_keyed_grid_coords(
        &mut self,
        family_id: &str,
        values: &[KeyedSetValue],
    ) -> Result<bool, WorthSignalJsError> {
        if family_id != "renderTileCoord" || values.is_empty() {
            return Ok(false);
        }

        let mut max_column = 0u32;
        let mut max_row = 0u32;

        for entry in values {
            let Some((column, row)) = parse_tile_key(&entry.key) else {
                return Ok(false);
            };
            let SignalValue::Object(fields) = &entry.value else {
                return Ok(false);
            };
            let Some(value_column) = object_number_field(fields, "column") else {
                return Ok(false);
            };
            let Some(value_row) = object_number_field(fields, "row") else {
                return Ok(false);
            };
            if value_column != column as f64 || value_row != row as f64 {
                return Ok(false);
            }
            max_column = max_column.max(column);
            max_row = max_row.max(row);
        }

        let width = max_column.saturating_add(1);
        let height = max_row.saturating_add(1);
        if checked_grid_cells(width, height)? != values.len() {
            return Ok(false);
        }

        wasm_debug(format!(
            "[worth-signals-wasm] keyed-set:coords-fast-path family={family_id} size={}x{} entries={}",
            width,
            height,
            values.len()
        ));
        self.seed_keyed_grid_coords(family_id, width, height)?;
        Ok(true)
    }
}
