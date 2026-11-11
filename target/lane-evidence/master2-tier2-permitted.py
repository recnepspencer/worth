"""Prepare or run combined merge Tier 2 through the current owned runners.

Default is plan-only. --execute is an explicit lane launch; preparation never
lists binaries, invokes Cargo, builds, or executes a test.
"""
from pathlib import Path
import argparse,json,os,re,shutil,subprocess,sys,tomllib

ROOT=Path(__file__).resolve().parents[2]
BASE='7c3d7f39192bb099f286de1af0704558b0863cdc'
EVIDENCE=ROOT/'target/lane-evidence'
sys.path[:0]=[str(ROOT/'scripts/ci'),str(ROOT/'scripts/test_cost'),str(ROOT/'scripts')]
POLICY=json.loads((EVIDENCE/'master2-tier2-selection-policy.json').read_text())
HELD={(r['package'],r['test']) for r in POLICY['ignored_child_parents']}
REQUIRED={(r['package'],r['test']) for r in POLICY['required_normal_cases']}
PATTERNS=[re.compile(p) for p in POLICY['excluded_campaign_patterns']]

class DiskFloorRefusal(RuntimeError):
    pass

def disk_guard():
    free=shutil.disk_usage(ROOT.anchor).free
    if free<25*1024**3:
        raise DiskFloorRefusal(f'disk floor; command not launched: {free} bytes free')

def permitted(package,name):
    key=(package,name)
    return key not in HELD and (key in REQUIRED or not any(p.search(name) for p in PATTERNS))

def guard(package):
    campaigns=' & '.join('not test(/'+p.pattern+'/)' for p in PATTERNS)
    required=['test(='+name+')' for owner,name in sorted(REQUIRED) if owner==package]
    eligible='('+campaigns+')'
    if required:
        eligible='('+eligible+' + '+' + '.join(required)+')'
    holds=['not test(='+name+')' for owner,name in sorted(HELD) if owner==package]
    return ' & '.join([eligible,*holds])

def source_plan():
    changed=subprocess.check_output(['git','diff','--name-only',BASE,'--'],cwd=ROOT,text=True).splitlines()
    changed+=subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=ROOT,text=True).splitlines()
    manifests=subprocess.check_output(['rg','--files','-g','Cargo.toml'],cwd=ROOT,text=True).splitlines()
    owners={}
    for p in manifests:
        path=ROOT/p
        content=tomllib.loads(Path('\\\\?\\'+str(path.resolve())).read_text(encoding='utf-8'))
        if 'package' in content:owners[path.parent.resolve()]=content['package']['name']
    direct={}
    for name in sorted(set(changed)):
        path=(ROOT/name).resolve()
        owner=next((owners[p] for p in path.parents if p in owners),None)
        if owner:direct.setdefault(owner,[]).append(name)
    return {'base':BASE,'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'dirty':bool(subprocess.check_output(['git','status','--short'],cwd=ROOT,text=True).strip()),
            'direct_changed_packages':direct,'changed_noncrate_inputs':[p for p in changed if p.startswith(('scripts/','docs/','.github/','.config/'))],
            'selection':POLICY,'status':'prepared-not-executed','qualification':'partial permitted selection; exclusions remain explicit'}

def runner_type():
    from cargo_capture import CargoRunner
    class GuardedRunner(CargoRunner):
        def __call__(self,command,cwd,log_path):
            assert 'RUST_MIN_STACK' not in self.environment
            if any(flag in command for flag in ['--ignored','--include-ignored']):
                raise ValueError('prohibited ignored-body execution flag')
            free=shutil.disk_usage(ROOT.anchor).free
            print('C_FREE_BYTES',free,'FLOOR',25*1024**3,flush=True)
            if free<25*1024**3:raise DiskFloorRefusal(f'disk floor; command not launched: {free} bytes free')
            return super().__call__(command,cwd,log_path)
    return GuardedRunner

def install_query_policy(output):
    import query_gate,query_gate_cost as cost,query_gate_prebuild
    Standard=cost.SuiteQueries
    original_selected=cost.selected
    original_expression=cost.native_expression
    def selected(suite,binary,name):
        if suite.endswith('-scale'):return False
        return original_selected(suite,binary,name) and permitted(binary['package'],name)
    def expression(suite,binary):
        return '('+original_expression(suite,binary)+' & '+guard(binary['package'])+')'
    class AuditedQueries(Standard):
        def __call__(self,discovered):
            jobs=super().__call__(discovered)
            chosen=[{'suite':suite,'binary':binary,'test':test} for suite,rows in self.expected.items() for binary,test in sorted(rows)]
            for row in chosen:
                metadata=discovered['rust-suites'][row['binary']]
                assert not metadata['testcases'][row['test']]['ignored']
                assert permitted(metadata['package-name'],row['test'])
            (self.output/'permitted-selection.json').write_text(json.dumps(chosen,indent=2)+'\n')
            return jobs
    cost.selected=selected;query_gate.selected=selected;cost.native_expression=expression;cost.SuiteQueries=AuditedQueries
    group,want,size,_=query_gate.TEST_SUITES['fixture']
    query_gate.TEST_SUITES['fixture']=(group,want,size,1)
    runner=runner_type();cost.CargoRunner=runner;query_gate.CargoRunner=runner;query_gate_prebuild.CargoRunner=runner
    return query_gate,cost

def execute_store(output,environment):
    from coverage import inventory
    from run_lane import execute
    from query_gate_cost import descriptor,exact_binary
    class StoreQueries:
        workspace_inventory=True
        def __call__(self,discovered):
            selected=[];held=[];terms=[]
            for binary_id,metadata in sorted(discovered['rust-suites'].items()):
                binary=descriptor(binary_id,metadata)
                assert binary['harness'],'classify standalone Store target before executing it'
                chosen=False
                for name,case in metadata['testcases'].items():
                    row={'package':binary['package'],'binary':binary_id,'test':name,'ignored':case['ignored']}
                    if not case['ignored'] and permitted(binary['package'],name):selected.append(row);chosen=True
                    else:held.append(row)
                if chosen:terms.append('('+exact_binary(binary_id)+' & '+guard(binary['package'])+')')
            actual={(r['package'],r['test']) for r in selected}
            required={(r['package'],r['test']) for r in POLICY['required_normal_cases']}
            if not required<=actual:raise ValueError('required normal graceful/D18 case absent, ignored or excluded: '+repr(required-actual))
            (output/'permitted-selection.json').write_text(json.dumps({'selected':selected,'held':held,'required_normal_cases':POLICY['required_normal_cases']},indent=2)+'\n')
            return [('permitted-native',' + '.join(terms),None,'ci')]
    return execute(ROOT,'store',[],output,profile='ci-test',cargo_runner=runner_type()(ROOT,environment),queries=StoreQueries(),include_docs=True)

def execute_governed(workspace,output,cost,environment):
    from run_lane import execute
    queries=cost.SuiteQueries(cost.landing_suites(workspace),output)
    # The owned executor proves this selected subset. CostGate additionally
    # requires a complete landing selection, which is deliberately not claimed.
    return execute(ROOT,workspace,[],output,cargo_runner=runner_type()(ROOT,environment),queries=queries,include_docs=True)

def execute_fixture(output,query_gate,environment,query_prerequisite):
    if query_prerequisite is None:
        from compiler_prebuild import CompilerScope,prepare
        environment.update(prepare(ROOT,'query',output/'query-compiler-prerequisite',runner_type()(ROOT,environment),scope=CompilerScope.FIXTURES))
    else:
        environment['WORTH_PREBUILT_COMPILER_OUTCOMES']=query_prerequisite
    # Keep the exact shared, issued environment and actual Gate fixture owner.
    # Its ordinary environment factory injects a stack override, so do not call it.
    query_gate.environment=lambda repository:environment
    original_chunk=query_gate.Gate.chunk
    def checked_chunk(gate,*args,**kwargs):
        assert 'RUST_MIN_STACK' not in gate.env
        disk_guard()
        return original_chunk(gate,*args,**kwargs)
    query_gate.Gate.chunk=checked_chunk
    class GuardedSubprocess:
        def __getattr__(self,name):return getattr(subprocess,name)
        def run(self,*args,**kwargs):
            disk_guard()
            return subprocess.run(*args,**kwargs)
    # Gate's --list observations bypass CargoRunner and chunk(). Delegate the
    # exact same subprocess call, guarded before creating the process.
    query_gate.subprocess=GuardedSubprocess()
    return query_gate.Gate(output,census=True).run('fixture')

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execute',action='store_true')
    parser.add_argument('--only',default='root,query,fixture,bank,store')
    parser.add_argument('--out',type=Path,default=EVIDENCE/'master2-combined-tier2')
    args=parser.parse_args()
    requested=args.only.split(',')
    assert len(requested)==len(set(requested)) and set(requested)<=set(['root','query','fixture','bank','store'])
    pause=EVIDENCE/'codex-batch1-pause-before-next-workspace'
    if args.execute and requested!=['root'] and pause.exists():
        print('DEFERRED BEFORE PREPARATION: finish and commit Root repair batch; no Cargo or test launched',flush=True)
        return 75
    plan=source_plan();args.out.mkdir(parents=True,exist_ok=True)
    (args.out/'plan.json').write_text(json.dumps(plan,indent=2)+'\n')
    print(json.dumps({'status':'prepared-not-executed','base':BASE,'workspaces':requested,'direct_changed_packages':len(plan['direct_changed_packages']),'exact_ignored_child_parent_holds':len(HELD)},indent=2))
    if not args.execute:return 0
    from application_build import application_build_environment
    from run_lane import child_environment,LaneExecutionFailure
    assert 'RUST_MIN_STACK' not in os.environ
    environment=application_build_environment(child_environment())
    assert 'RUST_MIN_STACK' not in environment
    os.environ['WORTH_IMPLEMENTATION_BUILD_ID']=environment['WORTH_IMPLEMENTATION_BUILD_ID']
    query_gate,cost=install_query_policy(args.out)
    from bank_prebuild import BankMetadataPrebuildFailure
    from query_gate_prebuild import PrerequisiteFailure
    results={}
    stopped=None
    def persist():
        (args.out/'permitted-summary.json').write_text(json.dumps({'results':results,'coverage':'partial','exclusions':POLICY,'stack_posture':{'external_override':False,'committed_cargo_worker_stack_bytes':67108864,'default_stack_proof':False}},indent=2)+'\n')
    def stage(name,operation):
        nonlocal stopped
        if stopped is not None:
            results[name]={'status':'blocked-unrun','failure':stopped}
            persist()
            return
        try:
            disk_guard()
            result=operation()
            results[name]={'failures':result.get('failures',[]),'coverage':result.get('executed_coverage')}
        except DiskFloorRefusal as failure:
            stopped=f'{type(failure).__name__}: {failure}'
            results[name]={'status':'resource-refused','failure':stopped}
        except LaneExecutionFailure as failure:
            results[name]={'failure':str(failure),'failures':failure.result.get('failures',[]),'coverage':failure.result.get('executed_coverage')}
        except (ValueError,OSError,BankMetadataPrebuildFailure,PrerequisiteFailure) as failure:
            results[name]={'status':'preparation-refused','failure':f'{type(failure).__name__}: {failure}','artifacts':str(args.out/name)}
        finally:
            persist()
    query_prerequisite=None
    for workspace in ['root','query','bank']:
        if workspace not in requested:continue
        output=args.out/workspace
        receipt=output/'prebuilt/environment.json'
        assert not receipt.exists(),'reuse completed evidence; do not rerun into an existing output'
        stage(workspace,lambda:execute_governed(workspace,output,cost,environment))
        if workspace=='query' and receipt.is_file():
            actual=json.loads(receipt.read_text()).get('WORTH_PREBUILT_COMPILER_OUTCOMES')
            expected=(output/'prebuilt/compiler/compiler-outcomes.json').resolve()
            if actual is not None and (Path(actual).resolve()!=expected or not expected.is_file()):
                raise ValueError('current-run Query compiler prerequisite receipt mismatch')
            query_prerequisite=actual
    if 'fixture' in requested:
        def fixture():
            result=execute_fixture(args.out/'fixture',query_gate,environment,query_prerequisite)
            return {'failures':[] if result['ok'] else [result]}
        stage('fixture',fixture)
    if 'store' in requested:
        stage('store',lambda:execute_store(args.out/'store',environment))
    return int(any(row.get('failure') or row.get('failures') for row in results.values()))

if __name__=='__main__':raise SystemExit(main())
