use super::*;
use std::marker::PhantomData;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneFaceBoxesRead {
    pub scene_key: String,
    pub probe: SceneBoxProbe,
}
worth_query_structured_value_binding!(pub SceneFaceBoxesInputBinding for SceneFaceBoxesRead { identity: "worth.query.scene-face.box-input.v1" });
pub struct SceneFaceBoxesReadBinding<S>(PhantomData<fn() -> S>);
impl<S: SceneFaceSchemaBinding> ApplicationQueryBinding<S> for SceneFaceBoxesReadBinding<S> {
    type Input = SceneFaceBoxesRead;
    type InputBinding = SceneFaceBoxesInputBinding;
    type Query = SceneFaceBoxesQuery;
    type ParameterBinding = SceneBoxParametersBinding;
    type ResultBinding = SceneFaceRowBinding;
    type ScopeBinding = SceneFacesScope<S>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.scene-face.box-read.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(100_000);
    fn scope_field() -> ApplicationFieldRef<
        S,
        Scene,
        SceneFacts,
        SceneKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        SceneKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        S,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}
impl<S: SceneFaceSchemaBinding> ApplicationQueryIntent<S> for SceneFaceBoxesRead {
    type Binding = SceneFaceBoxesReadBinding<S>;
    fn parameters(&self) -> ApplicationQueryParameterSet<SceneFaceBoxesQuery> {
        ApplicationQueryParameterSet::new()
            .bind(probe_parameter(), self.probe)
            .expect("typed Scene box probe binding")
    }
    fn into_scope(self) -> SceneFacesScope<S> {
        SceneFacesScope::new(SceneKey::reference(), self.scene_key)
    }
}
