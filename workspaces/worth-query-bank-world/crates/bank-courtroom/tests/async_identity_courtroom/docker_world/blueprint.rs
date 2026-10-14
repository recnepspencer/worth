//! Actual OIDC applications and participants declared to the isolated Authentik backend.
use super::IdentityFixture;
pub(super) fn blueprint_yaml(fixture: &IdentityFixture) -> String {
    let user_entries = fixture
        .participants()
        .iter()
        .map(|participant| {
            format!(
                r#"  - model: authentik_core.user
    state: must_created
    identifiers:
      username: {username}
    attrs:
      name: Dynamic courtroom user
      email: {username}@courtroom.invalid
      is_active: true
      password: {password}
"#,
                username = participant.username(),
                password = participant.password(),
            )
        })
        .collect::<String>();
    let redirect_uris = fixture
        .redirect_urls()
        .iter()
        .map(|redirect_url| {
            format!(
                "        - matching_mode: strict\n          redirect_uri_type: authorization\n          url: {redirect_url}\n"
            )
        })
        .collect::<String>();
    format!(
        r#"version: 1
metadata:
  name: WORTH bank identity courtroom
entries:
{user_entries}
  - model: authentik_providers_oauth2.oauth2provider
    state: must_created
    id: bank-provider
    identifiers:
      name: {slug}
    attrs:
      authorization_flow: !Find [authentik_flows.flow, [slug, default-provider-authorization-implicit-consent]]
      invalidation_flow: !Find [authentik_flows.flow, [slug, default-provider-invalidation-flow]]
      client_type: confidential
      client_id: {client_id}
      client_secret: {client_secret}
      grant_types:
        - authorization_code
      include_claims_in_id_token: true
      issuer_mode: per_provider
      sub_mode: user_username
      signing_key: !Find [authentik_crypto.certificatekeypair, [name, authentik Self-signed Certificate]]
      property_mappings:
        - !Find [authentik_providers_oauth2.scopemapping, [scope_name, openid]]
        - !Find [authentik_providers_oauth2.scopemapping, [scope_name, email]]
        - !Find [authentik_providers_oauth2.scopemapping, [scope_name, profile]]
      redirect_uris:
{redirect_uris}
  - model: authentik_providers_oauth2.oauth2provider
    state: must_created
    id: alternate-provider
    identifiers:
      name: {alternate_slug}
    attrs:
      authorization_flow: !Find [authentik_flows.flow, [slug, default-provider-authorization-implicit-consent]]
      invalidation_flow: !Find [authentik_flows.flow, [slug, default-provider-invalidation-flow]]
      client_type: confidential
      client_id: {alternate_client_id}
      client_secret: {alternate_client_secret}
      grant_types:
        - authorization_code
      include_claims_in_id_token: true
      issuer_mode: per_provider
      sub_mode: user_username
      signing_key: !Find [authentik_crypto.certificatekeypair, [name, authentik Self-signed Certificate]]
      property_mappings:
        - !Find [authentik_providers_oauth2.scopemapping, [scope_name, openid]]
        - !Find [authentik_providers_oauth2.scopemapping, [scope_name, email]]
        - !Find [authentik_providers_oauth2.scopemapping, [scope_name, profile]]
      redirect_uris:
{redirect_uris}
  - model: authentik_core.application
    state: must_created
    identifiers:
      slug: {slug}
    attrs:
      name: WORTH bank courtroom
      provider: !KeyOf bank-provider
  - model: authentik_core.application
    state: must_created
    identifiers:
      slug: {alternate_slug}
    attrs:
      name: WORTH bank alternate issuer
      provider: !KeyOf alternate-provider
"#,
        user_entries = user_entries,
        slug = fixture.slug(),
        client_id = fixture.client_id(),
        client_secret = fixture.client_secret(),
        alternate_slug = fixture.alternate_slug(),
        alternate_client_id = fixture.alternate_client_id(),
        alternate_client_secret = fixture.alternate_client_secret(),
        redirect_uris = redirect_uris,
    )
}
