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
        self.validate_completeness(subject)?;
        let footprint = self.source_meaning.footprint();
        let selected = footprint.root_selection.as_deref();
        let selected_count = selected.map_or(Some(0), |selection| {
            selection
                .entities
                .len()
                .checked_add(selection.aspects.len())
                .and_then(|count| count.checked_add(selection.adjacencies.len()))
        });
        let count = footprint
            .entities
            .len()
            .checked_add(footprint.aspects.len())
            .and_then(|count| count.checked_add(footprint.adjacencies.len()))
            .and_then(|count| count.checked_add(selected_count?))
            .ok_or_else(|| {
                WorthQuerySourceExpectationDenial::new(
                    WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
                    subject,
                )
            })?;
        let mut fields = Vec::new();
        let field_count = footprint
            .aspects
            .len()
            .checked_add(selected.map_or(0, |selection| selection.aspects.len()))
            .ok_or_else(|| {
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
        for aspect in footprint.aspects.iter().chain(
            selected
                .into_iter()
                .flat_map(|selection| &selection.aspects),
        ) {
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
