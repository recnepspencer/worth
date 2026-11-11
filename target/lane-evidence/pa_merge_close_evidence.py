from pathlib import Path
from collections import Counter
import subprocess, json, hashlib, re

root = Path.cwd()
e = root / 'target/lane-evidence'
def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()
head = git('rev-parse', 'HEAD')
source = []
for name in git('diff', '--name-only', '589f6fce9c', '--', '*.rs', '*.py').splitlines():
    p = root / name
    if not p.is_file() or name.startswith('target/'):
        continue
    data = p.read_bytes().replace(b'\r\n', b'\n')
    source.append({'path': name, 'sha256_git_lf': hashlib.sha256(data).hexdigest(),
                   'lines': len(data.splitlines())})
assert all(x['lines'] <= 400 for x in source if x['path'].endswith('.rs'))
provenance = {'reviewed_source_head': head, 'source_since_merge': source,
    'incoming_ancestor_exit': subprocess.run(['git','merge-base','--is-ancestor','2a51552fc5','HEAD']).returncode,
    'base_ancestor_exit': subprocess.run(['git','merge-base','--is-ancestor','7b35408444','HEAD']).returncode,
    'ui_diff_from_base': git('diff','--name-only','7b35408444','--','workspaces/worth-ui'),
    'input_reuse': [
      'First Tier2 logs qualify merge source 589f6fce9c, not a claim of green final source.',
      'Initial Rust repair batch dirty contents have LF hashes in pa-merge-repair-rust-source-hashes.json.',
      'Native production last changed in 7a12fb36a9. Later Native edits are test providers only; Bank default-stack parent evidence remains applicable.',
      'Query final metadata fixture evidence: round6/query 11 passing; consumer evidence: followup/fixture 3 passing.',
      'Final boundary correction only changes member-custody provider placement and removes five Topology funding edits. boundary-closure Root/fixture checks and exact 4/5 cases cover it.',
      'Final lint input changes are covered by closure Native/Topology lints. Other final scoped lint inputs remain unchanged.',
      'Evidence and generated telemetry commits change no Rust compiler inputs.'
    ]}
assert provenance['incoming_ancestor_exit'] == provenance['base_ancestor_exit'] == 0
assert not provenance['ui_diff_from_base']
(e/'pa-merge-final-source-provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')

# Index owned Cargo invocations and their native commands once; wrapper aliases
# and compiler-plan declarations have already been deduplicated.
census = json.loads((e/'pa-merge-actual-operation-census.json').read_text())
lanes = []
def inspect(path):
    data = (root/path).read_text(encoding='utf-8', errors='replace')
    outcomes = {}
    ignored = 0
    for line in data.splitlines():
        if not line.startswith('{'):
            continue
        try: obj = json.loads(line)
        except ValueError: continue
        if obj.get('type') == 'test' and obj.get('event') in ('ok','failed'):
            outcomes[obj['name']] = obj['event']
        if obj.get('type') == 'suite' and obj.get('event') in ('ok','failed'):
            ignored += obj.get('ignored', 0)
    exits = re.findall(r'^(?:Exit code|EXIT):\s*(-?\d+)\s*$',data,re.M)
    ends = re.findall(r'^(?:End|END):\s*(.+)$',data,re.M)
    status = re.findall(r'^(?:Git status --short empty|CLEAN):\s*(.+)$',data,re.M)
    counts = Counter(outcomes.values())
    body_counts = {'passed': counts['ok'], 'failed': counts['failed'], 'ignored_metadata': ignored} if outcomes else None
    if body_counts is None:
        summaries = re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored',data)
        if summaries:
            if 'default-stack.log' in str(path): summaries = summaries[-1:]
            body_counts = dict(zip(('passed','failed','ignored_metadata'),(sum(int(s[i]) for s in summaries) for i in range(3))))
    return {'end': ends[-1] if ends else None, 'exit': int(exits[-1]) if exits else None,
            'clean': status[0] if status else None, 'counts': body_counts}, outcomes
for row in census['invocations']:
    info, _ = inspect(row['logs'][0])
    lanes.append({**row, **info})
supplemental = [
 ('legacy-fixture-lane','pa-merge-fixture-tier2-driver.log'),
 ('diagnostic-child','pa-merge-preimage-child-exit.log'),
 ('diagnostic-child','pa-merge-memory-child-exit.log'),
 ('diagnostic-child','pa-merge-fixture-retention-child.log'),
 ('diagnostic-child','pa-merge-fixture-retention-owned.log'),
 *[('python-unit-test','pa-merge-unit-plan-'+suffix+'.log') for suffix in
   ('tests','example-tests','mixed-example-tests','paired-example-tests')]]
for kind,name in supplemental:
    path='target/lane-evidence/'+name
    data=(root/path).read_text(errors='replace')
    info,_=inspect(path)
    if kind=='python-unit-test':
        n=int(re.search(r'Ran (\d+) tests',data).group(1))
        info['counts']={'passed':n,'failed':0,'ignored_metadata':0}
    if kind=='legacy-fixture-lane':
        fixture=json.loads((e/'pa-merge-fixture-tier2/fixture.json').read_text())
        info['counts']={'passed':fixture['passed'],'failed':len(fixture['failed']),'ignored_metadata':0,'incomplete':2}
        info['child_chunks']=len(fixture['chunks'])
        info['child_detail']='target/lane-evidence/pa-merge-fixture-tier2/fixture.json'
    lanes.append({'kind':kind,'head':re.search(r'^HEAD: (.+)$',data,re.M).group(1),
      'start':re.search(r'^START: (.+)$',data,re.M).group(1),
      'command':json.loads(re.search(r'^COMMAND: (.+)$',data,re.M).group(1)),
      'logs':[path],**info})
(e/'pa-merge-lane-index.json').write_text(json.dumps({'source_head':head,'counts':census['counts'],'lanes':lanes},indent=2)+'\n')

focused = [
 'pa-merge-repair/root/run.log', 'pa-merge-repair/query/run.log',
 'pa-merge-repair/fixture/run-final.log', 'pa-merge-repair/store/run.log',
 'pa-merge-followup/root/run.log', 'pa-merge-followup/query/run.log',
 'pa-merge-followup/fixture/run.log', 'pa-merge-followup/store/run.log',
 'pa-merge-round6/query/run.log','pa-merge-round6/store/run.log',
 'pa-merge-boundary-closure/root/run.log','pa-merge-boundary-closure/fixture/run.log']
latest = {}
for relative in focused:
    if not (e/relative).exists(): continue
    info, events = inspect('target/lane-evidence/'+relative)
    for name,event in events.items(): latest[name] = (event, 'target/lane-evidence/'+relative)
original = json.loads((e/'pa-merge-classified-failures.json').read_text())
final = []
for old in original:
    row = dict(old)
    found = latest.get(row['name'])
    if not found:
        found = next((value for name,value in latest.items() if name.endswith('$'+row['name'])),None)
    if row['class'].startswith('CLEANUP'):
        row.update(final_class=row['class'], final_state='retained red')
    elif 'post_ack_capture_refusal' in row['name'] or 'related_acknowledged_successor_repairs' in row['name']:
        row.update(final_class='CLEANUP-C1-post-ACK-deliberate-refusal', final_state='retained red',
            final_reason='Benign cold-history setup repaired; now reaches unchanged deliberate zero-memory post-ACK repair assertion. Actual refusal variant is not printed; do not invent it.')
    elif row['class']=='HAPPY-fixture-world-checkpoint-composition':
        row.update(final_class='CLEANUP-C2-cold-member-read-bound',final_state='retained red',
            final_reason='Real World composition and all 217 caller-owned snapshot releases restore graceful close. Journey now reaches exact member-read bound 1 versus 0. Later suffix not reached.')
    elif row['class'].startswith('HAPPY_HELD'):
        row.update(final_class=row['class'],final_state='held HAPPY')
    elif 'example/application_correction$<main>' in row['name']:
        row.update(final_class=row['class'],final_state='green',final_log='target/lane-evidence/pa-merge-repair/query/application_correction.log')
    elif found:
        row.update(final_class=row['class'],final_state='green' if found[0]=='ok' else 'blocked HAPPY',final_log=found[1])
    else:
        raise AssertionError(('No focused evidence',row['name']))
    if found: row['final_log']=found[1]
    final.append(row)
totals=Counter(r['final_state'] for r in final)
(e/'pa-merge-final-families.json').write_text(json.dumps({'source_head':head,'totals':totals,'outcomes':final},indent=2)+'\n')
table = ['# Final P-A merge family status','',f'Source HEAD: `{head}`. Incoming: `2a51552fc5`; base: `7b35408444`.',
 '','The first-pass classification is immutable in `pa-merge-classified-failures.json`. This table follows each original outcome to its latest focused evidence; it is not a rerun of Tier2.',
 '', '|Final class|State|Original outcomes|','|---|---|---:|']
for (family,state),n in sorted(Counter((r['final_class'],r['final_state']) for r in final).items()):
    table.append(f'|{family}|{state}|{n}|')
table += ['',f'Totals: {dict(totals)}.',
 '', 'No ignored body, query-scale, exhaustive oracle or Store scenario was executed. Initial suite ignored numbers describe metadata, not execution. Four interrupted cumulative/fault/growth campaigns remain incomplete reds.',
 '', 'Five Topology capture setups remain HAPPY blocked. No frozen root allows their added ExecutionResourceLease factory. The unconstitutional funding slice was removed; all original assertions and installed/serving policy remain. The separately observed ordinary principal-resolution WorkExhausted under the earlier funded capture path is also unresolved; it is not relabelled setup or CLEANUP.',
 '', 'Other held HAPPY: standalone Complete/ACK capture-base binding (P-A contract); worker-only World/Query checkpoint accounting (P-C/P-A); durable sibling and released ProductBasisStale (P-C). Root strict lint retains 32 Native dead-code diagnostics pending claimed-contract retirement ruling. Boundary retains 12 findings (P-A 3, P-C 9). See pa-merge-held-owner-boundaries.md and pa-merge-native-retirement-boundary.md.',
 '', 'Default-stack Bank assessment and transfer parents: 2/0/0. Consumer restored outputs plus both affected helper callers: 3/0/0. Four member-custody controls: 4/0/0. Metadata transition focused set: 11/0/0. Exact per-case source/log mapping is in pa-merge-final-families.json.']
(e/'pa-merge-families.md').write_text('\n'.join(table)+'\n')
cleanup=['# P-A merge cleanup reds','',f'Latest source `{head}`. Each court is retained. Exact original and focused failures remain in the linked JSON and originating logs.','']
for row in final:
    if not row['final_class'].startswith('CLEANUP'): continue
    cleanup += ['## '+row['name'],'',row['final_class']+': '+row.get('final_reason',row['reason']),'',
                'Evidence: `'+row.get('final_log','pa-merge-classified-failures.json')+'`. Original assertion/stage:','',
                '```text', (row['failure'] if isinstance(row['failure'],str) else json.dumps(row['failure'],indent=2)).strip()[:7000], '```','']
(e/'pa-merge-cleanup-reds.md').write_text('\n'.join(cleanup)+'\n')
md=['# P-A merge lane evidence index','',f'Reviewed source: `{head}`. All commands, HEADs, start/end times, dirty flags and raw paths are in pa-merge-lane-index.json and the logs. First Tier2 is at 589f6fce9c; later evidence applies only to its named inputs.',
 '', 'Counts below are actual distinct invocations, including failed attribution/tool attempts. Metadata/list/declaration operations do not run test bodies. A missing test count is N/A, never a green zero-test claim.',
 '', 'Operation counts: `'+json.dumps(census['counts'],sort_keys=True)+'`.',
 '', '|Kind|HEAD|Exit|Pass/fail/ignored metadata|Log|','|---|---|---:|---|---|']
for row in lanes:
    if row['kind'] in ('cargo-metadata','nextest-list','cargo-tree'): continue
    count=row['counts']; counts='/'.join(str(count[k]) for k in ('passed','failed','ignored_metadata')) if count else 'N/A'
    md.append('|'+ '|'.join((row['kind'],(row.get('head') or 'see log')[:10],str(row['exit']),counts,row['logs'][0].replace('\\','/')))+'|')
md += ['', 'Original aggregate suite/body counts: pa-merge-initial-lane-counts.json. Zero-selected Query/Bank compile-fail and Bank determinism are UNRUN, not green. Aggregate gate exits propagate a workspace failure even for suites whose bodies passed.',
 '', 'Focused nextest selections execute no ignored body; nextest emits ignored metadata from other cases in the same binaries. Genuine consumer-root ran at merge source; no final-source main-stack claim is made. application_correction is a separate changed-contract standalone proof.',
 '', 'Formal model inputs unchanged: reused lifecycle-store-formal-toolchain.log; no formal rerun.',
 '', 'Primary validation total: 22 cargo checks, 16 clippy passes/attempts; 17 focused nextest builds plus one legacy fixture no-run build, 3 production/bin builds and 2 IR diagnostic rustc invocations; 30 nextest run invocations. Actual native executions, doctests and the one frozen-door test are separately enumerated.']
md += ['', 'Additional non-Cargo lane evidence is indexed above: one legacy fixture lane (308 child chunks, 578 passed/19 failed/2 incomplete), four diagnostic child probes, and four Python unit-plan runs. Final Python suite is21passed; the earlier22case draft modeled the compiler graph incorrectly and is superseded. Child fragment logs belong to their parent lane header; per-child timestamps were not recorded and are not invented.',
 '', 'Test-run accounting: 30 nextest invocations + one legacy fixture lane + 11 direct court/example commands (10 native and one cargo run) + four diagnostic children + four documentation commands + four Python unit suites + one frozen-door test =55 top-level test/probe commands. The legacy lane launches308 chunks; child processes spawned inside courts are not counted again. Four native phase_telemetry commands are bookkeeping, not tests.']
(e/'pa-merge-lane-index.md').write_text('\n'.join(md)+'\n')
print('Final outcome totals:',dict(totals))
print('Own source files:',len(source),'max Rust lines:',max(r['lines'] for r in source if r['path'].endswith('.rs')))
print('Lane index rows:',len(lanes))
