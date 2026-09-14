use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryInstalledApplicationProducerRegistry, WorthQueryProducerApplicability,
    WorthQueryProducerLifecyclePosture, WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisSelectionIdentity, WorthQueryObservedSource,
    WorthQueryPrimaryGraphApplicationRuntime,
};

type FamilySource<Schema, Family> = <Family as WorthQueryProducerOutputFamily<Schema>>::Source;
type FamilySourceValue<Schema, Family> =
    <<FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value;
type FamilySourceQuery<Schema, Family> =
    <FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query;

mod disclosure;
mod progression;
mod readiness_delivery;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputDemandDenialKind {
    ForeignSource,
    MissingApplicableProducer,
    AmbiguousApplicableProducer,
    ProducerUnavailable,
    SchedulingRejected,
    Superseded,
    Cancelled,
    TimedOut,
    WorkBudgetExceeded,
    RetentionBudgetExceeded,
    PublicationCapacityExceeded,
    ForeignDemand,
    ForeignSettlement,
    RetainedBasisUnavailable,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandDenial {
    kind: WorthQueryOutputDemandDenialKind,
    subject: String,
}

impl WorthQueryOutputDemandDenial {
    pub const fn kind(&self) -> WorthQueryOutputDemandDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryOutputDemandDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl std::fmt::Display for WorthQueryOutputDemandDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "output demand denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryOutputDemandDenial {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySelectedApplicationProducer {
    pub(super) identity: String,
    pub(super) applicability: WorthQueryProducerApplicability,
}

/// Runtime-affine admission for one exact source occurrence and installed producer.
pub struct WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    runtime_authority: u64,
    schema_binding: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    selected: WorthQuerySelectedApplicationProducer,
    source: FamilySourceValue<Schema, Family>,
    observed_source: WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    interest:
        Option<super::super::super::application_output_demand::WorthQueryOutputDemandInterest>,
}

impl<Schema, Family> WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    pub fn observed_source(&self) -> &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>> {
        &self.observed_source
    }

    pub fn matches_observed_source(
        &self,
        source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    ) -> bool {
        self.observed_source.idempotency_identity() == source.idempotency_identity()
    }

    pub fn notifications(
        &self,
    ) -> Result<
        super::super::super::application_output_demand::WorthQueryOutputDemandNotifications,
        WorthQueryOutputDemandDenial,
    > {
        self.interest
            .as_ref()
            .map(|interest| interest.notifications())
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Closed,
                    Family::IDENTITY,
                )
            })
    }

    pub fn close(&mut self) {
        self.interest.take();
    }
}

impl<Schema, Family> Drop for WorthQueryAdmittedOutputDemand<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    fn drop(&mut self) {
        self.interest.take();
    }
}

pub enum WorthQueryOutputDemandAdvance {
    Pending,
    Settled(
        std::sync::Arc<
            super::super::super::application_output_demand::WorthQueryOutputDemandSettlement,
        >,
    ),
}

impl WorthQuerySelectedApplicationProducer {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn applicability(&self) -> WorthQueryProducerApplicability {
        self.applicability
    }
}

impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn select<Family>(
        &self,
        applicability: WorthQueryProducerApplicability,
    ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let mut matching = self.entries.values().filter(|entry| {
            entry.declaration.output_family == Family::IDENTITY
                && entry.declaration.applicability.contains(&applicability)
        });
        let selected = matching.next().ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::MissingApplicableProducer,
                Family::IDENTITY,
            )
        })?;
        if matching.next().is_some() {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::AmbiguousApplicableProducer,
                Family::IDENTITY,
            ));
        }
        Ok(WorthQuerySelectedApplicationProducer {
            identity: selected.declaration.identity.clone(),
            applicability,
        })
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn select_output_producer<Family>(
        &self,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,
    ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        if source.runtime_authority != self.runtime.authority_identity().as_u64()
            || source.schema_binding != self.installed_schema.binding_identity()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            ));
        }
        let WorthQueryApplicationBasisSelectionIdentity::Product(observation) = &source.selection
        else {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            ));
        };
        let output_bindings = self.installed_producers.family_output_bindings::<Family>();
        let source_posture = self
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .source_posture_for_any_output_binding(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(source.footprint.root),
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                &output_bindings,
                source.idempotency_identity(),
            );
        let lifecycle = match source_posture {
            super::super::super::output_lineage::WorthQueryOutputSourcePosture::Exact(binding) => {
                return self.installed_producers.select_exact::<Family>(binding)
            }
            super::super::super::output_lineage::WorthQueryOutputSourcePosture::Absent => {
                WorthQueryProducerLifecyclePosture::Initial
            }
            super::super::super::output_lineage::WorthQueryOutputSourcePosture::Drifted => {
                WorthQueryProducerLifecyclePosture::Preserve
            }
        };
        self.installed_producers
            .select::<Family>(WorthQueryProducerApplicability::new(
                profile_kind,
                lifecycle,
            ))
    }
}
