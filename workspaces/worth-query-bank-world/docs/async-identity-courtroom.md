# Async Identity Courtroom

## Authentication boundary

The reference world uses Authentik OpenID Connect Authorization Code flow with
PKCE. Each authorization request carries state and nonce. Token validation
requires the configured issuer, audience, signature, expiry, and nonce before
the adapter may request Query principal admission.

The stable external identity is `(issuer, subject)`. Email, username, display
name, access-token text, fixture aliases, and Authentik database identifiers are
attributes rather than identity.

The fixture may use Authentik administration APIs to create applications,
providers, users, credentials, groups, redirect URIs, and teardown records. It
must still acquire identity through the supported OIDC flow and may not mint or
decode-and-trust a token locally.

## Courtroom TLS trust

Cold processes receive `trusted_root_pem` in their `cold_certification`
configuration. The courtroom CA signs the identity server certificate, and each
client adds that root to its TLS configuration. Missing or malformed roots fail
installation. The Selenium container imports the public root into its browser
NSS database before sessions start; administration HTTP receives the same root.
Browser sessions use temporary profiles and validate certificates and hostnames.
Ordinary deployments use their configured TLS roots.

## Process topology

The completed courtroom contains:

```text
Authentik issuer
      |
      +---- authorization-code + PKCE ---- user-node process A
      +---- authorization-code + PKCE ---- user-node process B
      +---- authorization-code + PKCE ---- user-node process N
                                                |
                                                +---- TCP ---- bank server
```

The bank server is the only authoritative application server. Every user node
has its own OS process, async runtime, dynamically selected listener, OIDC
redirect boundary, bounded queues, and authenticated session. User nodes are
clients/proxies and cannot cache, manufacture, or reconstruct bank authority.

## Dynamic participants

Every run provisions and discovers:

- two unrelated personal customers;
- one business owner, initiator, approver, and viewer;
- a teller and an auditor with different institution scopes;
- one principal with both customer and employee relationships;
- the institution, operational cash/settlement accounts, customer accounts,
  business accounts, external-principal mappings, and role relationships; and
- independent client registrations or exact redirect registrations for all
  user-node listeners.

Product code contains none of those identities.

## Hostile scenarios

The courtroom must prove concurrent transfers competing for funds, idempotent
retry after response loss, distinct-user business approval, live revocation,
permission-grant invalidation, relevant stale reads, irrelevant concurrent
mutation, process disconnect/crash, queue saturation, token expiry, malformed
token denial, and unknown or ambiguous principal mapping.

Authentication success is not authorization. Authorization is not commit
authority. Disconnect and response loss do not imply rollback.
