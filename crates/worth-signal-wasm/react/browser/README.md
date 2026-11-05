# Worker and React delivery matrix

Build and prepare the package once, before starting runtime tests. This also
rebuilds native changes instead of retaining an older WASM binary:

```powershell
./scripts/wasm/publish-worth-signals-wasm.ps1 -SkipPublish -RepositoryUrl https://github.com/recnepspencer/worth.git
node scripts/wasm/prepare-worth-signals-react-browser.mjs --react=19.2.5 --port=4192
```

Open the printed production and development URLs and click **Run checks**.
Repeat with `--react=18.3.1 --port=4194`. Development uses React Strict Mode.
`?order=worker` reverses deployment order. Each scenario owns a fresh runtime;
Worker construction is observed to verify the requested placement. The browser
consumer installs an npm tarball, with explicit emitted WASM and worker URLs.
The preparation script consumes the existing package without rebuilding the
shared stage, so running worker tests cannot lose generated modules mid-run.

| Scenario | Protected behavior |
| --- | --- |
| Single node and 64-node chain | Sequential delivery, burst convergence, no visible regression during worker catch-up |
| Render/subscribe race | Layout-time writes cannot leave the first subscriber stale |
| Shared subscribers and remount | Shared watch ownership, sibling survival, fresh remount, cleanup |
| Transaction | Coordinated input changes reach a computed React view |
| Resource lifecycle | Delayed fulfillment, refresh continuity, rejection, recovery, patch |
| Router and resource | Initial admission, push/pop/replace, shared route resource, isolation from a late prior-route load |
| Denied navigation | Redirect and not-found events render without advancing admitted route truth |
| Form | Draft edits, subsequent source updates, clearing the overlay |
| Disconnected control | Lost subscription must leave stale DOM and be detected |

Assertions use DOM MutationObserver delivery, with no polling runtime reads,
`act`, forced rerenders, or authored-work drains. Resource drains occur only
after assertions, during teardown. Startup, warm action-to-DOM, and command
acknowledgement are separate measurements. DOM timing does not measure physical
display paint. Request counts observe messages without changing delivery.
Timings are telemetry for this workload, not portable pass/fail thresholds or a
claim that worker placement is faster for every operation. JSON reports are
written under ignored `target/react-browser-results/`.
