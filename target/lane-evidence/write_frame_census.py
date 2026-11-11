import collections,json,re,datetime
from pathlib import Path
e=Path(__file__).resolve().parent
dev=json.loads((e/'whole-frame-dev-identified-dedup.json').read_text())
release=json.loads((e/'whole-frame-release-identified-dedup.json').read_text())
example_names={'invariant_reads','required_feature','shared_execution'}
example_maxima={r['symbol_path']:max(s['bytes'] for s in r['symbols'] if s['binary'] in example_names) for r in dev['functions'] if any(s['binary'] in example_names for s in r['symbols'])}
example_bands={'>=128KiB':sum(v>=131072 for v in example_maxima.values()),'64-128KiB':sum(65536<=v<131072 for v in example_maxima.values()),'32-64KiB':sum(32768<=v<65536 for v in example_maxima.values())}
def protected(n):
 if n.startswith('worth_runtime_world::'):return 'P-C World (protected)'
 if n.startswith('worth_relational::') and any(t in n for t in ('::storage::','::durability::','::root_','::root::')):return 'P-A Native storage/checkpoint/root (protected)'
 if n.startswith('worth_query_host::application_home::') and any(t in n for t in ('::transition','::opening::','::close::','::failure_close::')):return 'P-C Host lifecycle (protected)'
 return 'integration'
def probable(n):
 if any(t in n for t in ('::closure$', '::closure_env$','::{closure')):return 'owned closure capture / large by-value result'
 if any(t in n for t in ('_court_','_consumer_root::','_testkit_')) or n.split('::')[0] in ('invariant_reads','shared_execution','required_feature'):return 'test/example locals and repeated by-value operational results'
 if n.startswith('worth_store'):return 'by-value physical recovery phase/result; large enum variants'
 if n.startswith('worth_signal::'):
  if '::owner::' in n or '::owner_port_slots' in n:return 'by-value BranchState/owner transfer (AuthorityState contains SignalGraph)'
  return 'by-value evaluation storage/partition; large conditional slot/result variants'
 if n.startswith('worth_runtime_bridge::'):return 'by-value conditional runtime/Signal owner transfer and result variants'
 if '::history::edit::' in n or '::operation::' in n:return 'large inline operation/phase enum; ownership-returning Result repeats it'
 if any(t in n for t in ('::installation::','::builder::','::construction::','::application_home::')):return 'large by-value runtime/installation phases and refusal variants'
 if n.startswith('worth_relational::'):return 'by-value Native authority/validation/publication phase carriers'
 if any(t in n for t in ('::producer::','::application_output_demand::','::managed_run::')):return 'inline conditional admission/output/cleanup variants and by-value handoffs'
 return 'large by-value operation evidence/results (source attribution still required)'
lines=['# Whole-binary static frame census','',
'Source HEAD: `dae78e6ff76560ebbbc50a03c165bfc6be8f5055` (clean). No dev build was made for this census. Existing artifacts are identified below by exact path and modification time; their source inputs are not all certified at this HEAD. The release comparison was built once at that clean HEAD with the committed profile.', '',
'Method: every x64 PE `.pdata` runtime-function entry, decoded `UNWIND_INFO`, including saved registers; return addresses excluded. All inspected dev headers are version 1; none is chained. Symbol attribution uses exact RVAs and raw PDB public/procedure records. Rust linkage names take precedence over lossy display aliases; distinct raw linkage aliases remain. Rust v0 defining impl paths establish crate ownership, including foreign receiver types. Generic arguments and crate disambiguators are removed; function arrows do not terminate generic arguments. Trait receiver qualification remains. Each owner path retains its largest measured frame across all seven binaries.', '',
f"Named owner totals: **{len(dev['functions'])}** functions >=32 KiB: **{dev['bands']['>=128KiB']} >=128 KiB**, **{dev['bands']['64-128KiB']} 64–128 KiB**, **{dev['bands']['32-64KiB']} 32–64 KiB**. No Rust demangle failures.", '',
'All 572 dev unwind entries initially missing PDB public/procedure names were resolved from retained COFF objects. Exact PDB section contributions identify the originating module; section length and auxiliary checksum select retained COFF sections; every nonrelocation byte matches the executable. Across every matching candidate version the normalized owner path agrees (zero ambiguous owner paths). Relocation targets are not asserted as program-equivalence proof. This census identifies static owner functions; it does not claim full object/program equivalence. No nearest-symbol inference or new build was used. Full match provenance remains in `whole-frame-dev-identified.json`; `frame-census-unattributed.md` records zero remaining dev gaps.', '',
'All PE RSDS GUID/age pairs match their PDB identity stream; both file SHA-256 hashes are retained in the identified JSON. The committed consumer profile omits Query execution/topology debug information and test profiles omit line tables. No profile/symbol settings were changed.', '',
'Probable causes below are source-level family classifications, not compiler local-variable layouts. The PDBs expose no Rust UDT layouts (the >=1 KiB type enumeration found only Windows `_CONTEXT`). Precise allocation/lifetime attribution remains a required design step before edits; no claim is made that an arbitrary box would be lawful.', '',
'## Artifact inventory','', '|Binary|Unwind entries|Entries >=32 KiB|Unnamed|Modified UTC|Artifact|','|---|---:|---:|---:|---|---|']
for b in dev['binaries']:
 lines.append(f"|{b['binary']}|{b['unwind_function_count']}|{len(b['large_functions'])}|{sum(not r['names'] for r in b['large_functions'])}|{datetime.datetime.fromtimestamp(b['mtime_ns']/1e9,datetime.timezone.utc).isoformat()}|`{b['exe']}`|")
lines+=['','## Release comparison','', 'Exact owned command: `cargo build --release -p worth-query-topology-entry --example invariant_reads --example required_feature --example shared_execution --manifest-path workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml --message-format=json`. Logs: `whole-frame-release-build.log`, `whole-frame-release/build.log`; compiler unit plan/configuration is retained. No profile override, stack change or new target directory.', '',
f"Matching three-example scope: dev has **{len(example_maxima)}** owner functions >=32 KiB with bands {example_bands}; release retains **{len(release['functions'])} named owner functions >=32 KiB**, including **{release['bands']['64-128KiB']} >=64 KiB** and **{release['bands']['>=128KiB']} >=128 KiB**. Subset maxima are recomputed from the three examples alone, not inherited from the seven-binary maximum. Thus the large frames do **not** all shrink below 32 KiB. All release unwind entries, including unknown/non-owner ones, top out at {max(r['frame_bytes'] for b in release['binaries'] for r in b['large_functions'])} bytes. No release runtime success is claimed. Release retains {len(release['unresolved'])} unnamed large unwind entries; those do not conceal the negative answer to whether every frame is below 32 KiB.", '',
'|Example|Dev large entries|Release large entries|Release maximum static frame|','|---|---:|---:|---:|']
for b in release['binaries']:
 old=next(v for v in dev['binaries'] if v['binary']==b['binary'])
 lines.append(f"|{b['binary']}|{len(old['large_functions'])}|{len(b['large_functions'])}|{max(v['frame_bytes'] for v in b['large_functions'])}|")
lines+=['','Release owner functions >=64 KiB:','']
for r in release['functions']:
 if r['max_frame']>=65536:lines.append(f"- {r['max_frame']}: `{r['symbol_path']}`")
lines+=['','## Named dev owner functions, descending maximum','', '|Bytes|Owner|Probable cause|Symbol path|Binaries|','|---:|---|---|---|---|']
for r in dev['functions']:lines.append(f"|{r['max_frame']}|{protected(r['symbol_path'])}|{probable(r['symbol_path'])}|`{r['symbol_path'].replace('|',' / ')}`|{', '.join(r['binaries'])}|")
(e/'frame-census.md').write_text('\n'.join(lines)+'\n',encoding='utf-8')
unknown=['# Unattributed large unwind entries','', 'Exact RVAs and sizes, intentionally not guessed from nearest symbols. These are excluded from named-owner deduplication.','', '|Binary|RVA|Bytes|','|---|---:|---:|']
for b in dev['binaries']:
 for r in b['large_functions']:
  if not r['names']:unknown.append(f"|{b['binary']}|0x{r['rva']:x}|{r['frame_bytes']}|")
(e/'frame-census-unattributed.md').write_text('\n'.join(unknown)+('\n\nNo remaining unnamed dev entries.\n' if len(unknown)==5 else '\n'))
unrel=['# Release entries without exact names','', 'The single committed release build removes its temporary example COFF objects and disables incremental output. PDB section contributions identify their original object names, but those objects no longer exist. No second build or profile override was made. Every anonymous physical frame is retained below; named-owner totals are a lower bound. The release negative result (not all frames below 32 KiB) is independent of attribution.','', '|Example|RVA|Bytes|','|---|---:|---:|']
for b in release['binaries']:
 for r in b['large_functions']:
  if not r['names']:unrel.append(f"|{b['binary']}|0x{r['rva']:x}|{r['frame_bytes']}|")
(e/'frame-census-release-unattributed.md').write_text('\n'.join(unrel)+'\n')
print('DEV',len(dev['functions']),dev['bands'],'RELEASE',len(release['functions']),release['bands'])
print('PROTECTED >=64',collections.Counter(protected(r['symbol_path']) for r in dev['functions'] if r['max_frame']>=65536))
