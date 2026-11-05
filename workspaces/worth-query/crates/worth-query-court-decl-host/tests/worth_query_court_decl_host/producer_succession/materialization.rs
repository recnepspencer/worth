//! Independent output restoration in an app that also installs owned producers.
use super::{
    independent_generator_courts::settle_independent, producers::independent_parent,
    realization::*, *,
};
#[test]
fn independent_payload_and_its_retained_source_relation_rematerialize_with_owned_storage_installed()
{
    // Parent is independent; child/leaf/twin opt in and install current custody storage.
    assert_eq!(<independent_parent::initial::Producer as WorthQueryApplicationProducerBinding<Schema>>::OWNERSHIP,ProducerPublicationOwnership::Independent);
    assert_eq!(<producers::child::initial::Producer as WorthQueryApplicationProducerBinding<Schema>>::OWNERSHIP,ProducerPublicationOwnership::SourceScopeCascade);
    let selections = Selections::default();
    let app = world::install(0, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    settle_independent(&app, &request);
    let original = request
        .query(realization_read::ReadRealization("root".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let source = request
        .query(Read("root".into()))
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let selected = app
        .with_host_advancement(|phase| {
            app.runtime()
                .on_branch(app.runtime().current_world())
                .select(&phase)
        })
        .unwrap()
        .unwrap();
    let suspended = selected
        .suspend_current_generated_output::<independent_parent::initial::Producer>(&scope, source)
        .unwrap_or_else(|e| {
            panic!("suspend independent payload and incident Realizes edge: {e:?}")
        });
    assert!(request
        .query(realization_read::ReadRealization("root".into()))
        .execute()
        .is_err());
    let mut reconstruction = app
        .with_host_advancement(|phase| {
            app.runtime()
                .reconstruct_generated_output::<independent_parent::initial::Producer>(
                    &phase, suspended,
                )
        })
        .unwrap()
        .unwrap_or_else(|_| panic!("reconstruct actual manifest"));
    let WorthQueryReconstructedOutputEntity::Generated(output) = reconstruction
        .output::<independent_parent::initial::Output>()
        .unwrap()
    else {
        panic!("realization is the generated role")
    };
    reconstruction
        .field(&output, RealizationKey::reference(), original.key.clone())
        .unwrap();
    reconstruction
        .field(&output, RealizedValue::reference(), original.value)
        .unwrap();
    let retained = reconstruction
        .retained_relation_target(Realizes::reference(), &output, Node::reference())
        .unwrap();
    reconstruction
        .relation_to_retained(Realizes::reference(), &output, &retained)
        .unwrap();
    let completed = reconstruction
        .finish()
        .unwrap_or_else(|_| panic!("exact payload, fields and relation manifest"));
    app.runtime()
        .restore_generated_output(completed, &scope)
        .unwrap_or_else(|_| {
            panic!("rematerialize payload and relation with custody storage enabled")
        });
    let restored = request
        .query(realization_read::ReadRealization("root".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(
        restored, original,
        "restore exact entity identity and field values"
    );
    // Changed source data forces a real Preserve, whose handler verifies the
    // exact Realizes endpoint and Native verifies relation-write continuity.
    selections.lock().unwrap().clear();
    settlement::resize(&app, &request, 1, 77);
    settle_independent(&app, &request);
    assert!(selections
        .lock()
        .unwrap()
        .iter()
        .any(|s| s.key == "root" && s.preserve && s.prior));
}
