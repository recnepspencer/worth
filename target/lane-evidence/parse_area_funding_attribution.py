from pathlib import Path
import re,json,sys
marker='WORTH-UI-TEMPORARY-INSTRUMENTATION'
log=Path(sys.argv[1]); text=log.read_text(errors='replace')
blocks=re.findall(marker+r' BEGIN head=41 covered=0(.*?)'+marker+r' END',text,re.S)
observations=[]
for block in blocks:
 rows=re.findall(marker+r' PHASE ([a-z_/]+) units=(\d+)',block)
 phases={name:int(value) for name,value in rows}
 assert len(phases)==len(rows), 'duplicate phase names in one capture'
 total=re.findall(marker+r' TOTAL units=(\d+)',block)
 if total and phases: observations.append((int(total[0]),phases))
assert observations,'complete owner phase observation missing'
unique={json.dumps(value,sort_keys=True) for value in observations}
assert len(unique)==1,'distinct diagnostic captures found'
total,phases=observations[0]
exclusive=dict(phases)
native=sum(value for name,value in phases.items() if name.startswith('native/'))
exclusive['store/native_and_image_seal']-=native
assert exclusive['store/native_and_image_seal']>=0
residual=total-sum(exclusive.values())
assert residual>=0
result={'total':total,'baseline':16240012,'delta':total-16240012,'head':41,'covered_before':0,
        'phases':phases,'exclusive_phases':exclusive,'selection_ack_residual':residual,'log':str(log)}
if len(sys.argv)>2:Path(sys.argv[2]).write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
