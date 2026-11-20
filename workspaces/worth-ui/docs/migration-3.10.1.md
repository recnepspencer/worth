# Milestone 3.10.1 Migration

> **Parked (2026-11-20). Doesn't build.** Worth UI is parked by user ruling: `worth-ui-query-binding` and `worth-ui-certification` still depend on the retired `worth-query` engine, so the `workspaces/worth-ui` workspace does not compile. Commands, compiled examples and compile-pass fragments on this page do not build or run until the migration to `worth-query-host` tracked in [#262](https://github.com/recnepspencer/worth/issues/262) lands.

Milestone 3.10.1 removed transitional routes that let downstream code observe
or call intermediate runtime phases. Migrate to the named audience instead of
adding a compatibility wrapper.

## Removed Routes

- `WorthUiBuilder` and `WorthUiAppBuilder` aliases were removed.
- Inherent `WorthUiActiveApplicationSession::execute_framework_turn` is no
  longer a product call.
- Product `facade::mounted` and lane-specific `facade::runtime` midpoint
  exports were removed.
- The product DSL package projection facade was removed.
- Loose artifact-input and declaration-vector preparation routes were removed.

## Audience Replacements

| Removed use | Replacement |
| --- | --- |
| old builder aliases | `worth_ui::facade::app::WorthUi::app()` returning `WorthUiApplicationBuilder` |
| ordinary raw framework turn | `WorthUiActiveApplicationSession::execute_mounted_frame(...)` |
| mounted midpoint imports | typed outcomes re-exported by `worth_ui::facade::app` |
| authored DSL types through product runtime | import authored types from `worth-ui-dsl`; use `facade::source` only for transport and ingress |
| loose runtime preparation | pass `WorthUiRustAuthoredArtifactInput` or one complete `WorthUiWatchedCandidateSubmission` to the application builder |
| runtime implementation inspection | `worth_ui::facade::inspection` queries and receipts |
| direct Query state in UI | `worth_ui::facade::query_binding` and `worth-ui-query-binding` installed references |

Certification can reach selected private transitions through
`worth-ui-test-support`. That authority is deliberately unavailable to product
code and must not be copied into examples.

## No Compatibility Lane

Do not recreate a removed name as a type alias, extension trait, wrapper,
feature-gated re-export, or dead forwarding module. The compiler failures are
part of the migration contract: they force callers onto the owner that can
preserve application, host, mounted, and Query authority coherently.

If a use case cannot be expressed through the named product facades, treat it
as a missing product contract. Do not solve it with a deep runtime import.
