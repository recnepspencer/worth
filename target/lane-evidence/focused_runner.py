from pathlib import Path
import sys,json,traceback
sys.stdout.reconfigure(encoding="utf-8",errors="replace")
root=Path.cwd()
sys.path.insert(0,str(root/'scripts/test_cost'))
from run_lane import execute
from logs import json_objects as documents
from coverage import inventory
workspace,label,package,target,expression=sys.argv[1:]
output=root/'target/test-cost'/workspace/label
result=None
code=0
try:
    targets=({'lib':True,'test':target.removeprefix('lib+').split(',')} if target.startswith('lib+') else {'lib':True} if target=='lib' else {'test':target.split(',')})
    result=execute(root,workspace,['owner-unit' if target=='lib' else 'ordinary'],output,
        expression=expression,native_selection={'packages':package.split(','),'targets':targets})
except Exception as error:
    traceback.print_exc()
    code=1
finally:
    counts=[0,0,0]
    for log in output.rglob('*.log'):
        text=log.read_text(encoding='utf-8-sig',errors='replace')
        if log.name.startswith('run-'):
            print('execution_log: '+str(log))
            for event in documents(text):
                if event.get('type')=='test' and event.get('event') in ('ok','failed'):
                    counts[0 if event['event']=='ok' else 1]+=1
        if code and ('binaries-build' in log.name or log.name.startswith('run-')):
            print('failure_log: '+str(log))
            print('\n'.join(event['message']['rendered'] for event in documents(text) if event.get('reason')=='compiler-message' and event.get('message',{}).get('level')=='error') if 'binaries-build' in log.name else text[-25000:])
    selection=output/'build/request-selection.json'
    if selection.exists():
        counts[2]=sum(row['ignored'] for row in inventory(json.loads(selection.read_text(encoding='utf-8'))) if row.get('selected',True))
    print('EVIDENCE_TEST_COUNTS: '+','.join(map(str,counts)))
sys.exit(code)
