//! Full-record correspondence conservatively declines even unprojected changes.
use super::*;
use worth_query_host::facade::admission::authenticated_principal::*;

pub(super) fn restore_changed_unprojected_field(
    world: &host::Host,
    provider: &Arc<prefix::Calls>,
    scope: &WorthQueryRequestScope,
) {
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, scope)
        .on_branch(world.current_world());
    let source = request
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let selected = world
        .with_host_advancement(|phase| {
            world
                .on_branch(world.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    let suspended = selected
        .suspend_current_generated_output::<producer::Producer>(
            scope,
            source.observed_sources()[0].clone(),
        )
        .unwrap();
    let completed = reconstruction::with_bounds(
        world,
        suspended,
        "taken-generated",
        bounds::Coordinates::new(7, 7, 8, 8),
    );
    let builds = provider.builds.load(Ordering::SeqCst);
    world
        .restore_generated_output(completed, scope)
        .unwrap_or_else(|failure| {
            panic!(
                "a lawful unprojected change uses normal reconstruction: {:?}",
                failure.cause()
            )
        });
    assert_eq!(
        provider.builds.load(Ordering::SeqCst),
        builds + 2,
        "both actual scopes rebuild when full authoritative records do not correspond"
    );
    for directory in ["A", "B"] {
        assert_eq!(
            values(request.query(find(directory)).execute().unwrap().rows()),
            expected(directory, "taken-generated")
        );
    }
}
