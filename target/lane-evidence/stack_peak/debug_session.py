"""One debugger session follows the selected original entry and its re-execs."""
import ctypes as c,json,struct,subprocess,time
from pathlib import Path
from windows_debug import *
from painted_worker import Worker,STACK
class Session:
 def __init__(self,artifact,symbols,targets=None):
  self.artifact=artifact;self.symbols=symbols;self.targets=targets;self.states={};self.results=[];self.handles=set();self.failures=[]
 def launch(self):
  a=self.artifact;args=[a['exe']]
  if 'test' in a:args+=['--exact',a['test'],'--nocapture']
  command=subprocess.list2cmdline(args);print('DIAGNOSTIC COMMAND',json.dumps(args),flush=True)
  si=SI();si.cb=c.sizeof(si);si.flags=0x100;si.stdin=k.GetStdHandle(-10&0xffffffff);si.stdout=k.GetStdHandle(-11&0xffffffff);si.stderr=k.GetStdHandle(-12&0xffffffff)
  pi=PI();checked(k.CreateProcessW(None,c.create_unicode_buffer(command),None,None,True,1,None,a['cwd'],c.byref(si),c.byref(pi)));self.root=pi.pid;self.handles.update([pi.process,pi.thread])
  try:self.loop()
  finally:
   for state in self.states.values():k.TerminateProcess(state['process'],0xdead)
   for h in self.handles:
    if h:k.CloseHandle(h)
  assert not self.failures,self.failures
  count=5 if 'test' in a else 1
  assert len(self.results)==count,(len(self.results),count)
  self.results.sort(key=lambda x:x['index']);return self.results
 def created(self,e):
  raw=bytes(e.data);process,thread,base=struct.unpack_from('<QQQ',raw,8);index=len(self.states)
  self.handles.update([process,thread]);entry=base+self.symbols.entries['entry'];old=read(process,entry,1);assert old!=b'\xcc'
  bp(process,entry,b'\xcc');self.states[e.pid]={'process':process,'base':base,'threads':{e.tid:thread},'entry':entry,'byte':old,'index':index,'worker':None};self.handles.add(struct.unpack_from('<Q',raw)[0])
 def entry(self,state,e,ctx):
  worker=state['worker'];thread=state['threads'][e.tid]
  print('ENTRY',e.pid,e.tid,worker.phase if worker else 'original',flush=True)
  if worker is None:
   d.SymSetOptions(0x10|0x200|0x4);checked(d.SymInitializeW(state['process'],str(Path(self.artifact['exe']).parent),True));state['symbols_initialized']=True
   setreg(ctx,'rip',state['entry']);assert k.SuspendThread(thread)!=0xffffffff
   print('CREATE_REMOTE_BEGIN',flush=True)
   tid=w.DWORD();handle=checked(k.CreateRemoteThread(state['process'],None,STACK,state['entry'],get(ctx,'rcx'),4|0x10000,c.byref(tid)))
   print('CREATE_REMOTE_END',tid.value,flush=True)
   self.handles.add(handle);state['threads'][tid.value]=handle
   target=self.targets[state['index']] if self.targets else None
   state['worker']=Worker(state['process'],thread,ctx,state['entry'],handle,tid.value,state['index'],self.symbols,state['base'],target)
   assert k.ResumeThread(handle)!=0xffffffff
  elif e.tid==worker.tid and worker.phase=='entry':worker.start(ctx)
  elif e.tid==worker.tid and worker.phase=='limits':
   bp(state['process'],state['entry'],state['byte']);worker.paint()
  else:raise AssertionError(('unexpected entry',e.pid,e.tid,worker.phase))
 def loop(self):
  exits=set();started=time.monotonic()
  while True:
   event=Event()
   if not k.WaitForDebugEvent(c.byref(event),15000):
    assert time.monotonic()-started<1800,'diagnostic exceeded30minutes'
    print('WAITING',[(pid,st['worker'].probe_peak if st['worker'] else 0) for pid,st in self.states.items()],flush=True)
    continue
   raw=bytes(event.data);disposition=0x10002
   if event.code in (2,3,5):print('EVENT',event.code,event.pid,event.tid,flush=True)
   if event.code==3:self.created(event)
   elif event.code==2:
    state=self.states[event.pid];thread=struct.unpack_from('<Q',raw)[0];state['threads'][event.tid]=thread;self.handles.add(thread)
    if state['worker'] and event.tid==state['worker'].tid:state['worker'].teb=struct.unpack_from('<Q',raw,8)[0]
   elif event.code==6:
    handle=struct.unpack_from('<Q',raw)[0]
    if handle:self.handles.add(handle)
   elif event.code==1:
    code=struct.unpack_from('<I',raw)[0];address=struct.unpack_from('<Q',raw,16)[0];state=self.states[event.pid];worker=state['worker']
    if code==0x80000003:print('BREAK',hex(address),'entry',hex(state['entry']),flush=True)
    if code==0x80000003 and address==state['entry']:
     self.entry(state,event,context(state['threads'][event.tid]))
    elif code==0x80000003 and worker and worker.phase=='body' and address==worker.hook.notify:
     worker.new_minimum(context(worker.thread))
    elif code==0x80000004 and worker and event.tid==worker.tid:
     result=worker.event(context(worker.thread))
     if result:
      result['pid']=event.pid;self.results.append(result);print('PEAK',json.dumps({k:v for k,v in result.items() if k!='call_path'}),flush=True)
    elif code not in (0x80000003,0x406d1388):
     print('EXCEPTION',event.pid,event.tid,hex(code),'first',struct.unpack_from('<I',raw,152)[0],hex(address),'info',struct.unpack_from('<QQ',raw,32),flush=True);disposition=0x80010001
     if code==0xc0000005:
      ctx=context(state['threads'][event.tid]);print('FAULT_CONTEXT',{n:hex(get(ctx,n)) for n in ['rip','rsp','rax','rcx','rdx','dr6','dr7']},flush=True)
      print('FAULT_PATH',json.dumps(self.symbols.capture(state['process'],state['threads'][event.tid],ctx,state['base'])[:24]),flush=True)
     if code==0xc00000fd:self.failures.append(('overflow',event.pid,event.tid))
   elif event.code==5:
    code=struct.unpack_from('<I',raw)[0];print('PROCESS EXIT',event.pid,code,flush=True);exits.add(event.pid)
    if code:self.failures.append(('exit',event.pid,code))
    if self.states[event.pid].get('symbols_initialized'):d.SymCleanup(self.states[event.pid]['process'])
    if event.pid==self.root:
     checked(k.ContinueDebugEvent(event.pid,event.tid,disposition));break
   checked(k.ContinueDebugEvent(event.pid,event.tid,disposition))
   assert time.monotonic()-started<1800,'diagnostic session exceeded30minutes'
