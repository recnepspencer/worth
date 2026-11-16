//! Actual process installation documents shared by the Bank courtroom callers.
use super::{IdentityEndpoints, IdentityFixture, Path, SocketAddr};

pub fn server_configuration(
    fixture: &IdentityFixture,
    endpoints: &IdentityEndpoints,
    redirect_url: &str,
    rail_address: SocketAddr,
    home_directory: &Path,
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
        "home_directory": home_directory,
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
                "correction": { "freeze_grant": 26, "revoke_grant": 27 },
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
        "cold_certification": { "trusted_root_pem": endpoints.trusted_root_pem() }
    })
}

pub fn node_configuration(
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
        "cold_certification": { "trusted_root_pem": endpoints.trusted_root_pem() },
        "external_redirect_url": redirect_url
    })
}
