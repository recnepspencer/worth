//! The same installed forward predicate judges actual historical affected rows.
use super::*;
use crate::domain_computation::primary_graph::access_strategy::ordered_storage;
use worth_foundational::facade::AspectValue;
use worth_relational::facade::{
    history::RelationalAuthoredManagedSourceMaterial, identity::EntityId,
};
use crate::domain_computation::primary_graph::vocabulary::application_query::source_rows::WorthQueryManagedIndexSourceSlot;
use crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop;
use crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial;

pub(super) fn relevant<S, I, P>(
    installed: &TypedIndexProvider<S, I, P>,
    execution: &ExecutionRequest<'_, '_>,
    before: source_rows::SourceImageView<'_>,
    after: source_rows::SourceImageView<'_>,
    raw_probe: &AspectValue,
    material: &RelationalAuthoredManagedSourceMaterial,
) -> Result<bool, WorthQueryManagedIndexWorkDenial>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    decoded_provider(execution, before.bindings, |control, session| {
        let probe = session
            .decode_managed_probe::<I::ProbeBinding>(raw_probe)
            .ok_or_else(|| {
                control.refuse(WorthQueryManagedIndexStop::ValueDecode(
                    session
                        .denial()
                        .expect("failed historical probe latches its refusal"),
                ))
            })?;
        provider_result(
            execution,
            control,
            installed.provider.validate_probe(&probe, control),
        )?;
        for image in [before, after] {
            let source_rows::SourceImageRows::Maintained(rows) = image.rows else {
                return Err(control
                    .refuse(WorthQueryManagedIndexStop::ForeignCoverage)
                    .into());
            };
            let visits = u64::try_from(material.row_count())
                .map_err(|_| control.refuse(WorthQueryManagedIndexStop::CapacityOverflow))?;
            control.checkpoint(visits)?;
            let source =
                WorthQueryManagedIndexSourceRows::<S, I>::new(image, session, control, *execution);
            for entity in material.rows() {
                control.poll()?;
                ordered_storage::lookup(control, rows.len(), size_of::<EntityId>())?;
                if !rows.contains(entity) {
                    continue;
                }
                // get validates the exact owner/full-generation slot, then
                // resolves its real body: both immutable descents are paid.
                for _ in 0..2 {
                    ordered_storage::lookup(control, rows.len(), size_of::<EntityId>())?;
                }
                let row = source.get(WorthQueryManagedIndexSourceSlot {
                    owner: image.owner,
                    ordinal: 0,
                    entity: Some(entity),
                })?;
                if provider_result(
                    execution,
                    control,
                    installed.provider.matches_source(row, &probe, control),
                )? {
                    return Ok(true);
                }
            }
            source.check()?;
        }
        Ok(false)
    })
}
