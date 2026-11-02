//! Installed connection and artifact meaning of one actual settled parent.
use super::*;

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
        use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind as Kind;
        if !self.contains_connection_type::<C>() {
            return Err(WorthQueryOutputDemandDenial::new(
                Kind::ForeignDemand,
                "dependent output connection is not installed for this program",
            ));
        }
        if parent.target_feature != TypeId::of::<C::SourceFeature>() {
            return Err(WorthQueryOutputDemandDenial::new(
                Kind::ForeignSettlement,
                "dependent output edge does not leave this settled program feature",
            ));
        }
        if !parent.retained.belongs_to(&self.backing.runtime) {
            return Err(WorthQueryOutputDemandDenial::new(
                Kind::ForeignSettlement,
                "dependent output parent belongs to another installed application",
            ));
        }
        let artifact = self.validate_derived_artifact_demand::<C, ConnectionDemand<S, C>>()?;
        if let (Some(child), Some(parent_artifact)) = (artifact, parent.artifact)
            && !child.dependencies().iter().any(|dependency| {
                dependency.identity() == parent_artifact.identity()
                    || dependency.identity() == parent_artifact.feature()
            })
        {
            return Err(WorthQueryOutputDemandDenial::new(
                Kind::ForeignSettlement,
                "the dependent artifact does not consume its settled parent artifact",
            ));
        }
        Ok(artifact)
    }
}
