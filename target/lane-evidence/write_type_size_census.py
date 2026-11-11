"""Report observed layouts and explicitly bounded source-use counts."""
import collections,json,re
from pathlib import Path
from tree_sitter import Language,Parser
import tree_sitter_rust
e=Path('target/lane-evidence');r=Path.cwd(); parser=Parser(Language(tree_sitter_rust.language()))
source=json.loads((e/'type-census-source.json').read_text());definitions=source['definitions']
by_name=collections.defaultdict(list)
for definition in definitions:by_name[definition['name']].append(definition)
def descendants(node):
 yield node
 for child in node.named_children:yield from descendants(child)
def owned_names(node):
 if node is None:return set()
 if node.type in ('reference_type','pointer_type','function_type','abstract_type'):return set()
 if node.type=='generic_type':
  outer=node.child_by_field_name('type')
  if outer and outer.text.decode().split('::')[-1] in ('Box','Arc','Weak','Vec','HashMap','HashSet','BTreeMap','BTreeSet','ExecutionArray'):return set()
 names={node.text.decode()} if node.type=='type_identifier' else set()
 for child in node.named_children:names.update(owned_names(child))
 return names
uses=collections.defaultdict(set)
for function in source['functions']:
 match=function['matches'][0];raw=match['body'].encode();tree=parser.parse(raw)
 fn=next(n for n in descendants(tree.root_node) if n.type=='function_item')
 names=owned_names(fn.child_by_field_name('return_type'))
 params=fn.child_by_field_name('parameters')
 if params:
  for param in params.named_children:
   if param.type=='self_parameter' and b'&' not in param.text:names.add('Self')
   else:names.update(owned_names(param.child_by_field_name('type')))
 for node in descendants(fn):
  if node.type=='let_declaration':names.update(owned_names(node.child_by_field_name('type')))
  if node.type=='struct_expression':
   name=node.child_by_field_name('name')
   if name:
    pieces=re.findall(r'\b[A-Z][A-Za-z0-9_]*\b',name.text.decode());names.update(pieces[:1])
 receiver=match['impl'].split('<',1)[0].split('::')[-1].strip()
 if 'Self' in names:names.remove('Self');names.add(receiver)
 for name in names:
  candidates=by_name.get(name,[])
  if not candidates:continue
  def score(d):
   common=0
   for a,b in zip(Path(d['file']).parts,Path(match['file']).parts):
    if a!=b:break
    common+=1
   return common
  best=max(map(score,candidates));chosen=[d for d in candidates if score(d)==best]
  if len(chosen)==1:uses[(chosen[0]['file'],name)].add(function['symbol'])
families={}; observations=[]
def family(head):
 return families.setdefault(head,dict(head=head,maximum=0,instances=set(),sizes=set(),source=set(),members=[],use_count=None,exemplar=''))
for workspace in ('root','query','store','legacy-layout'):
 rows=json.loads((e/('type-size-'+workspace+'-rows.json')).read_text())
 observations.extend(rows)
 groups=collections.defaultdict(list)
 for row in rows:groups[(row[1],row[2])].append(row)
 for row in rows:
  if int(row[5])<1024:continue
  head=row[4].split('<',1)[0];f=family(head);f['instances'].add(row[4]);f['sizes'].add(int(row[5]))
  if int(row[5])>f['maximum']:
   f['maximum']=int(row[5]);f['exemplar']=row[4]
   if not row[3]:
    fields=sorted((x for x in groups[(row[1],row[2])] if x[3]),key=lambda x:-int(x[5]))
    f['members']=[x[3]+': '+x[5] for x in fields[:3]]
  if not row[3]:f['source'].add(row[1]);f['use_count']=len(uses[(row[1],row[2])])
consumer=json.loads((e/'type-size-consumer-layout-rows.json').read_text())+json.loads((e/'type-size-bank-layout-rows.json').read_text())
for row in consumer:
 f=family(row[1].split('<',1)[0]);f['instances'].add(row[1]);f['sizes'].add(int(row[2]))
 if int(row[2])>f['maximum']:f['maximum']=int(row[2]);f['exemplar']=row[1]
compiler=json.loads((e/'compiler-type-layouts.json').read_text())
selected=[]
for index,row in enumerate(compiler):
 name=row['type']
 if row['bytes']<1024 or not (name.startswith(('worth_','bank')) or 'worth_' in name or 'bank_' in name or name.startswith('{closure@consumer_root')):continue
 selected.append(index);head=name.split('<',1)[0];f=family(head);f['instances'].add(name);f['sizes'].add(row['bytes'])
 if row['bytes']>f['maximum']:
  f['maximum']=row['bytes'];f['exemplar']=name;f['members']=[]
 if not f['members']:
  members=[];variant=''
  for member in row['members']:
   if match:=re.search(r'variant `([^`]+)`: (\d+) bytes',member):variant=match[1]
   if match:=re.search(r'field `([^`]+)`: (\d+) bytes',member):members.append((int(match[2]),variant+match[1]))
  f['members']=[label+': '+str(size) for size,label in sorted(members,reverse=True)[:3]]
explicit_reexports={
 'worth_query_bank_testkit_server::facade::recorded_correction::WorthQueryWorkflowOperation':'workspaces/worth-query/crates/worth-query-publication/src/application_entry/operation/workflow/mod.rs',
 'worth_query_bank_testkit_server::facade::recorded_correction::fixture::World':'workspaces/worth-query-bank-world/crates/worth-query-bank-testkit-server/src/ordinary_mutations/recorded_correction/fixture.rs',
 'worth_query_bank_testkit_server::facade::parallel_history::observation::World':'workspaces/worth-query-bank-world/crates/worth-query-bank-testkit-server/src/parallel_history/seeded_world.rs',
}
for head,f in families.items():
 if head in explicit_reexports:
  file=explicit_reexports[head];f['source'].add(file);f['use_count']=len(uses[(file,head.split('::')[-1])])
 name=head.split('::')[-1];candidates=by_name.get(name,[])
 if not f['source']:
  in_crate=[d for d in candidates if d['crate']==head.split('::')[0]]
  if in_crate:candidates=in_crate
  if len(candidates)>1:
   module='/'.join(head.split('::')[1:-1])
   exact=[d for d in candidates if d['file'].endswith('/src/'+module+'.rs') or d['file'].endswith('/src/'+module+'/mod.rs')]
   if len(exact)==1:candidates=exact
  if len(candidates)==1:f['source'].add(candidates[0]['file']);f['use_count']=len(uses[(candidates[0]['file'],name)])
ordered=sorted(families.values(),key=lambda f:(-f['maximum'],f['head']))
payload=[]
for f in ordered:
 f=dict(f,instances=sorted(f['instances']),sizes=sorted(f['sizes']),source=sorted(f['source']))
 payload.append(f)
(e/'type-size-families.json').write_text(json.dumps(payload,indent=2))
lines=['# Type-size census before the layout batch','',
 'Source: baseline HEAD `750f746567` plus retained unit-only census changes; correction measurements at committed HEAD `805533bb7f`. All measurements use the committed 1.99.1 compiler and dev profile. No runtime, proof, witness or authority value is constructed by a census test.','',
 'Root 3/0/49 (49 ignored metadata entries, none selected or executed); Query 3/0/0; Store 2/0/0; baseline consumer 1/0/0. Correction: Legacy, Bank and updated consumer 1/0/0 each; Native and World compiler layout builds execute no tests. Exact commands, HEAD, dirty status, times and exit codes are in `type-size-{root,query,store}.log` and `type-size-consumer-selected.log`. The first consumer preflight log records a two-target declaration refusal before compilation; the corrected selection has one test root.','',
 f'The owner-unit captures printed {len(observations):,} type/field observations; the real consumer and Bank guards printed {len(consumer)} concrete public carrier/return layouts. The compiler diagnostic retains {len(compiler):,} distinct rendered layouts, including {len(selected):,} layouts of WORTH data or wrappers containing it at least 1 KiB. Exact generic instances, alignment and members are in `compiler-type-layouts.json`; raw compiler output is in `type-size-consumer/compiler-layouts.log` and `type-size-{{native,world,legacy,bank,consumer}}-layout/compiler-layouts.log`. Correction driver refusals were artifact-selection metadata defects, not compiler/product defects; corrected logs preserve the successful compiles.','',
 'The table groups by rendered nominal constructor (generic arguments elided) and reports the maximum observed size, not an assertion that its instantiations share one layout. Aliases printed by the compiler remain separate labels unless a unit observation supplies their canonical type name. Closures with one displayed source span may have several reported layouts; every distinct layout is retained. Sizes can differ between unit cfg and dependency cfg. No padding subtraction or guessed private size is used.','',
 'The use count is an explicit-source lower bound over the 172 selected source functions (top 100 plus 12 per additional owner), deduplicated by symbol. It counts owning receivers, typed parameters/results/locals and named struct constructors; references and heap-indirected generic payloads are excluded. Inferred locals and implicit copies are not counted. A dash means the constructor has no unique definition attribution in this source inventory, not that it is absent or unused. This is not a complete inferred by-value count across all 1,658 frame functions.','',
 'Definition paths below are direct owner observations or unique declaration-name matches (scoped to the defining crate and exact module path when needed). Unresolved aliases/wrappers retain their exact compiler label and raw layout, rather than acquiring a fabricated source location. Private protected owner carriers are observed where their public containers require their layout; no protected source file is edited.','',
 '| Max bytes | Type constructor | Instances | Explicit uses | Definition / evidence | Largest observed fields |','|---:|---|---:|---:|---|---|']
for f in payload:
 path=', '.join(f['source']) or 'compiler layout; definition attribution unresolved'
 members='; '.join(f['members']).replace('|','\\|')
 label=f['head'].replace('|','\\|')
 lines.append(f'| {f["maximum"]:,} | `{label}` | {len(f["instances"])} | {f["use_count"] if f["use_count"] is not None else "—"} | {path} | {members} |')
lines+=['','## Explicit facade attribution','', 'Three ambiguous short names were resolved by their written re-export chains, without size subtraction: Bank recorded_correction exports Query Host application_entry; Host facade.rs:360 exports Publication facade application_entry; Publication application_entry/mod.rs:114 and operation.rs:34 export operation/workflow/mod.rs WorthQueryWorkflowOperation. Bank recorded_correction fixture World is declared in ordinary_mutations/recorded_correction/fixture.rs:12 through its explicit path mount. Parallel-history observation.rs:8 re-exports seeded_world::World, declared in parallel_history/seeded_world.rs:10. Bank facade.rs mounts mod.rs as server_world and re-exports it, accounting for the additional facade namespace.','', '## Coverage still requiring qualification','',
 'This inventory exhausts the recorded observations, not every possible generic instantiation. Source candidates not instantiated by these owner units or the real consumer remain explicit gaps. Do not label the source-type inventory exhaustive until those gaps are measured or routed. The compiler diagnostic resolves the private History Phase/Forking/Selecting layouts; the existing Memory host remains the real reference, with no artificial application factory.','',
 'Protected Native storage/checkpoint/root and World/Lifecycle/transition carriers remain owner-held. All remaining unobserved constrained candidates are listed below; a zero gap count is limited to the recorded source candidate plan. C2 owns residual frame/future ceilings; it is not a waiver of this census.','']
observed_names={row['type'].split('<',1)[0].split('::')[-1] for row in compiler}
observed_names.update(row[4].split('<',1)[0].split('::')[-1] for row in observations)
plan=json.loads((e/'type-census-plan.json').read_text())
missing=[row for row in plan['skipped'] if row['name'] not in observed_names]
for row in missing:lines.append('- Exact constrained layout not observed: `'+row['name']+'` in `'+row['file']+'`.')
(e/'type-size-census.md').write_text('\n'.join(lines)+'\n',encoding='utf8')
print('FAMILIES',len(payload),'LAYOUT RECORDS',len(selected),'EXPLICIT GAPS',len(missing),'MARKDOWN_BYTES',(e/'type-size-census.md').stat().st_size)
