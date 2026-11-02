//! Domain exact-name predicate; candidate prefix meaning remains unchanged.
use super::*;
impl WorthQueryManagedIndexRefinementProvider<Catalog, PrefixIndex> for Provider {
    const REFINEMENT_IDENTITY: &'static str = "managed-catalog.exact-name";
    const REFINEMENT_VERSION: u32 = 1;
    fn validate_refinement_probe(
        &self,
        probe: &query::Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        control.checkpoint(probe.prefix.len() as u64 + 1)?;
        #[cfg(test)]
        if self.0.reject_refinement.load(Ordering::SeqCst) {
            return Err(Stop::Domain(PrefixIndexDenial::InvalidTopology));
        }
        Ok(())
    }
    fn refine_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, Catalog, PrefixIndex>,
        probe: &query::Probe,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        control.checkpoint(probe.prefix.len() as u64 + 1)?;
        let matches = row
            .with_text(query::name(), |name| name == probe.prefix)
            .map_err(text_denial)?;
        #[cfg(test)]
        {
            let observe = self.0.after_refinement.lock().unwrap().take();
            if let Some(observe) = observe {
                observe();
            }
        }
        Ok(matches)
    }
}
