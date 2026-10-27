//! Guard positions always refer to the original authored path, in either direction.
use super::*;
use worth_query_installation::facade::WorthQueryInstalledRootPathGuard;
pub(super) fn apply<E>(
    basis: &RootPathReadBasis<'_, '_, '_>,
    path: &WorthQueryInstalledRootPath,
    after_step: usize,
    frontier: &mut Frontier,
    ports: &mut RootPathReadPorts<'_, E>,
    set_source: &mut Option<SourceFactsBuilder>,
) -> Result<(), RootPathReadDenial<E>> {
    for guard in path
        .guards()
        .iter()
        .filter(|guard| guard.after_step() == after_step)
    {
        if frontier.is_empty() {
            break;
        }
        let candidates = frontier.keys().copied().collect::<BTreeSet<_>>();
        let RootPathReadPorts {
            checkpoint,
            read_guard_with_sources,
            observation,
        } = ports;
        let mut prepared = false;
        let mut prepare = || {
            if prepared {
                return Err(RootPathReadDenial::traversal(guard.field().as_str()));
            }
            prepare_sources(basis, guard, frontier, set_source, observation)?;
            checkpoint(guard.field().as_str()).map_err(RootPathReadDenial::Adapter)?;
            prepared = true;
            Ok(())
        };
        let read = read_guard_with_sources(guard, &candidates, &mut prepare)?;
        if !prepared {
            return Err(RootPathReadDenial::traversal(guard.field().as_str()));
        }
        ports.notify(RootPathReadObservation::Predicate {
            records: read.entity_records_examined(),
            matches: read.matching_entity_ids_reserved(),
            fields: read.field_reads(),
            subject: guard.field().as_str(),
        })?;
        let matching = read.into_matching_entity_ids();
        frontier.retain(|entity, _| matching.contains(entity));
    }
    Ok(())
}
fn prepare_sources<E>(
    basis: &RootPathReadBasis<'_, '_, '_>,
    guard: &WorthQueryInstalledRootPathGuard,
    frontier: &mut Frontier,
    set_source: &mut Option<SourceFactsBuilder>,
    observation: &mut dyn FnMut(RootPathReadObservation<'_>) -> Result<(), E>,
) -> Result<(), RootPathReadDenial<E>> {
    for (&entity, source) in frontier {
        if !source.has_field(entity, guard.aspect(), guard.field()) {
            observation(RootPathReadObservation::SourceRead {
                subject: guard.entity(),
            })
            .map_err(RootPathReadDenial::Adapter)?;
        }
        let observed = source
            .observe_guard_field(
                basis.execution,
                &basis.projection,
                basis.graph,
                entity,
                guard.entity(),
                guard.aspect(),
                guard.field(),
            )
            .map_err(|cause| RootPathReadDenial::source(guard.entity(), cause))?;
        if let Some(set_source) = &mut *set_source
            && set_source.record_field(observed)
        {
            observation(RootPathReadObservation::SourceCopy {
                units: 1,
                subject: guard.field().as_str(),
            })
            .map_err(RootPathReadDenial::Adapter)?;
        }
    }
    Ok(())
}
