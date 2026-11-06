# Track 6 — WASM size certification

Gate 6.0 / Track 6 oracles for the published `worth_signal_wasm_bg.wasm`.

## Reports

- `baseline-pre-track6.json` — pre-Track-6 publish artifact (**13,092,002** bytes)
- `post-track6.json` — after `release-wasm` + explicit `wasm-opt -Oz`

## Measure

```powershell
node scripts/wasm/measure-worth-signals-wasm-size.mjs
```

Package verify enforces magic bytes, `MAX_WASM_BYTES` from
`scripts/wasm/wasm_size_policy.mjs`, and path-leak needles.

For release 1.6.0, the accepted optimized extended-profile runtime is
11,707,918 bytes. Its release ceiling is 12,293,314 bytes (5% headroom),
still below the original 13,092,002-byte baseline. The Track 6 reports above
remain historical measurements of the earlier runtime.
