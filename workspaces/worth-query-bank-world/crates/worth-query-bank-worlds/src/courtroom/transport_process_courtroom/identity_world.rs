#[allow(dead_code)]
#[path = "../async_identity_courtroom/administration.rs"]
pub mod administration;
#[allow(dead_code)]
#[path = "../async_identity_courtroom/browser.rs"]
pub mod browser;
#[allow(dead_code)]
#[path = "../async_identity_courtroom/docker_world.rs"]
pub mod docker_world;
#[allow(dead_code)]
#[path = "../async_identity_courtroom/fixture.rs"]
pub mod fixture;
#[path = "../async_identity_courtroom/node_browser.rs"]
pub mod node_browser;

// The browser owner expects this sibling for direct-callback entry points.
#[allow(dead_code)]
#[path = "../async_identity_courtroom/callback.rs"]
pub mod callback;
