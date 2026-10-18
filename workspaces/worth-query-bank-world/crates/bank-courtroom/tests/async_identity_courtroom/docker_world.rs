#[path = "docker_world/blueprint.rs"]
mod blueprint;
#[path = "docker_world/tls.rs"]
mod tls;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant, SystemTime};

use super::fixture::IdentityFixture;

const AUTHENTIK_IMAGE: &str = "ghcr.io/goauthentik/server:2026.5.6";
const POSTGRES_IMAGE: &str = "docker.io/library/postgres:16-alpine";
const SELENIUM_IMAGE: &str = "docker.io/selenium/standalone-chrome:4.46.0-20260707";

pub struct DockerIdentityWorld {
    directory: PathBuf,
    project: String,
    authentik_port: u16,
    selenium_port: u16,
    trusted_root_pem: String,
}

impl DockerIdentityWorld {
    pub async fn start(fixture: &IdentityFixture) -> Result<Self, String> {
        let unique = unique_suffix();
        let directory = std::env::temp_dir().join(format!("worth-bank-courtroom-{unique}"));
        std::fs::create_dir_all(&directory)
            .map_err(|error| format!("courtroom directory creation failed: {error}"))?;
        let project = format!("worthbank{unique}");
        let mut world = Self {
            directory,
            project,
            authentik_port: 0,
            selenium_port: 0,
            trusted_root_pem: String::new(),
        };
        std::fs::write(world.directory.join("compose.yml"), compose_yaml(fixture))
            .map_err(|error| format!("courtroom compose write failed: {error}"))?;
        let trusted_root_pem = tls::prepare(&world.directory, AUTHENTIK_IMAGE)?;
        std::fs::write(
            world.directory.join("bank-blueprint.yaml"),
            tls::declare(&world.directory, blueprint::blueprint_yaml(fixture))?,
        )
        .map_err(|error| format!("courtroom blueprint write failed: {error}"))?;
        world.trusted_root_pem = trusted_root_pem;
        world.compose(["up", "-d"]).map(|_| ())?;
        world.compose([
            "exec",
            "-T",
            "selenium",
            "certutil",
            "-d",
            "sql:/home/seluser/.pki/nssdb",
            "-A",
            "-t",
            "C,,",
            "-n",
            "WORTH isolated court CA",
            "-i",
            "/worth-trust/root.pem",
        ])?;
        world.authentik_port = world.published_port("server", 9443)?;
        world.selenium_port = world.published_port("selenium", 4444)?;
        Ok(world)
    }

    pub async fn wait_until_ready(
        &self,
        fixture: &IdentityFixture,
    ) -> Result<IdentityEndpoints, String> {
        let endpoints = IdentityEndpoints::new(
            self.authentik_port,
            self.selenium_port,
            fixture.slug(),
            self.trusted_root_pem.clone(),
        );
        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .add_root_certificate(
                reqwest::Certificate::from_pem(self.trusted_root_pem.as_bytes())
                    .map_err(|error| format!("named root certificate invalid: {error}"))?,
            )
            .build()
            .map_err(|error| format!("readiness client failed: {error}"))?;
        let deadline = Instant::now() + Duration::from_secs(360);
        loop {
            let discovery_ready = endpoint_ready(&client, &endpoints.discovery_url()).await;
            let alternate_discovery_ready = endpoint_ready(
                &client,
                &endpoints.discovery_url_for(fixture.alternate_slug()),
            )
            .await;
            let selenium_ready = endpoint_ready(&client, &endpoints.webdriver_status_url()).await;
            if discovery_ready && alternate_discovery_ready && selenium_ready {
                return Ok(endpoints);
            }
            if Instant::now() >= deadline {
                return Err(
                    "Authentik or Selenium did not become ready before deadline".to_string()
                );
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    pub fn project_name(&self) -> String {
        self.project.clone()
    }

    pub fn directory_path(&self) -> PathBuf {
        self.directory.clone()
    }

    pub fn require_project_absent(project: &str) -> Result<(), String> {
        for resource in ["container", "volume", "network"] {
            let output = Command::new("docker")
                .args([
                    resource,
                    "ls",
                    "--filter",
                    &format!("label=com.docker.compose.project={project}"),
                    "--quiet",
                ])
                .output()
                .map_err(|error| format!("Docker {resource} inspection failed: {error}"))?;
            if !output.status.success() {
                return Err(command_failure("Docker teardown inspection", &output));
            }
            if !output.stdout.is_empty() {
                return Err(format!(
                    "Docker teardown left {resource} resources for project {project}"
                ));
            }
        }
        Ok(())
    }

    fn published_port(&self, service: &str, container_port: u16) -> Result<u16, String> {
        let output = self.compose(["port", service, &container_port.to_string()])?;
        let address = String::from_utf8(output.stdout)
            .map_err(|_| format!("Docker port output for {service} was not UTF-8"))?;
        address
            .trim()
            .rsplit_once(':')
            .and_then(|(_, port)| port.parse().ok())
            .ok_or_else(|| format!("Docker did not publish a usable {service} port: {address:?}"))
    }

    fn compose<const N: usize>(&self, arguments: [&str; N]) -> Result<Output, String> {
        let output = Command::new("docker")
            .args(["compose", "--project-name", &self.project, "--file"])
            .arg(self.directory.join("compose.yml"))
            .args(arguments)
            .current_dir(&self.directory)
            .output()
            .map_err(|error| format!("Docker Compose could not start: {error}"))?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(command_failure("Docker Compose", &output))
        }
    }
}

impl Drop for DockerIdentityWorld {
    fn drop(&mut self) {
        let _ = self.compose(["down", "--volumes", "--remove-orphans"]);
        if verified_courtroom_directory(&self.directory) {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }
}

pub struct IdentityEndpoints {
    authentik_origin: String,
    selenium_origin: String,
    slug: String,
    trusted_root_pem: String,
}

impl IdentityEndpoints {
    fn new(authentik_port: u16, selenium_port: u16, slug: &str, trusted_root_pem: String) -> Self {
        Self {
            authentik_origin: format!("https://host.docker.internal:{authentik_port}"),
            selenium_origin: format!("http://127.0.0.1:{selenium_port}"),
            slug: slug.to_string(),
            trusted_root_pem,
        }
    }

    pub fn trusted_root_pem(&self) -> &str {
        &self.trusted_root_pem
    }

    pub fn issuer(&self) -> String {
        self.issuer_for(&self.slug)
    }

    pub fn issuer_for(&self, slug: &str) -> String {
        format!("{}/application/o/{slug}/", self.authentik_origin)
    }

    pub fn authentik_origin(&self) -> String {
        self.authentik_origin.clone()
    }

    pub fn discovery_url(&self) -> String {
        format!("{}.well-known/openid-configuration", self.issuer())
    }

    pub fn discovery_url_for(&self, slug: &str) -> String {
        format!("{}.well-known/openid-configuration", self.issuer_for(slug))
    }

    pub fn introspection_url(&self) -> String {
        format!("{}/application/o/introspect/", self.authentik_origin)
    }

    pub fn revocation_url(&self) -> String {
        format!("{}/application/o/revoke/", self.authentik_origin)
    }

    pub fn webdriver_url(&self) -> String {
        self.selenium_origin.clone()
    }

    fn webdriver_status_url(&self) -> String {
        format!("{}/status", self.selenium_origin)
    }
}

async fn endpoint_ready(client: &reqwest::Client, url: &str) -> bool {
    client
        .get(url)
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .is_ok_and(|response| response.status().is_success())
}

fn command_failure(label: &str, output: &Output) -> String {
    format!(
        "{label} failed with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn verified_courtroom_directory(path: &Path) -> bool {
    path.parent() == Some(std::env::temp_dir().as_path())
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("worth-bank-courtroom-"))
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("{}{}", std::process::id(), nanos)
}

fn compose_yaml(fixture: &IdentityFixture) -> String {
    format!(
        r#"services:
  postgresql:
    image: {POSTGRES_IMAGE}
    environment:
      POSTGRES_DB: authentik
      POSTGRES_USER: authentik
      POSTGRES_PASSWORD: {postgres_password}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -d authentik -U authentik"]
      interval: 2s
      timeout: 2s
      retries: 60
    volumes:
      - database:/var/lib/postgresql/data
  server:
    image: {AUTHENTIK_IMAGE}
    command: server
    depends_on:
      postgresql:
        condition: service_healthy
    environment: &authentik_environment
      AUTHENTIK_POSTGRESQL__HOST: postgresql
      AUTHENTIK_POSTGRESQL__NAME: authentik
      AUTHENTIK_POSTGRESQL__USER: authentik
      AUTHENTIK_POSTGRESQL__PASSWORD: {postgres_password}
      AUTHENTIK_SECRET_KEY: {authentik_secret}
    ports:
      # The host adapter and Selenium must observe one exact issuer origin.
      - "0.0.0.0::9443"
  worker:
    image: {AUTHENTIK_IMAGE}
    command: worker
    depends_on:
      postgresql:
        condition: service_healthy
    environment:
      <<: *authentik_environment
      AUTHENTIK_BOOTSTRAP_TOKEN: {bootstrap_token}
    volumes:
      - ./bank-blueprint.yaml:/blueprints/worth-bank.yaml:ro
  selenium:
    image: {SELENIUM_IMAGE}
    shm_size: 2gb
    volumes:
      - ./root.pem:/worth-trust/root.pem:ro
    ports:
      - "127.0.0.1::4444"
volumes:
  database:
"#,
        postgres_password = fixture.postgres_password(),
        authentik_secret = fixture.authentik_secret(),
        bootstrap_token = fixture.bootstrap_token(),
    )
}
