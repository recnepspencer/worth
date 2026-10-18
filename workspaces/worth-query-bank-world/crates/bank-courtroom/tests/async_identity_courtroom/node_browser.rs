use std::time::{Duration, Instant};

use thirtyfour::prelude::{ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};

use super::browser::{enter_value, submit_focused_stage};
use super::fixture::IdentityParticipant;

pub async fn complete_node_authorization(
    webdriver_url: &str,
    authorization_url: &str,
    participant: &IdentityParticipant,
) -> Result<(), String> {
    let mut capabilities = DesiredCapabilities::chrome();
    capabilities
        .add_arg("--headless=new")
        .map_err(|error| format!("headless capability failed: {error}"))?;
    capabilities
        .add_arg("--no-sandbox")
        .map_err(|error| format!("sandbox capability failed: {error}"))?;
    let driver = WebDriver::new(webdriver_url, capabilities)
        .await
        .map_err(|error| format!("WebDriver session failed: {error}"))?;
    let result = authenticate(&driver, authorization_url, participant).await;
    let quit = driver
        .quit()
        .await
        .map_err(|error| format!("WebDriver teardown failed: {error}"));
    result.and(quit)
}

async fn authenticate(
    driver: &WebDriver,
    authorization_url: &str,
    participant: &IdentityParticipant,
) -> Result<(), String> {
    driver
        .goto(authorization_url)
        .await
        .map_err(|error| format!("authorization navigation failed: {error}"))?;
    enter_value(
        driver,
        "input[name='uidField']",
        participant.username(),
        true,
    )
    .await?;
    submit_focused_stage(driver, "identification").await?;
    enter_value(
        driver,
        "input[autocomplete='current-password']",
        participant.password(),
        false,
    )
    .await?;
    submit_focused_stage(driver, "password").await?;
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let body = driver
            .source()
            .await
            .map_err(|error| format!("node callback page was unavailable: {error}"))?;
        if body.contains("authenticated") {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("node callback did not complete before deadline".to_string());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}
