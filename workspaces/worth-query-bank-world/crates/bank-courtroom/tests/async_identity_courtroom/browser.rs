use std::time::{Duration, Instant};
use thirtyfour::prelude::{
    By, ChromiumLikeCapabilities, DesiredCapabilities, Key, WebDriver, WebElement,
};
use thirtyfour::LoggingPrefsLogLevel;

use super::callback::{CallbackReceiver, ReceivedAuthorizationCallback};
use super::fixture::IdentityParticipant;

const ELEMENT_WAIT: Duration = Duration::from_secs(45);
const ELEMENT_POLL: Duration = Duration::from_millis(250);

pub async fn complete_browser_authorization(
    webdriver_url: &str,
    authorization_url: &str,
    participant: &IdentityParticipant,
    callback: &CallbackReceiver,
) -> Result<ReceivedAuthorizationCallback, String> {
    complete_browser_authorization_with_delivery(
        webdriver_url,
        authorization_url,
        participant,
        callback,
        true,
    )
    .await
}

pub async fn complete_browser_authorization_after_response_loss(
    webdriver_url: &str,
    authorization_url: &str,
    participant: &IdentityParticipant,
    callback: &CallbackReceiver,
) -> Result<ReceivedAuthorizationCallback, String> {
    complete_browser_authorization_with_delivery(
        webdriver_url,
        authorization_url,
        participant,
        callback,
        false,
    )
    .await
}

async fn complete_browser_authorization_with_delivery(
    webdriver_url: &str,
    authorization_url: &str,
    participant: &IdentityParticipant,
    callback: &CallbackReceiver,
    deliver_callback_response: bool,
) -> Result<ReceivedAuthorizationCallback, String> {
    let mut capabilities = DesiredCapabilities::chrome();
    capabilities
        .add_arg("--headless=new")
        .map_err(|error| format!("headless capability failed: {error}"))?;
    capabilities
        .add_arg("--no-sandbox")
        .map_err(|error| format!("sandbox capability failed: {error}"))?;
    capabilities
        .set_browser_log_level(LoggingPrefsLogLevel::Warning)
        .map_err(|error| format!("browser logging capability failed: {error}"))?;
    let driver = WebDriver::new(webdriver_url, capabilities)
        .await
        .map_err(|error| format!("WebDriver session failed: {error}"))?;
    let callback_result = async {
        driver
            .goto(authorization_url)
            .await
            .map_err(|error| format!("authorization navigation failed: {error}"))?;
        enter_value(
            &driver,
            "input[name='uidField']",
            participant.username(),
            true,
        )
        .await?;
        submit_focused_stage(&driver, "identification").await?;
        enter_value(
            &driver,
            "input[autocomplete='current-password']",
            participant.password(),
            false,
        )
        .await?;
        submit_focused_stage(&driver, "password").await?;
        let receive = async {
            if deliver_callback_response {
                callback.receive().await
            } else {
                callback.receive_without_response().await
            }
        };
        match tokio::time::timeout(Duration::from_secs(90), receive).await {
            Ok(result) => result,
            Err(_) => Err(format!(
                "browser callback did not arrive before deadline; {}",
                postcredential_diagnostic(&driver).await
            )),
        }
    }
    .await;
    let quit_result = driver.quit().await;
    if let Err(error) = quit_result {
        return Err(format!("WebDriver teardown failed: {error}"));
    }
    callback_result
}

pub(super) async fn enter_value(
    driver: &WebDriver,
    selector: &str,
    value: &str,
    capture_precredential_render: bool,
) -> Result<(), String> {
    match find_visible_semantic_field(driver, selector).await {
        Ok(_) => {}
        Err(error) => {
            return Err(format!(
                "browser field `{selector}` was unavailable: {error}; {}",
                field_failure_diagnostic(driver, capture_precredential_render).await
            ));
        }
    }
    if let Err(error) = driver.action_chain().send_keys(value).perform().await {
        return Err(format!(
            "browser field `{selector}` rejected keyboard input: {error}; {}",
            field_failure_diagnostic(driver, capture_precredential_render).await
        ));
    }
    Ok(())
}

async fn find_visible_semantic_field(
    driver: &WebDriver,
    selector: &str,
) -> Result<WebElement, String> {
    let deadline = Instant::now() + ELEMENT_WAIT;
    loop {
        if let Some(element) = deep_visible_field(driver, selector).await? {
            return Ok(element);
        }
        if Instant::now() >= deadline {
            return Err("no visible enabled semantic field appeared before deadline".to_string());
        }
        tokio::time::sleep(ELEMENT_POLL).await;
    }
}

async fn deep_visible_field(
    driver: &WebDriver,
    selector: &str,
) -> Result<Option<WebElement>, String> {
    let selector_literal = format!("{selector:?}");
    let script = format!(
        r#"
        const selector = {selector_literal};
        const roots = [document];
        const belongsToRenderedBody = (field) => {{
            let element = field;
            while (element) {{
                if (element === document.body) {{
                    return true;
                }}
                if (element.parentElement) {{
                    element = element.parentElement;
                    continue;
                }}
                const root = element.getRootNode();
                element = root && root.host ? root.host : null;
            }}
            return false;
        }};
        for (let rootIndex = 0; rootIndex < roots.length && rootIndex < 64; rootIndex += 1) {{
            const root = roots[rootIndex];
            for (const element of root.querySelectorAll("*")) {{
                if (element.shadowRoot) {{
                    roots.push(element.shadowRoot);
                }}
            }}
            for (const field of root.querySelectorAll(selector)) {{
                const bounds = field.getBoundingClientRect();
                const style = getComputedStyle(field);
                if (
                    !field.disabled &&
                    belongsToRenderedBody(field) &&
                    bounds.width > 0 &&
                    bounds.height > 0 &&
                    style.display !== "none" &&
                    style.visibility !== "hidden" &&
                    style.opacity !== "0"
                ) {{
                    field.focus();
                    return field;
                }}
            }}
        }}
        return null;
        "#
    );
    let result = driver
        .execute(script, Vec::new())
        .await
        .map_err(|error| format!("semantic field lookup failed: {error}"))?;
    if result.json().is_null() {
        Ok(None)
    } else {
        result
            .element()
            .map(Some)
            .map_err(|error| format!("semantic field resolution failed: {error}"))
    }
}

pub(super) async fn submit_focused_stage(driver: &WebDriver, stage: &str) -> Result<(), String> {
    if let Err(error) = driver.action_chain().send_keys(Key::Enter).perform().await {
        return Err(format!(
            "browser {stage} stage rejected submission: {error}; {}",
            page_diagnostic(driver).await
        ));
    }
    Ok(())
}

async fn page_diagnostic(driver: &WebDriver) -> String {
    let location = match driver.current_url().await {
        Ok(mut url) => {
            url.set_query(None);
            url.set_fragment(None);
            url.to_string()
        }
        Err(error) => format!("<unavailable: {error}>"),
    };
    let title = driver
        .title()
        .await
        .unwrap_or_else(|error| format!("<unavailable: {error}>"));
    let inputs = element_inventory(driver, "input").await;
    let buttons = element_inventory(driver, "button").await;
    format!("page={{url={location:?}, title={title:?}, inputs={inputs:?}, buttons={buttons:?}}}")
}

async fn field_failure_diagnostic(
    driver: &WebDriver,
    capture_precredential_render: bool,
) -> String {
    let page = page_diagnostic(driver).await;
    let runtime = client_runtime_diagnostic(driver).await;
    let logs = browser_error_diagnostic(driver).await;
    let render = if capture_precredential_render {
        precredential_render_diagnostic(driver).await
    } else {
        "<suppressed after identifier entry>".to_string()
    };
    format!("{page}; client_runtime={runtime}; browser_logs={logs:?}; render={render}")
}

async fn postcredential_diagnostic(driver: &WebDriver) -> String {
    let page = page_diagnostic(driver).await;
    let runtime = client_runtime_diagnostic(driver).await;
    let logs = browser_error_diagnostic(driver).await;
    format!("{page}; client_runtime={runtime}; browser_logs={logs:?}")
}

async fn client_runtime_diagnostic(driver: &WebDriver) -> String {
    let script = r#"
        const resources = performance.getEntriesByType("resource")
            .filter((entry) => entry.responseStatus >= 400 || entry.transferSize === 0)
            .slice(0, 16)
            .map((entry) => {
                const resource = new URL(entry.name);
                return {
                    initiator: entry.initiatorType,
                    path: resource.pathname,
                    status: entry.responseStatus,
                    transfer_size: entry.transferSize,
                };
            });
        const field = document.querySelector("input[autocomplete='username']");
        const field_ancestors = [];
        for (let element = field; element && field_ancestors.length < 12; element = element.parentElement) {
            const style = getComputedStyle(element);
            field_ancestors.push({
                aria_hidden: element.getAttribute("aria-hidden"),
                classes: element.className,
                display: style.display,
                hidden: element.hidden,
                tag: element.tagName,
                visibility: style.visibility,
            });
        }
        return {
            body_text_length: document.body?.innerText?.length ?? -1,
            field_ancestors,
            ready_state: document.readyState,
            resources,
        };
    "#;
    match driver.execute(script, Vec::new()).await {
        Ok(result) => result.json().to_string(),
        Err(error) => format!("<unavailable: {error}>"),
    }
}

async fn precredential_render_diagnostic(driver: &WebDriver) -> String {
    let text = match driver.find(By::Css("body")).await {
        Ok(body) => body
            .text()
            .await
            .map(|text| text.chars().take(256).collect::<String>())
            .unwrap_or_else(|error| format!("<text unavailable: {error}>")),
        Err(error) => format!("<body unavailable: {error}>"),
    };
    format!("visible_text={text:?}")
}

async fn browser_error_diagnostic(driver: &WebDriver) -> Vec<String> {
    let Ok(entries) = driver.browser_log().await else {
        return vec!["<unavailable>".to_string()];
    };
    entries
        .into_iter()
        .take(16)
        .map(|entry| {
            let message = redact_url_details(&entry.message);
            format!(
                "level={:?},source={:?},message={message:?}",
                entry.level, entry.source
            )
        })
        .collect()
}

fn redact_url_details(message: &str) -> String {
    let mut redacted = String::with_capacity(message.len().min(512));
    let mut hiding = false;
    for character in message.chars().take(512) {
        if hiding && (character.is_whitespace() || matches!(character, '"' | '\'')) {
            hiding = false;
            redacted.push(character);
        } else if !hiding && matches!(character, '?' | '#') {
            hiding = true;
            redacted.push_str("<url-details-redacted>");
        } else if !hiding {
            redacted.push(character);
        }
    }
    redacted
}

async fn element_inventory(driver: &WebDriver, selector: &str) -> Vec<String> {
    let Ok(elements) = driver.find_all(By::Css(selector)).await else {
        return vec!["<inventory unavailable>".to_string()];
    };
    let mut inventory = Vec::with_capacity(elements.len().min(16));
    for element in elements.iter().take(16) {
        inventory.push(element_descriptor(element).await);
    }
    inventory
}

async fn element_descriptor(element: &WebElement) -> String {
    let name = safe_attribute(element, "name").await;
    let element_type = safe_attribute(element, "type").await;
    let autocomplete = safe_attribute(element, "autocomplete").await;
    let displayed = element.is_displayed().await.ok();
    let enabled = element.is_enabled().await.ok();
    format!(
        "name={name:?},type={element_type:?},autocomplete={autocomplete:?},displayed={displayed:?},enabled={enabled:?}"
    )
}

async fn safe_attribute(element: &WebElement, name: &str) -> Option<String> {
    element.attr(name).await.ok().flatten()
}
