use worth_query_installation::facade::ApplicationSchema;

use std::any::TypeId;

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

mod bridge_denial;
mod checkpoint_delivery;
mod disclosure;
mod progression;
mod readiness;
mod scheduling_progression;

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
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(source.source_root());
        let mut source_posture = self
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .source_posture_for_any_output_binding(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                &output_bindings,
                source.idempotency_identity(),
                source.checkpoint_identity(),
            );
        if source_posture
            == super::super::super::output_lineage::WorthQueryOutputSourcePosture::Absent
        {
            source_posture = recovered_source_posture(
                &self.recovered_outputs,
                scope,
                source.partition_identity(),
                source.checkpoint_identity(),
                &output_bindings,
            );
        }
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

fn recovered_source_posture(
    recovered_outputs: &[super::super::super::application_output_demand::WorthQueryReadmittedAcceptedOutput],
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    source_partition: [u8; 32],
    checkpoint_identity: crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity,
    output_bindings: &[TypeId],
) -> super::super::super::output_lineage::WorthQueryOutputSourcePosture {
    let mut retained_output = false;
    for recovered in recovered_outputs {
        let checkpoint = &recovered.checkpoint;
        let Some(binding) = recovered.correspondence.binding_type() else {
            continue;
        };
        if checkpoint.scope != scope
            || checkpoint.source_partition != source_partition
            || !output_bindings.contains(&binding)
        {
            continue;
        }
        if checkpoint.source == checkpoint_identity.bytes() {
            return super::super::super::output_lineage::WorthQueryOutputSourcePosture::Exact(
                binding,
            );
        }
        retained_output = true;
    }
    if retained_output {
        super::super::super::output_lineage::WorthQueryOutputSourcePosture::Drifted
    } else {
        super::super::super::output_lineage::WorthQueryOutputSourcePosture::Absent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::domain_computation::primary_graph::{application_output_demand, output_lineage};

    struct OutputBinding;

    #[test]
    fn recovered_output_selects_exact_then_preserve_before_lineage_is_lazily_restored() {
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                1,
            ),
        );
        let partition = [0x22; 32];
        let checkpoint_identity = crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new([0x33; 32]);
        let binding = TypeId::of::<OutputBinding>();
        let recovered = application_output_demand::WorthQueryReadmittedAcceptedOutput {
            checkpoint: application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity {
                producer: "producer".to_owned(),
                source: checkpoint_identity.bytes(),
                scope,
                source_partition: partition,
                producer_dependency: None,
                idempotency_key: [0x44; 32],
                roles: Vec::new(),
            },
            correspondence: Arc::new(
                crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
                    binding,
                    Vec::new(),
                    |_| None,
                )
                .expect("an empty output family can be rebound for selection evidence"),
            ),
        };

        assert_eq!(
            recovered_source_posture(
                std::slice::from_ref(&recovered),
                scope,
                partition,
                checkpoint_identity,
                &[binding],
            ),
            output_lineage::WorthQueryOutputSourcePosture::Exact(binding),
        );
        assert_eq!(
            recovered_source_posture(
                std::slice::from_ref(&recovered),
                scope,
                partition,
                crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new([0x55; 32]),
                &[binding],
            ),
            output_lineage::WorthQueryOutputSourcePosture::Drifted,
        );
        assert_eq!(
            recovered_source_posture(
                &[recovered],
                scope,
                [0x66; 32],
                checkpoint_identity,
                &[binding],
            ),
            output_lineage::WorthQueryOutputSourcePosture::Absent,
        );
    }
}
