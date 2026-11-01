use super::*;
use std::marker::PhantomData;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneFacesRead {
    pub scene_key: String,
}
worth_query_structured_value_binding!(pub SceneFacesInputBinding for SceneFacesRead { identity: "worth.query.scene-face.read-input.v1" });
pub type SceneFacesScope<S> =
    ApplicationQueryFieldScope<S, Scene, SceneFacts, SceneKey, String, ReadOnly, NoApplicationUnit>;
pub struct SceneFacesReadBinding<S>(PhantomData<fn() -> S>);
impl<S: SceneFaceSchemaBinding> ApplicationQueryBinding<S> for SceneFacesReadBinding<S> {
    type Input = SceneFacesRead;
    type InputBinding = SceneFacesInputBinding;
    type Query = SceneFacesQuery;
    type ParameterBinding = SceneFacesParametersBinding;
    type ResultBinding = SceneFaceRowBinding;
    type ScopeBinding = SceneFacesScope<S>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.scene-face.read.v1";
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
impl<S: SceneFaceSchemaBinding> ApplicationQueryIntent<S> for SceneFacesRead {
    type Binding = SceneFacesReadBinding<S>;
    fn parameters(&self) -> ApplicationQueryParameterSet<SceneFacesQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> SceneFacesScope<S> {
        SceneFacesScope::new(SceneKey::reference(), self.scene_key)
    }
}
