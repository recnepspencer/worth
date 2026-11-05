#!/bin/bash
# Apply the owner-vocabulary batch: primary-graph families first, then the domain families,
# then family F's two refusal translators into request_execution::execution_stop.
# Run from the repository root on a clean tree. Items named in the seam designs and in
# held-back.md stay where they are (their packets move them).
set -euo pipefail
L=workspaces/worth-query/crates/worth-query-execution/src/lib.rs
DC=workspaces/worth-query/crates/worth-query-execution/src/domain_computation
V=plans/query-split/stage2/ownership/vocabulary
R=plans/query-split/stage2/ownership/rerun
D=plans/query-split/stage2/seams
HELD=$V/held-back.md

skeleton() {  # moves.json parent-dir level-name: the vocabulary module and one empty child per family
python3 - "$1" "$2" "$3" <<'PY'
import json, os, sys
moves = json.load(open(sys.argv[1])); P = sys.argv[2]; level = sys.argv[3]
fams = sorted({m['to'].split('::')[-1] for m in moves})
os.makedirs(P + '/vocabulary', exist_ok=True)
with open(P + '/vocabulary.rs', 'w') as f:
    f.write("//! Each family's vocabulary: the refusals, observations, identities and evidence its\n"
            "//! operations exchange, held below every family so one family can name another's without a\n"
            "//! cycle. Each child holds one family's items, moved whole from that family.\n")
    for fam in fams:
        f.write(f"pub(crate) mod {fam};\n")
for fam in fams:
    open(P + f'/vocabulary/{fam}.rs', 'w').write(
        f"//! `{fam}`'s vocabulary, moved whole from `{level}::{fam}`: its root file's items here, one child per other source file.\n")
s = open(P + '/mod.rs').read()
if 'mod vocabulary;' not in s:
    s = s.rstrip('\n') + '\npub(crate) mod vocabulary;\n'
open(P + '/mod.rs', 'w').write(s)
PY
}

moves() {  # simulation.json prefix out.json: the simulated moves as unit_move.py input, held names dropped
python3 - "$1" "$2" "$3" "$HELD" <<'PY'
import json, re, sys
d = json.load(open(sys.argv[1])); prefix = sys.argv[2]
held = set(re.findall(r'`(\w+)`', open(sys.argv[4]).read()))
kept = [{'item': prefix + m['item'], 'to': prefix + 'vocabulary::' + m['to'].split(':')[1]}
        for m in d['moves'] if m['item'].split('::')[-1] not in held]
json.dump(kept, open(sys.argv[3], 'w'), indent=0)
print(sys.argv[3], len(kept), 'moves')
PY
}

level() {  # parent stack prefix parent-dir level-name
  python3 scripts/query_split/vocabulary_simulation.py $L "$1" --impl-limit 600 --host-order "$2" \
    --exclude-designed $D/contracts.md $D/merges.md $D/proposal-03-plan.md $HELD --out $V > /dev/null
  moves $V/"$5"/simulation.json "$3" $V/moves-"$5".json
  skeleton $V/moves-"$5".json "$4" "$5"
  python3 scripts/query_split/unit_move.py $L $V/moves-"$5".json --report $V
}

level domain_computation::primary_graph $R/stack-primary_graph-draft.json \
  domain_computation::primary_graph:: $DC/primary_graph primary_graph
level domain_computation $R/stack-domain_computation-draft.json \
  domain_computation:: $DC domain_computation
python3 scripts/query_split/unit_move.py $L $V/moves-family-f.json --report $V
python3 scripts/test_topology/src_court_doors.py items --root .
