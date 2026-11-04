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
        PreparedSourceFactMaterialization<'source, 'request, 'authority>,
        WorthQuerySourceExpectationDenial,
    > {
        self.validate_fact_retention(layout, subject, control)?;
        prepare_fields(
            self.source_meaning.footprint(),
            &self.query_identifier,
            self.invocation_managed_source()?,
            self.invocation_source_authorization()?,
            control,
        )
    }
}

pub(super) fn prepare_fields<'source, 'request, 'authority>(
    footprint: &'source WorthQueryObservedSourceFootprint,
    subject: &'source str,
    managed: Option<
        &'source std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead,
        >,
    >,
    authorization: Option<
        &'source std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization,
        >,
    >,
    control: SourceFactExpansionControl<'request, 'authority>,
) -> Result<
    PreparedSourceFactMaterialization<'source, 'request, 'authority>,
    WorthQuerySourceExpectationDenial,
> {
    let fact_count = footprint.fact_count().ok_or_else(|| {
        WorthQuerySourceExpectationDenial::new(
            WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded,
            subject,
        )
    })?;
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
        footprint,
        subject,
        managed,
        authorization,
        fields,
        fact_count,
        control,
    })
}
