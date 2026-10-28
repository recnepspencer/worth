use super::*;

impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation::primary_graph::application_query) fn admit_fact_materialization(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        subject: &str,
    ) -> Result<
        super::PreparedSourceFactMaterialization<'_, Query>,
        WorthQuerySourceExpectationDenial,
    > {
        self.require_supported_fact_expansion(subject)?;
        self.validate_completeness(subject)?;
        let footprint = self.source_meaning.footprint();
        footprint.validate_selections(layout, None)?;
        let count = super::source_fact_count(self, subject)?;
        let mut fields = Vec::new();
        let field_count = footprint.field_count().ok_or_else(|| {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
                subject,
            )
        })?;
        fields.try_reserve_exact(field_count).map_err(|_| {
            WorthQuerySourceExpectationDenial::new(
                WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
                subject,
            )
        })?;
        for aspect in footprint.fact_sources().flat_map(|facts| facts.aspects) {
            let contract = layout
                .aspect_contract(&aspect.entity_name, &aspect.aspect)
                .ok_or_else(|| {
                    WorthQuerySourceExpectationDenial::new(
                        WorthQuerySourceExpectationDenialKind::SourceContractMismatch,
                        subject,
                    )
                })?;
            fields.push(super::ValidatedSourceField::resolve(aspect, contract)?);
        }
        Ok(super::PreparedSourceFactMaterialization {
            source: self,
            fields,
            fact_count: count,
        })
    }
}
