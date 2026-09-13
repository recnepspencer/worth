# Contributing to WORTH

Issues, design criticism, reproductions, documentation improvements, and code
contributions are welcome.

Before working on code, read [AGENTS.md](./AGENTS.md) and every document under
[`_docs/coding_guidelines`](./_docs/coding_guidelines). WORTH treats authority,
truth ownership, phase progression, physical placement, performance, and test
evidence as mechanically enforced architecture rather than local convention.

## Pull requests

Keep a pull request centered on one coherent responsibility. Explain:

- the behavior or boundary being changed;
- the authority owner and destination topology;
- the focused tests or other evidence that protect the change; and
- any compatibility, recovery, or performance consequence.

Generated `AGENT_CONTEXT.md` files must be updated through repository tooling,
not edited manually.

## Contributor grant

WORTH uses a commercial/source-available licensing model. To preserve the
project's ability to issue commercial licenses and eventually convert released
versions to Apache-2.0, every contributor must agree to the
[Contributor License Agreement](./CONTRIBUTOR-LICENSE-AGREEMENT.md).

The pull-request template records that agreement. Do not submit a contribution
on behalf of an employer or another rights holder unless you are authorized to
make the grant.

## License

Contributions accepted into this repository are distributed under the same
[Business Source License 1.1](./LICENSE) terms as the repository and are also
covered by the contributor grant.
