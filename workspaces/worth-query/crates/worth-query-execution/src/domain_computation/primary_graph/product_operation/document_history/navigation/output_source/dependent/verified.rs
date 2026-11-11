//! A verified D parent is read custody, never a demand or a settlement.
use super::*;
use crate::domain_computation::primary_graph::{
    application_installation::WorthQueryProgramApplicationRuntime,
    WorthQueryHistoryOutputQualificationDenial as Denial,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationFixedOutputRoot, ApplicationOutputGraphShape,
    ApplicationProgramDefinition,
};
use worth_query_installation::facade::ApplicationSchema;

type Artifact =
    worth_query_declaration::facade::application_program::ApplicationDerivedArtifactDeclaration;

/// The only issuer is the history parent owner. This is not convertible to
/// WorthQuerySettledHistoryProgramOutput or an ordinary parent settlement.
///
/// ```compile_fail,E0308
/// use worth_query_execution::publication_boundary::document_history::{
///     WorthQueryVerifiedHistoryParent, WorthQuerySettledHistoryProgramOutput,
/// };
/// use worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema,
///     application_program::ApplicationProgramDefinition};
/// fn forge<S: ApplicationSchema, P: ApplicationProgramDefinition<S>, D: WorthQueryApplicationOutputDemand<S>>(
///     parent: WorthQueryVerifiedHistoryParent<S, P, D>,
/// ) -> WorthQuerySettledHistoryProgramOutput<S, P, D> { parent }
/// ```
pub struct WorthQueryVerifiedHistoryParent<S, P, D> {
    custody: Arc<VerifiedHistoryParentCustody>,
    marker: std::marker::PhantomData<fn() -> (S, P, D)>,
}

pub(in crate::domain_computation::primary_graph) struct VerifiedHistoryParentCustody {
    profile: WorthQueryHistoryOutputProfile,
    original: Arc<WorthQueryApplicationReadObservation>,
    basis: Arc<WorthQueryApplicationReadObservation>,
    source: worth_relational::facade::identity::EntityId,
    partition: [u8; 32],
    family: &'static str,
    target_feature: std::any::TypeId,
    artifact: Option<Artifact>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl<S, P, D> WorthQueryVerifiedHistoryParent<S, P, D> {
    pub(in crate::domain_computation::primary_graph) fn from_admitted_child(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        child: crate::domain_computation::primary_graph::application_installation::HistoryAdmittedChildSource<S, P, D>,
    ) -> Result<Self, crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial>
    {
        let memory = crate::domain_computation::primary_graph::structural_walk::memory::reserve(
            execution, execution.has_accounting(),
            (std::mem::size_of::<VerifiedHistoryParentCustody>() + 2 * std::mem::size_of::<usize>()) as u64,
        ).map_err(crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial::Preparation)?;
        let (profile, original, basis, target_feature, artifact, source, partition, family) =
            child.into_parts();
        Ok(Self {
            custody: Arc::new(VerifiedHistoryParentCustody {
                profile,
                original,
                basis,
                source,
                partition,
                family,
                target_feature,
                artifact,
                _memory: memory,
            }),
            marker: std::marker::PhantomData,
        })
    }
    /// Compare a fresh owner-issued verification with this retained read custody.
    /// Equality itself grants no permission and cannot create a settlement.
    pub fn require_matching_output_custody(
        &self,
        fresh: Option<&Self>,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial>
    {
        if fresh.is_some_and(|other| {
            self.custody.profile.same_custody(&other.custody.profile)
                && Arc::ptr_eq(&self.custody.original, &other.custody.original)
                && Arc::ptr_eq(&self.custody.basis, &other.custody.basis)
                && self.custody.source == other.custody.source
                && self.custody.partition == other.custody.partition
                && self.custody.family == other.custody.family
        }) {
            Ok(())
        } else {
            Err(
                crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial::Demand(
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        format!(
                            "verified history child {} source {:?}",
                            self.family(),
                            self.source()
                        ),
                    ),
                ),
            )
        }
    }
    pub(in crate::domain_computation::primary_graph) fn source_route(
        &self,
    ) -> (
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        [u8; 32],
    ) {
        (crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(self.custody.source), self.custody.partition)
    }
    pub fn source(&self) -> worth_relational::facade::identity::EntityId {
        self.custody.source
    }
    pub fn family(&self) -> &'static str {
        self.custody.family
    }
    /// The exact candidate this owner verified; it conveys no current grant.
    pub fn observation(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.custody.basis
    }

    pub(in crate::domain_computation::primary_graph) fn custody(
        &self,
    ) -> &Arc<VerifiedHistoryParentCustody> {
        &self.custody
    }
}
impl VerifiedHistoryParentCustody {
    pub(in crate::domain_computation::primary_graph) fn profile(
        &self,
    ) -> &WorthQueryHistoryOutputProfile {
        &self.profile
    }
    pub(in crate::domain_computation::primary_graph) fn original(
        &self,
    ) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.original
    }
    pub(in crate::domain_computation::primary_graph) fn target_feature(&self) -> std::any::TypeId {
        self.target_feature
    }
    pub(in crate::domain_computation::primary_graph) fn artifact(&self) -> Option<Artifact> {
        self.artifact
    }
}

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    /// Authorize the captured source at C without projecting or admitting an
    /// original root. Stale output returns None for the genuine fallback owner.
    pub fn verify_history_root_parent<R>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        external: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<S>,
    ) -> Result<Option<WorthQueryVerifiedHistoryParent<S, P, Demand<S, R>>>, Denial>
    where
        R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
        Rows<S, R>: crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            S,
            Query<S, R>,
        >,
    {
        use crate::domain_computation::primary_graph::WorthQueryOriginalReadOutputDenial as Original;
        let profile = authority.profile();
        let runtime = self.runtime();
        let control = |cause| Denial::Control(Box::new(cause));
        authority
            .validate(phase, runtime, authority.request())
            .map_err(control)?;
        // A completed ordinary action already owns a genuine root and its
        // discovery. Recording keeps that original execution posture; this
        // verified historical-parent lane never substitutes for it.
        if authority.has_original_required_outputs() {
            return Ok(None);
        }
        let installed = runtime.installed_schema().installed_query_binding::<Source<S, R>>()
            .map_err(|cause| Denial::Source(Box::new(crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial::Admission(
                crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(cause.kind()), "verified history parent source")))))?;
        profile
            .source_intent()
            .authorize_graph_source(runtime, phase, authority, &installed, external)
            .map_err(|cause| Denial::Source(Box::new(cause)))?;
        if !profile.uses_owned_model_graph()
            || profile.is_output_forest()
            || !profile.matches_program(self)
            || profile.root != crate::equality_type_id::EqualityTypeId::of::<R>().into_external()
        {
            return Err(Denial::ParentMismatch {
                source: profile.source_intent().source_subject(),
                family: profile.family,
            });
        }
        self.validate_selected_output_root::<R>(phase, access, authority.original())
            .map_err(Original::Program)?;
        let artifact = self
            .validate_root_artifact_demand::<R>()
            .map_err(Original::Output)?;
        let execution = phase
            .checked_execution_request_for(runtime.product_runtime())
            .map_err(|cause| control(WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
        let data = if authority.recording_model().is_some() {
            runtime
                .on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(
                    authority.original().branch_incarnation(),
                ))
                .select(phase)
                .map_err(Original::Observation)?
                .retain_application_read()
        } else {
            Arc::clone(authority.original())
        };
        let selected = runtime
            .select_application_read_observation(phase, &data)
            .map_err(Original::Observation)?;
        let current =
            runtime
                .runtime
                .primary_provider
                .graph
                .with_runtime(&execution, |native| {
                    if let Some(model)=authority.recording_model() {
                        return super::super::super::output_verification::current_recording_route_is_current(
                            &execution,native,&selected,model,profile.source_intent().source_subject(),profile.family,profile.source_intent().source_partition())
                            .map_err(|cause|Denial::Source(Box::new(cause)));
                    }
                    let evidence = authority
                        .selected_output_read(&execution, native)
                        .map_err(Denial::SelectionRead)?;
                    super::super::super::output_verification::selected_profile_root_is_current(
                        &execution, native, evidence, &selected, profile,
                    )
                })?;
        if !current {
            return authority
                .confirm_read(phase, runtime, authority.request(), None)
                .map_err(control);
        }
        let bytes = (std::mem::size_of::<VerifiedHistoryParentCustody>()
            + 2 * std::mem::size_of::<usize>()) as u64;
        let memory = crate::domain_computation::primary_graph::structural_walk::memory::reserve(
            &execution,
            execution.has_accounting(),
            bytes,
        )
        .map_err(|cause| {
            Denial::Original(Original::Output(
                crate::domain_computation::primary_graph::output_reuse::read_denial(cause),
            ))
        })?;
        let parent = WorthQueryVerifiedHistoryParent {
            custody: Arc::new(VerifiedHistoryParentCustody {
                profile: profile.clone(),
                original: Arc::clone(authority.original()),
                basis: Arc::clone(&data),
                source: profile.source_intent().source_subject(),
                partition: profile.source_intent().source_partition(),
                family: profile.family,
                target_feature: crate::equality_type_id::EqualityTypeId::of::<
                    <R::RootConnection as ApplicationConnectionShape<S>>::TargetFeature,
                >()
                .into_external(),
                artifact,
                _memory: memory,
            }),
            marker: std::marker::PhantomData,
        };
        authority
            .confirm_read(phase, runtime, authority.request(), Some(parent))
            .map_err(control)
    }
}

impl<S, P, D> Clone for WorthQueryVerifiedHistoryParent<S, P, D> {
    fn clone(&self) -> Self {
        Self {
            custody: Arc::clone(&self.custody),
            marker: std::marker::PhantomData,
        }
    }
}
