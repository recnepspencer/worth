use std::net::SocketAddr;
use std::path::Path;

use bank_external_rail::RailProcessHandle;

#[path = "world/rail.rs"]
mod rail;

use super::identity_world::administration::AuthentikAdministration;
use super::identity_world::docker_world::{DockerIdentityWorld, IdentityEndpoints};
use super::identity_world::fixture::IdentityFixture;
use super::identity_world::node_browser::complete_node_authorization;
use super::process::CourtroomProcess;

pub struct TransportProcessWorld {
    primary_node: CourtroomProcess,
    replacement_node: Option<CourtroomProcess>,
    peer_node: CourtroomProcess,
    approver_node: CourtroomProcess,
    reviewer_node: CourtroomProcess,
    bank_server: CourtroomProcess,
    rail: RailProcessHandle,
    docker: DockerIdentityWorld,
    fixture: IdentityFixture,
    endpoints: IdentityEndpoints,
    replacement_redirect: String,
    server_address: SocketAddr,
    pub client: reqwest::Client,
    pub primary_address: SocketAddr,
    pub peer_address: SocketAddr,
    pub approver_address: SocketAddr,
    pub reviewer_address: SocketAddr,
}

impl TransportProcessWorld {
    pub async fn start() -> Self {
        let rail = RailProcessHandle::spawn(
            Path::new(env!("CARGO_BIN_EXE_cold-bank-rail")),
            "127.0.0.1:0",
        )
        .expect("the external rail should start in a separate process");
        let (mut primary_node, primary_bound) = spawn_user_node().await;
        let (mut peer_node, peer_bound) = spawn_user_node().await;
        let (mut approver_node, approver_bound) = spawn_user_node().await;
        let (mut reviewer_node, reviewer_bound) = spawn_user_node().await;
        let (replacement_node, replacement_bound) = spawn_user_node().await;
        let primary_address = parse_address(&primary_bound.address);
        let peer_address = parse_address(&peer_bound.address);
        let approver_address = parse_address(&approver_bound.address);
        let reviewer_address = parse_address(&reviewer_bound.address);
        let replacement_address = parse_address(&replacement_bound.address);
        let primary_redirect = external_redirect(primary_address);
        let peer_redirect = external_redirect(peer_address);
        let approver_redirect = external_redirect(approver_address);
        let reviewer_redirect = external_redirect(reviewer_address);
        let replacement_redirect = external_redirect(replacement_address);
        let fixture = IdentityFixture::dynamic_with_redirects(vec![
            primary_redirect.clone(),
            peer_redirect.clone(),
            approver_redirect.clone(),
            reviewer_redirect.clone(),
            replacement_redirect.clone(),
        ]);
        let docker = DockerIdentityWorld::start(&fixture)
            .await
            .expect("cold identity world should start");
        let endpoints = docker
            .wait_until_ready(&fixture)
            .await
            .expect("cold identity world should become ready");
        let (mut bank_server, server_bound) = spawn_bank_server().await;
        let server_address = parse_address(&server_bound.address);
        let server_ready = bank_server
            .install(server_configuration(
                &fixture,
                &endpoints,
                &primary_redirect,
                rail.local_addr(),
            ))
            .await
            .expect("authoritative Bank process should become ready");
        let primary_ready = primary_node
            .install(node_configuration(
                &fixture,
                &endpoints,
                server_address,
                &primary_redirect,
            ))
            .await
            .expect("primary node should become ready");
        let peer_ready = peer_node
            .install(node_configuration(
                &fixture,
                &endpoints,
                server_address,
                &peer_redirect,
            ))
            .await
            .expect("peer node should become ready");
        let approver_ready = approver_node
            .install(node_configuration(
                &fixture,
                &endpoints,
                server_address,
                &approver_redirect,
            ))
            .await
            .expect("approver node should become ready");
        let reviewer_ready = reviewer_node
            .install(node_configuration(
                &fixture,
                &endpoints,
                server_address,
                &reviewer_redirect,
            ))
            .await
            .expect("reviewer node should become ready");
        assert_distinct_processes(&[
            std::process::id(),
            server_ready.process_id,
            primary_ready.process_id,
            peer_ready.process_id,
            approver_ready.process_id,
            reviewer_ready.process_id,
            replacement_bound.process_id,
            rail.pid(),
        ]);
        assert_eq!(parse_address(&server_ready.address), server_address);
        Self {
            primary_node,
            replacement_node: Some(replacement_node),
            peer_node,
            approver_node,
            reviewer_node,
            bank_server,
            rail,
            docker,
            fixture,
            endpoints,
            replacement_redirect,
            server_address,
            client: reqwest::Client::new(),
            primary_address,
            peer_address,
            approver_address,
            reviewer_address,
        }
    }

    pub async fn authenticate_participants(&self) {
        self.authenticate_primary().await;
        authenticate_node(
            &self.client,
            self.peer_address,
            &self.endpoints.webdriver_url(),
            &self.fixture.participants()[1],
        )
        .await;
        authenticate_node(
            &self.client,
            self.approver_address,
            &self.endpoints.webdriver_url(),
            &self.fixture.participants()[4],
        )
        .await;
        authenticate_node(
            &self.client,
            self.reviewer_address,
            &self.endpoints.webdriver_url(),
            &self.fixture.participants()[5],
        )
        .await;
    }

    pub async fn authenticate_primary(&self) {
        authenticate_node(
            &self.client,
            self.primary_address,
            &self.endpoints.webdriver_url(),
            self.fixture.primary_participant(),
        )
        .await;
    }

    pub async fn set_access_token_validity(&self, validity: &str) {
        AuthentikAdministration::new(
            self.endpoints.authentik_origin(),
            self.fixture.bootstrap_token().to_owned(),
        )
        .expect("Authentik administration should configure")
        .set_access_token_validity(self.fixture.slug(), validity)
        .await
        .expect("provider token validity should update");
    }

    pub async fn crash_and_restart_primary(&mut self) -> SocketAddr {
        let replacement = self
            .replacement_node
            .take()
            .expect("replacement node is consumed once");
        let restarted_address = replacement.local_address();
        let crashed = std::mem::replace(&mut self.primary_node, replacement);
        crashed
            .terminate()
            .await
            .expect("primary node crash should reap its process");
        self.primary_node
            .install(node_configuration(
                &self.fixture,
                &self.endpoints,
                self.server_address,
                &self.replacement_redirect,
            ))
            .await
            .expect("replacement node should become ready");
        self.primary_address = restarted_address;
        restarted_address
    }

    pub async fn shutdown(self) {
        self.primary_node
            .shutdown()
            .await
            .expect("primary node should stop");
        self.peer_node
            .shutdown()
            .await
            .expect("peer node should stop");
        self.approver_node
            .shutdown()
            .await
            .expect("approver node should stop");
        self.reviewer_node
            .shutdown()
            .await
            .expect("reviewer node should stop");
        self.bank_server
            .shutdown()
            .await
            .expect("Bank server should stop");
        let project = self.docker.project_name();
        drop(self.docker);
        DockerIdentityWorld::require_project_absent(&project)
            .expect("courtroom teardown should remove every Docker resource");
    }
}

async fn spawn_user_node() -> (CourtroomProcess, super::process::ProcessPosture) {
    CourtroomProcess::spawn(Path::new(env!("CARGO_BIN_EXE_cold-bank-user-node")))
        .await
        .expect("user-node process should bind")
}

async fn spawn_bank_server() -> (CourtroomProcess, super::process::ProcessPosture) {
    CourtroomProcess::spawn(Path::new(env!("CARGO_BIN_EXE_cold-bank-http-server")))
        .await
        .expect("authoritative Bank process should bind")
}

async fn authenticate_node(
    client: &reqwest::Client,
    address: SocketAddr,
    webdriver: &str,
    participant: &super::identity_world::fixture::IdentityParticipant,
) {
    let outcome = client
        .post(format!("http://{address}/session/authorize"))
        .send()
        .await
        .expect("node authorization should respond")
        .json::<bank_user_node::BankUserNodeAuthorizationOutcome>()
        .await
        .expect("node authorization response should be typed");
    let bank_user_node::BankUserNodeAuthorizationOutcome::AuthorizationRequired {
        authorization_url,
    } = outcome
    else {
        panic!("fresh node did not require authorization: {outcome:?}");
    };
    complete_node_authorization(webdriver, &authorization_url, participant)
        .await
        .expect("real browser authorization should complete at the node");
}

fn server_configuration(
    fixture: &IdentityFixture,
    endpoints: &IdentityEndpoints,
    redirect_url: &str,
    rail_address: SocketAddr,
) -> serde_json::Value {
    let participants = fixture
        .participants()
        .iter()
        .enumerate()
        .map(|(index, participant)| {
            serde_json::json!({
                "principal": index + 1,
                "external_subject": participant.username(),
                "account": {
                    "identity": format!("fixture:{}", index + 100),
                    "display_name": format!("Courtroom account {}", index + 1),
                    "status": "open",
                    "activity_amounts_minor": [1_000, 200]
                }
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "external_rail": rail_address,
        "oidc": {
            "issuer": endpoints.issuer(),
            "client_id": fixture.client_id(),
            "client_secret": fixture.client_secret(),
            "redirect_url": redirect_url,
            "introspection_url": endpoints.introspection_url(),
            "revocation_url": endpoints.revocation_url()
        },
        "world": {
            "institution": 1,
            "institution_cash_account": "fixture:1",
            "participants": participants,
            "estate": {
                "branch": 2,
                "estate": 3,
                "estate_account": "fixture:100",
                "deceased_principal": 1,
                "specialist_principal": 2,
                "assignment": 11,
                "notice": 12,
                "grant": 14,
                "aftermath": {
                    "destination_account": "fixture:102",
                    "beneficiary_principal": 3,
                    "executor_principal": 4,
                    "legal_authority": 15,
                    "disbursement_grant": 16,
                    "compensation_service_assignment": 25,
                    "amount_ceiling_minor": 10_000
                },
                "elevation": {
                    "requester_principal": 2,
                    "approver_principal": 5,
                    "reviewer_principal": 6,
                    "approver_assignment": 17,
                    "reviewer_assignment": 18,
                    "request_grant": 19,
                    "upper_bound_grant": 20,
                    "self_approval_grant": 21,
                    "approval_grant": 22,
                    "close_grant": 23,
                    "review_grant": 24
                }
            }
        },
        "cold_certification": true
    })
}

fn node_configuration(
    fixture: &IdentityFixture,
    endpoints: &IdentityEndpoints,
    server: SocketAddr,
    redirect_url: &str,
) -> serde_json::Value {
    serde_json::json!({
        "issuer": endpoints.issuer(),
        "client_id": fixture.client_id(),
        "client_secret": fixture.client_secret(),
        "introspection_url": endpoints.introspection_url(),
        "revocation_url": endpoints.revocation_url(),
        "bank_server_origin": format!("http://{server}/"),
        "maximum_request_concurrency": 1,
        "maximum_live_streams": 1,
        "cold_certification": true,
        "external_redirect_url": redirect_url
    })
}

fn parse_address(value: &str) -> SocketAddr {
    value.parse().expect("process address should be canonical")
}

fn external_redirect(address: SocketAddr) -> String {
    format!(
        "http://host.docker.internal:{}/oidc/callback",
        address.port()
    )
}

fn assert_distinct_processes(processes: &[u32]) {
    let unique = processes.iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        unique.len(),
        processes.len(),
        "every runtime boundary needs a distinct PID"
    );
}
