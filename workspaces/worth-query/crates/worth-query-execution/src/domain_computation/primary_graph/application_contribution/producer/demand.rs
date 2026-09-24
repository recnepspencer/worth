use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryInstalledApplicationProducerRegistry, WorthQueryProducerApplicability,
    WorthQueryProducerLifecyclePosture, WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::WorthQueryObservedSource;

type FamilySource<Schema, Family> = <Family as WorthQueryProducerOutputFamily<Schema>>::Source;
type FamilySourceValue<Schema, Family> =
    <<FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value;
type FamilySourceQuery<Schema, Family> =
    <FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query;

mod bridge_denial;
mod checkpoint_delivery;
mod disclosure;
mod progression;
mod readiness;
mod scheduling_progression;
mod selection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputDemandDenialKind {
    ForeignSource,
    MissingApplicableProducer,
    AmbiguousApplicableProducer,
    ProducerUnavailable,
    ProductSelection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    SchedulingRejected,
    SchedulingDeferred,
    PublicationStale,
    NoEffect,
    Superseded,
    Cancelled,
    TimedOut,
    WorkBudgetExceeded,
    RetentionBudgetExceeded,
    PublicationCapacityExceeded,
    ForeignDemand,
    ForeignSettlement,
    IncompleteDependencyCoverage,
    RetainedBasisUnavailable,
    Closed,
    DuplicatePerformedSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputDemandRecoveryPosture {
    Retryable,
    Terminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputDemandDenial {
    kind: WorthQueryOutputDemandDenialKind,
    subject: String,
    pub(in crate::domain_computation::primary_graph) recovery_posture:
        WorthQueryOutputDemandRecoveryPosture,
}

impl WorthQueryOutputDemandDenial {
    pub const fn kind(&self) -> WorthQueryOutputDemandDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub const fn recovery_posture(&self) -> WorthQueryOutputDemandRecoveryPosture {
        self.recovery_posture
    }

    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryOutputDemandDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
            recovery_posture: kind.default_recovery_posture(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn product_selection(
        denial: crate::basis::WorthQueryProductBranchAdmissionDenial,
        subject: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryOutputDemandDenialKind::ProductSelection(denial),
            subject,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn with_recovery_posture(
        mut self,
        recovery_posture: WorthQueryOutputDemandRecoveryPosture,
    ) -> Self {
        self.recovery_posture = recovery_posture;
        self
    }
}

impl WorthQueryOutputDemandDenialKind {
    const fn default_recovery_posture(self) -> WorthQueryOutputDemandRecoveryPosture {
        use WorthQueryOutputDemandRecoveryPosture::{Retryable, Terminal};
        match self {
            Self::ProductSelection(denial) => {
                if denial.is_transient() {
                    Retryable
                } else {
                    Terminal
                }
            }
            Self::SchedulingDeferred => Retryable,
            Self::ForeignSource
            | Self::MissingApplicableProducer
            | Self::AmbiguousApplicableProducer
            | Self::ProducerUnavailable
            | Self::SchedulingRejected
            | Self::PublicationStale
            | Self::NoEffect
            | Self::Superseded
            | Self::Cancelled
            | Self::TimedOut
            | Self::WorkBudgetExceeded
            | Self::RetentionBudgetExceeded
            | Self::PublicationCapacityExceeded
            | Self::ForeignDemand
            | Self::ForeignSettlement
            | Self::IncompleteDependencyCoverage
            | Self::RetainedBasisUnavailable
            | Self::Closed
            | Self::DuplicatePerformedSource => Terminal,
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
    pub(super) exact_retained_output: bool,
    pub(super) retained_resources: Option<super::WorthQueryProducerDemandResources>,
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
    observed_source: WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    currentness_work_limit: std::num::NonZeroUsize,
    maximum_retained_bytes: usize,
    resources: Option<super::WorthQueryProducerDemandResources>,
    resources_validated: bool,
    producer_contacts_in_this_demand: usize,
    admission_kind: super::super::super::application_output_demand::DemandAdmissionKind,
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
                && entry.declaration.output_family_type == std::any::TypeId::of::<Family>()
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
            exact_retained_output: false,
            retained_resources: None,
        })
    }
}
