use crate::boundary::errors::WorthSignalJsError;
use crate::expression::model::SignalValue;

use super::super::aspects::{checked_grid_cells, checked_packed_capacity};
use super::super::debug::{perf_now_ms, wasm_debug};
use super::super::state::{KeyedEnsureStats, KeyedTarget, PackedFieldReadStats};
use super::super::RuntimeCore;

impl RuntimeCore {
    pub fn read_keyed_value(
        &mut self,
        family_id: &str,
        key: &str,
    ) -> Result<SignalValue, WorthSignalJsError> {
        let id = if self.lock_store()?.recipe_families.contains_key(family_id) {
            self.ensure_recipe_key(family_id, key)?
        } else {
            self.ensure_source_key(family_id, key, None)?
        };
        self.read_value(&id)
    }

    pub fn read_keyed_values(
        &mut self,
        family_id: &str,
        keys: Vec<String>,
    ) -> Result<Vec<SignalValue>, WorthSignalJsError> {
        let mut values = Vec::with_capacity(keys.len());
        for key in keys {
            values.push(self.read_keyed_value(family_id, &key)?);
        }
        Ok(values)
    }

    pub fn read_keyed_values_packed_fields(
        &mut self,
        family_id: &str,
        keys: Vec<String>,
        fields: Vec<String>,
    ) -> Result<Vec<f32>, WorthSignalJsError> {
        let started_at = perf_now_ms();
        let targets = self.ensure_keyed_targets(family_id, &keys)?;
        let mut read_stats = self.bulk_evaluate_targets(&targets)?;
        let mut packed = Vec::with_capacity(keys.len().saturating_mul(fields.len()));
        self.pack_fields_from_targets(&targets, &fields, &mut packed, &mut read_stats)?;
        wasm_debug(format!(
            "[worth-signals-wasm] packed-many:read family={family_id} keys={} elapsed_ms={:.1} runtime_read_ms={:.1} field_extract_ms={:.1} source_reads={} recipe_reads={} recipe_cold_reads={} fields_packed={}",
            read_stats.key_reads,
            perf_now_ms() - started_at,
            read_stats.runtime_read_ms,
            read_stats.field_extract_ms,
            read_stats.source_reads,
            read_stats.recipe_reads,
            read_stats.recipe_cold_reads,
            read_stats.fields_packed
        ));
        Ok(packed)
    }

    pub fn read_keyed_grid_packed_fields(
        &mut self,
        family_id: &str,
        columns: u32,
        rows: u32,
        fields: Vec<String>,
    ) -> Result<Vec<f32>, WorthSignalJsError> {
        let ensure_started_at = perf_now_ms();
        let targets = self.ensure_keyed_grid_targets(family_id, columns, rows)?;
        wasm_debug(format!(
            "[worth-signals-wasm] packed-grid:ensure family={family_id} elapsed_ms={:.1} source_hits={} source_created={} recipe_hits={} recipe_created={}",
            perf_now_ms() - ensure_started_at,
            targets.1.source_hits,
            targets.1.source_created,
            targets.1.recipe_hits,
            targets.1.recipe_created
        ));
        let extract_started_at = perf_now_ms();
        let mut read_stats = self.bulk_evaluate_targets(&targets.0)?;
        let mut packed = Vec::with_capacity(checked_packed_capacity(columns, rows, fields.len())?);
        self.pack_fields_from_targets(&targets.0, &fields, &mut packed, &mut read_stats)?;
        wasm_debug(format!(
            "[worth-signals-wasm] packed-grid:extract family={family_id} elapsed_ms={:.1} runtime_read_ms={:.1} field_extract_ms={:.1} keys={} source_reads={} recipe_reads={} recipe_cold_reads={} fields_packed={}",
            perf_now_ms() - extract_started_at,
            read_stats.runtime_read_ms,
            read_stats.field_extract_ms,
            read_stats.key_reads,
            read_stats.source_reads,
            read_stats.recipe_reads,
            read_stats.recipe_cold_reads,
            read_stats.fields_packed
        ));
        Ok(packed)
    }

    pub fn read_keyed_rect_packed_fields(
        &mut self,
        family_id: &str,
        columns: u32,
        rows: u32,
        row: u32,
        start_column: u32,
        width: u32,
        height: u32,
        fields: Vec<String>,
    ) -> Result<Vec<f32>, WorthSignalJsError> {
        if row >= rows || start_column >= columns {
            return Ok(Vec::new());
        }
        let ensure_started_at = perf_now_ms();
        let targets = self.ensure_keyed_rect_targets(
            family_id,
            columns,
            rows,
            row,
            start_column,
            width,
            height,
        )?;
        wasm_debug(format!(
            "[worth-signals-wasm] packed-rect:ensure family={family_id} row={} start={} size={}x{} elapsed_ms={:.1} source_hits={} source_created={} recipe_hits={} recipe_created={}",
            row,
            start_column,
            width,
            height,
            perf_now_ms() - ensure_started_at,
            targets.1.source_hits,
            targets.1.source_created,
            targets.1.recipe_hits,
            targets.1.recipe_created
        ));
        let clamped_width = width.min(columns.saturating_sub(start_column));
        let clamped_height = height.min(rows.saturating_sub(row));
        let extract_started_at = perf_now_ms();
        let mut read_stats = self.bulk_evaluate_targets(&targets.0)?;
        let mut packed = Vec::with_capacity(checked_packed_capacity(
            clamped_width,
            clamped_height,
            fields.len(),
        )?);
        self.pack_fields_from_targets(&targets.0, &fields, &mut packed, &mut read_stats)?;
        wasm_debug(format!(
            "[worth-signals-wasm] packed-rect:extract family={family_id} row={} start={} size={}x{} elapsed_ms={:.1} runtime_read_ms={:.1} field_extract_ms={:.1} keys={} source_reads={} recipe_reads={} recipe_cold_reads={} fields_packed={}",
            row,
            start_column,
            clamped_width,
            clamped_height,
            perf_now_ms() - extract_started_at,
            read_stats.runtime_read_ms,
            read_stats.field_extract_ms,
            read_stats.key_reads,
            read_stats.source_reads,
            read_stats.recipe_reads,
            read_stats.recipe_cold_reads,
            read_stats.fields_packed
        ));
        Ok(packed)
    }

    pub fn prewarm_keyed_grid(
        &mut self,
        family_id: &str,
        columns: u32,
        rows: u32,
    ) -> Result<(), WorthSignalJsError> {
        let ensure_started_at = perf_now_ms();
        let targets = self.ensure_keyed_grid_targets(family_id, columns, rows)?;
        let evaluate_started_at = perf_now_ms();
        let read_stats = self.bulk_evaluate_targets(&targets.0)?;
        wasm_debug(format!(
            "[worth-signals-wasm] keyed-grid:prewarm family={family_id} size={}x{} ensure_ms={:.1} evaluate_ms={:.1} source_hits={} source_created={} recipe_hits={} recipe_created={} source_reads={} recipe_reads={} recipe_cold_reads={} runtime_read_ms={:.1}",
            columns,
            rows,
            perf_now_ms() - ensure_started_at,
            perf_now_ms() - evaluate_started_at,
            targets.1.source_hits,
            targets.1.source_created,
            targets.1.recipe_hits,
            targets.1.recipe_created,
            read_stats.source_reads,
            read_stats.recipe_reads,
            read_stats.recipe_cold_reads,
            read_stats.runtime_read_ms
        ));
        Ok(())
    }

    fn pack_fields_from_targets(
        &mut self,
        targets: &[KeyedTarget],
        fields: &[String],
        packed: &mut Vec<f32>,
        stats: &mut PackedFieldReadStats,
    ) -> Result<(), WorthSignalJsError> {
        let store = self.lock_store()?;
        for target in targets {
            let object = match store.read_value(&target.id) {
                Some(SignalValue::Object(entries)) => entries,
                Some(other) => {
                    return Err(WorthSignalJsError::invalid_input(format!(
                        "target `{}` is not an object value: {other:?}",
                        target.id
                    )));
                }
                None => {
                    return Err(WorthSignalJsError::invalid_input(format!(
                        "missing stored value for `{}`",
                        target.id
                    )));
                }
            };
            let extract_started_at = perf_now_ms();
            for field in fields {
                let Some((_, value)) = object.iter().find(|(candidate, _)| candidate == field)
                else {
                    return Err(WorthSignalJsError::invalid_input(format!(
                        "target `{}` is missing numeric field `{field}`",
                        target.id
                    )));
                };
                match value {
                    SignalValue::Number(number) => packed.push(*number as f32),
                    other => {
                        return Err(WorthSignalJsError::invalid_input(format!(
                            "target `{}` field `{field}` is not numeric: {other:?}",
                            target.id
                        )));
                    }
                }
            }
            stats.field_extract_ms += perf_now_ms() - extract_started_at;
            stats.fields_packed = stats.fields_packed.saturating_add(fields.len());
        }
        Ok(())
    }

    fn bulk_evaluate_targets(
        &mut self,
        targets: &[KeyedTarget],
    ) -> Result<PackedFieldReadStats, WorthSignalJsError> {
        let mut stats = PackedFieldReadStats::default();
        if targets.is_empty() {
            return Ok(stats);
        }
        {
            let store = self.lock_store()?;
            for target in targets {
                if store.sources.contains_key(&target.id) {
                    stats.source_reads = stats.source_reads.saturating_add(1);
                } else if let Some(recipe) = store.recipes.get(&target.id) {
                    stats.recipe_reads = stats.recipe_reads.saturating_add(1);
                    if !recipe.initialized {
                        stats.recipe_cold_reads = stats.recipe_cold_reads.saturating_add(1);
                    }
                }
            }
        }
        stats.key_reads = targets.len();
        let read_started_at = perf_now_ms();
        let evaluator = self.evaluator();
        let nodes = targets.iter().map(|target| target.node).collect::<Vec<_>>();
        let _ = self
            .runtime
            .targets(nodes)
            .read_many(&self.store, &evaluator)
            .map_err(WorthSignalJsError::from)?;
        stats.runtime_read_ms = perf_now_ms() - read_started_at;
        Ok(stats)
    }

    fn ensure_keyed_targets(
        &mut self,
        family_id: &str,
        keys: &[String],
    ) -> Result<Vec<KeyedTarget>, WorthSignalJsError> {
        let mut stats = KeyedEnsureStats::default();
        let mut targets = Vec::with_capacity(keys.len());
        for key in keys {
            let id = self.ensure_keyed_entry(family_id, key, &mut stats)?;
            let node = self.node_for_id(&id)?;
            targets.push(KeyedTarget { id, node });
        }
        Ok(targets)
    }

    fn ensure_keyed_grid_targets(
        &mut self,
        family_id: &str,
        columns: u32,
        rows: u32,
    ) -> Result<(Vec<KeyedTarget>, KeyedEnsureStats), WorthSignalJsError> {
        let mut stats = KeyedEnsureStats::default();
        let mut targets = Vec::with_capacity(checked_grid_cells(columns, rows)?);
        for row in 0..rows {
            for column in 0..columns {
                let key = format!("tile-{column}-{row}");
                let id = self.ensure_keyed_entry(family_id, &key, &mut stats)?;
                let node = self.node_for_id(&id)?;
                targets.push(KeyedTarget { id, node });
            }
        }
        Ok((targets, stats))
    }

    fn ensure_keyed_rect_targets(
        &mut self,
        family_id: &str,
        columns: u32,
        rows: u32,
        row: u32,
        start_column: u32,
        width: u32,
        height: u32,
    ) -> Result<(Vec<KeyedTarget>, KeyedEnsureStats), WorthSignalJsError> {
        if row >= rows || start_column >= columns {
            return Ok((Vec::new(), KeyedEnsureStats::default()));
        }
        let clamped_width = width.min(columns.saturating_sub(start_column));
        let clamped_height = height.min(rows.saturating_sub(row));
        let mut stats = KeyedEnsureStats::default();
        let mut targets = Vec::with_capacity(checked_grid_cells(clamped_width, clamped_height)?);
        for row_offset in 0..clamped_height {
            let current_row = row + row_offset;
            for column_offset in 0..clamped_width {
                let current_column = start_column + column_offset;
                let key = format!("tile-{current_column}-{current_row}");
                let id = self.ensure_keyed_entry(family_id, &key, &mut stats)?;
                let node = self.node_for_id(&id)?;
                targets.push(KeyedTarget { id, node });
            }
        }
        Ok((targets, stats))
    }
}
