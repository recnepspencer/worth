# Construction-input observations in closed testkits

`setup_observation.rs` is included privately by existing testkit owners. These
normal support libraries are compiled for their courts; `cfg(test)` would erase
the hooks because the owners declare `lib.test=false`. The source is not included
by production owners. No feature, dependency, production behavior or fixture
sharing path is added.

The optional `WORTH_TEST_COST_SETUP_DIRECTORY` destination uses the existing
schema-1 `test-cost-setup` / `construction-started` event consumed by
`telemetry.setup_summary`. `NEXTEST_BINARY_ID` identifies the observing binary.
When the destination is absent, the recipe closure does not execute. Builders
ignore observation I/O errors. Files are created separately per process, owning
module and sequence; independent included copies cannot corrupt a shared append
stream. A missing observation is unknown, not zero repeated construction.

Fingerprints are canonical, typed, length-prefixed input bytes rendered as hex,
not a probabilistic hash. Field order is canonicalized by name, duplicate/empty
names refuse, and type tags distinguish signed, unsigned, text and Boolean
inputs. Each recipe includes a revision. The census groups by builder and
fingerprint, so identities remain distinct between builders. Fingerprints
describe construction inputs only; generated identities and live mutable state
cannot be shared on that evidence.

| Owning testkit | Observed recipe | Complete variable input set |
|---|---|---|
| `worth-testkit-server` | `root-default-query-provider-server` | `register_worth_native`; concrete `TestWorkspaceProvider` and remaining configuration are fixed by revision 1 |
| `worth-query-testkit-installed-operation-fixture` | `query-default-installed-runtime` | no caller inputs; fixed `reversed=false`, `conflicting_duplicate=false`, default schema/executors and revision 1 |
| `worth-query-testkit-installed-operation-fixture` | `query-installed-geometry-package` | `reversed`, `conflicting_duplicate`; remaining declaration is fixed by revision 1 |
| `worth-query-bank-testkit-server` | `bank-parallel-history-seeded-world` | `seed`; names, balances, memberships and funding derive from it in revision 1 |
| `worth-query-bank-testkit-server` | `bank-default-auth-ordinary-read-world` | `scenario`, `unrelated_accounts`, `pending_payment_count`; default approval authentication, one approver and payment 900 are fixed and included explicitly in revision 1 |

Unknown recipes remain unknown: arbitrary Root workspace providers, Query custom
packages/schemas and authority-bearing request handles, Bank supplied approval
authentication configurations, and other uninstrumented recipes. A geometry
package observation measures that declaration, not its enclosing complete live
workspace. No full-world equivalence is inferred from a smaller recipe.

Closed-owner search: `rg -l` for the three exact package names across governed
Root/Query/Bank/Store `Cargo.toml` families finds only their own manifests,
`worth-query-testkit-query`, `worth-query-bank-testkit-courtroom`,
`bank-courtroom`, and named court consumers. The shared Rust module is private;
no new public facade type is introduced.

The six std-only transport tests cover disabled laziness, typed canonical
identity, field boundaries, duplicate refusal, I/O refusal and separate complete
observations. Actual owning court cases provide the construction observations;
they remain separate bounded captures from accepted full membership evidence.
