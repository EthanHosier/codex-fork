use std::path::Path;

use anyhow::Result;
use serde_json::json;
use tempfile::TempDir;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::body_json;
use wiremock::matchers::header;
use wiremock::matchers::method;
use wiremock::matchers::path;

use super::API_KEY_ENV;
use super::BASE_URL_OVERRIDE_ENV;
use super::MODEL;
use super::PROVIDER_ID;
use super::WIRE_API;

const TEST_API_KEY: &str = "test-qlm-key";

fn codex_command(codex_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("codex")?);
    cmd.env("CODEX_HOME", codex_home);
    Ok(cmd)
}

fn configure_command(codex_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = codex_command(codex_home)?;
    cmd.args(["--configure-qlm"]);
    Ok(cmd)
}

#[test]
fn configure_qlm_requires_api_key_without_writing_config() -> Result<()> {
    let codex_home = TempDir::new()?;
    let output = configure_command(codex_home.path())?
        .env_remove(API_KEY_ENV)
        .output()?;

    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)?
            .contains(&format!("{API_KEY_ENV} is not set or is empty"))
    );
    assert!(!codex_home.path().join("config.toml").exists());
    Ok(())
}

#[tokio::test]
async fn configure_qlm_rejects_provider_error_without_writing_config() -> Result<()> {
    let codex_home = TempDir::new()?;
    let server = MockServer::start().await;
    let test_base_url = format!("{}/openai", server.uri());
    Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .and(header("authorization", format!("Bearer {TEST_API_KEY}")))
        .and(body_json(json!({
            "model": MODEL,
            "input": "Reply with OK.",
            "stream": true,
            "max_output_tokens": 16,
        })))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid API key"))
        .mount(&server)
        .await;

    let output = configure_command(codex_home.path())?
        .env(API_KEY_ENV, TEST_API_KEY)
        .env(BASE_URL_OVERRIDE_ENV, &test_base_url)
        .output()?;

    assert!(!output.status.success());
    assert_eq!(
        server.received_requests().await.unwrap_or_default().len(),
        1
    );
    assert!(String::from_utf8(output.stderr)?.contains("401"));
    assert!(!codex_home.path().join("config.toml").exists());
    Ok(())
}

#[tokio::test]
async fn configure_qlm_saves_provider_after_successful_responses_probe() -> Result<()> {
    let codex_home = TempDir::new()?;
    std::fs::write(
        codex_home.path().join("config.toml"),
        "approval_policy = \"never\"\n\n[model_providers.qlm]\nname = \"Existing QLM name\"\n",
    )?;
    let server = MockServer::start().await;
    let test_base_url = format!("{}/openai", server.uri());
    Mock::given(method("POST"))
        .and(path("/openai/responses"))
        .and(header("authorization", format!("Bearer {TEST_API_KEY}")))
        .and(body_json(json!({
            "model": MODEL,
            "input": "Reply with OK.",
            "stream": true,
            "max_output_tokens": 16,
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(
                    "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_test\",\"status\":\"completed\",\"output\":[]}}\n\n",
                ),
        )
        .mount(&server)
        .await;

    let output = configure_command(codex_home.path())?
        .env(API_KEY_ENV, TEST_API_KEY)
        .env(BASE_URL_OVERRIDE_ENV, &test_base_url)
        .output()?;

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        server.received_requests().await.unwrap_or_default().len(),
        1
    );
    let config: toml::Value = toml::from_str(&std::fs::read_to_string(
        codex_home.path().join("config.toml"),
    )?)?;
    assert_eq!(config["approval_policy"].as_str(), Some("never"));
    assert_eq!(config["model_provider"].as_str(), Some(PROVIDER_ID));
    assert_eq!(config["model"].as_str(), Some(MODEL));
    assert_eq!(
        config["model_providers"][PROVIDER_ID]["name"].as_str(),
        Some("Existing QLM name")
    );
    assert_eq!(
        config["model_providers"][PROVIDER_ID]["base_url"].as_str(),
        Some(test_base_url.as_str())
    );
    assert_eq!(
        config["model_providers"][PROVIDER_ID]["env_key"].as_str(),
        Some(API_KEY_ENV)
    );
    assert_eq!(
        config["model_providers"][PROVIDER_ID]["wire_api"].as_str(),
        Some(WIRE_API)
    );
    Ok(())
}
