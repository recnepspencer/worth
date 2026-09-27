# File Composition Laws

1. A file is one named semantic responsibility, not an editing workspace. Its filename and parent topology must predict its contents, exclusions, public entry point, and subordinate logic. Catch-all files or modules such as `helpers`, `common`, `util`, `utils`, `shared`, `logic`, `manager`, and `processor` are forbidden because they name no responsibility. The reader should descend from entry point to orchestration, semantic steps, and mechanics without discovering an unordered private API. Correct framework usage does not excuse collapsed responsibility.

2. A nontrivial function is either one semantic step or an orchestrator of named steps. An orchestrator may coordinate broadly but must read as the operation's table of contents; it may not inline classification, validation, transformation, persistence, effects, diagnostics, and result construction across mixed abstraction levels.

3. Every domain predicate, classification, policy branch, lifecycle distinction, and numerical assumption must have a semantic name and produce a named fact, result variant, or proof-bearing type. Downstream code consumes that result rather than re-reading the raw condition. Unnamed branching is hidden policy.

4. A function's explicit inputs, output, failure topology, and declared effects must describe its full semantic responsibility. Hidden semantic context, undeclared mutation, ambient authority, or I/O behind a pure-looking signature is composition fraud and requires a visible boundary.

5. Classification, validation, eligibility, and planning must complete before the first effect unless an explicit transaction governs the sequence. Once mutation begins, the function must visibly remain in an effect phase; fallible preparatory work may not be interleaved with irreversible application.

6. Success, advisory, denial, cancellation, partial execution, cleanup, and recovery paths require the same semantic decomposition as the happy path. Cleanup mechanics and error translation must not obscure which phase failed, which effects occurred, or what authority remains.

7. Awaiting, spawning, callback registration, lock acquisition, transaction boundaries, retries, and handoff to another executor are semantic control-transfer points. They must appear in orchestration or be named by a function whose contract exposes the transfer; generic helpers may not hide them.

8. Placement follows semantic radius. Logic remains inline while obvious and local, moves to a private function when it names one step, to a child module when steps form a subordinate responsibility, to a sibling when they become a peer responsibility, and to shared infrastructure only under shared semantic authority. Types, constants, fixtures, and helpers follow the same rule. Extraction must increase predictability, not merely reduce length.

9. Comments explain why; they do not supply missing structure. Section headers and block comments cannot create boundaries. If a block needs a responsibility label to be understood, encode that label as a function, type, child module, or sibling file.

10. File order descends from meaning to mechanics: public capability, primary orchestration, semantic steps, local proof types, then mechanical support. Authoring chronology is never structure.

11. Local names must preserve semantic role, origin, phase, authority, scope, and truth status to the degree required at their visibility radius. Framework names, generic categories, generic verbs, numbered suffixes, and temporal adjectives are insufficient when the code relies on a stronger distinction. Shortening is legal only when meaning remains visually undeniable.

12. A function name is a contract for abstraction level, effects, and result. High-level names may orchestrate but not expose mechanics; mechanical names may implement but not decide policy. A body that performs work its name does not predict is miscomposed.

13. Public facade and export files may aggregate capabilities but must not implement domain behavior, workflow branching, persistence, or diagnostic construction. A facade that implements becomes hidden topology.

14. Tests obey production composition. Each test file owns one behavior, scenario, or invariant family; setup, action, observation, and assertion remain structurally legible. Shared fixtures may remove ceremony but must not hide which responsibility built the world or failed the proof.

15. Line limits detect decay but do not define cohesion. A file is healthy when it can be reviewed as one idea, deleted with its responsibility, and extended at an obvious location without preserving unrelated behavior. Small files can still be incoherent; large mechanically uniform artifacts are valid only through explicit exemption.
