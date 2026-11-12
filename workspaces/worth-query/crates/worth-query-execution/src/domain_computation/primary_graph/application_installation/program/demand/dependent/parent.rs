//! Installed parent target and artifact meaning; each posture owns its diagnosis.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind as Kind;

pub(in crate::domain_computation::primary_graph) enum ParentMeaningMismatch {
    Connection,
    Feature,
    ArtifactDependency,
    ArtifactContract(WorthQueryOutputDemandDenial),
}
impl ParentMeaningMismatch {
    fn settled(self) -> WorthQueryOutputDemandDenial {
        match self {
            Self::Connection => WorthQueryOutputDemandDenial::new(
                Kind::ForeignDemand,
                "dependent output connection is not installed for this program",
            ),
            Self::Feature => WorthQueryOutputDemandDenial::new(
                Kind::ForeignSettlement,
                "dependent output edge does not leave this settled program feature",
            ),
            Self::ArtifactDependency => WorthQueryOutputDemandDenial::new(
                Kind::ForeignSettlement,
                "the dependent artifact does not consume its settled parent artifact",
            ),
            Self::ArtifactContract(cause) => cause,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn for_verified(
        self,
        source: worth_relational::facade::identity::EntityId,
        family: &str,
    ) -> WorthQueryOutputDemandDenial {
        let (kind, reason) = match self {
            Self::Connection => (Kind::ForeignDemand, "connection is not installed"),
            Self::Feature => (
                Kind::ForeignSource,
                "connection does not leave this parent's feature",
            ),
            Self::ArtifactDependency => (
                Kind::ForeignSource,
                "child artifact does not consume this parent's artifact",
            ),
            Self::ArtifactContract(cause) => {
                let subject = format!(
                    "verified history parent {family} source {source:?}: {}",
                    cause.subject()
                );
                return cause.with_subject(subject);
            }
        };
        WorthQueryOutputDemandDenial::new(
            kind,
            format!("verified history parent {family} source {source:?}: {reason}"),
        )
    }
}
impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub(in crate::domain_computation::primary_graph::application_installation::program::demand) fn validate_dependent_output_parent<
        D,
        C,
    >(
        &self,
        parent: &WorthQuerySettledProgramOutput<S, P, D>,
    ) -> Result<Option<Artifact>, WorthQueryOutputDemandDenial>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    {
        // Preserve the settled entrance's original refusal ordering and text.
        self.validate_parent_target::<C>(parent.target_feature)
            .map_err(ParentMeaningMismatch::settled)?;
        if !parent.retained.belongs_to(self.backing.runtime()) {
            return Err(WorthQueryOutputDemandDenial::new(
                Kind::ForeignSettlement,
                "dependent output parent belongs to another installed application",
            ));
        }
        self.validate_parent_artifact::<D, C>(parent.artifact)
            .map_err(ParentMeaningMismatch::settled)
    }
    pub(in crate::domain_computation::primary_graph) fn validate_dependent_parent_meaning<D, C>(
        &self,
        target_feature: TypeId,
        parent_artifact: Option<Artifact>,
    ) -> Result<Option<Artifact>, ParentMeaningMismatch>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    {
        self.validate_parent_target::<C>(target_feature)?;
        self.validate_parent_artifact::<D, C>(parent_artifact)
    }
    fn validate_parent_target<C: ApplicationConnectionShape<S>>(
        &self,
        target_feature: TypeId,
    ) -> Result<(), ParentMeaningMismatch> {
        if !self.contains_connection_type::<C>() {
            return Err(ParentMeaningMismatch::Connection);
        }
        if target_feature != TypeId::of::<C::SourceFeature>() {
            return Err(ParentMeaningMismatch::Feature);
        }
        Ok(())
    }
    fn validate_parent_artifact<D, C>(
        &self,
        parent_artifact: Option<Artifact>,
    ) -> Result<Option<Artifact>, ParentMeaningMismatch>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    {
        let artifact = self
            .validate_derived_artifact_demand::<C, ConnectionDemand<S, C>>()
            .map_err(ParentMeaningMismatch::ArtifactContract)?;
        if let (Some(child), Some(parent)) = (artifact, parent_artifact)
            && !child.dependencies().iter().any(|dependency| {
                dependency.identity() == parent.identity()
                    || dependency.identity() == parent.feature()
            })
        {
            return Err(ParentMeaningMismatch::ArtifactDependency);
        }
        Ok(artifact)
    }
}
