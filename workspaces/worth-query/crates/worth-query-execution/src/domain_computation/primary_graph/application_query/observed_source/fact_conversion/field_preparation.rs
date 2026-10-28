use super::*;

impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation::primary_graph::application_query) fn admit_fact_materialization<
        'source,
        'request,
        'authority,
    >(
        &'source self,
        layout: &WorthQueryPrimaryGraphLayout,
        subject: &str,
        control: SourceFactExpansionControl<'request, 'authority>,
    ) -> Result<
        PreparedSourceFactMaterialization<'source, 'request, 'authority, Query>,
        WorthQuerySourceExpectationDenial,
    > {
        self.validate_fact_retention(layout, subject, control)?;
        let footprint = self.source_meaning.footprint();
        let fact_count = source_fact_count(self, subject)?;
        let field_count = footprint.field_count().ok_or_else(|| {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
                subject,
            )
        })?;
        let mut fields = Vec::new();
        fields.try_reserve_exact(field_count).map_err(|_| {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
                subject,
            )
        })?;
        for aspect in footprint.fact_sources().flat_map(|facts| facts.aspects) {
            control.poll(subject)?;
            // Validation above checked the same immutable source borrow and layout.
            fields.push(ValidatedSourceField(aspect));
        }
        Ok(PreparedSourceFactMaterialization {
            source: self,
            fields,
            fact_count,
            control,
        })
    }
}
