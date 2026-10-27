#!/usr/bin/env python3
"""Actual executable + OS pipes. Every reverse OS producer is mocked/denied."""
import json,os,pathlib,select,subprocess,sys,time,uuid
ROOT=pathlib.Path(__file__).resolve().parents[1]
BINARY=pathlib.Path(sys.argv[1]).resolve() if len(sys.argv)>1 else ROOT/'target/debug/converter'
BASE=ROOT/'.qa/process';BASE.mkdir(parents=True,exist_ok=True)
for name in ['home','tmp','data','package']:(BASE/name).mkdir(exist_ok=True)
checks=0
class Process:
 def __init__(self):
  env={**os.environ,'HOME':str(BASE/'home'),'TMPDIR':str(BASE/'tmp')}
  self.p=subprocess.Popen([str(BINARY)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env)
  self.buffer=b'';self.responses={};self.calls=[];self.cancels=[];self.reply={'host.clipboard.read':'','host.clipboard.write':{'ok':True},'host.settings.open':{'ok':True}};self.hold=False
  hello=self.read();assert hello['type']=='hello' and hello['id']=='com.tezbar.native.converter' and hello['sdkVersion']=='0.1.1'
  assert self.call('initialize',{'protocol':1,'extensionId':'com.tezbar.native.converter','platform':'macOS','packageDir':str(BASE/'package'),'dataDir':str(BASE/'data'),'tempDir':str(BASE/'tmp'),'grantedCapabilities':['clipboard.read','clipboard.write','settings.open'],'migrationPaths':{}})['result']['ready']
 def send(self,v):self.p.stdin.write(json.dumps(v,ensure_ascii=True).encode()+b'\n');self.p.stdin.flush()
 def read(self,timeout=8):
  deadline=time.monotonic()+timeout
  while b'\n' not in self.buffer:
   remaining=deadline-time.monotonic();assert remaining>0,'process timeout'
   assert select.select([self.p.stdout],[],[],remaining)[0],'process timeout'
   chunk=os.read(self.p.stdout.fileno(),65536);assert chunk,'premature EOF: '+self.p.stderr.read().decode();self.buffer+=chunk
  line,self.buffer=self.buffer.split(b'\n',1);return json.loads(line)
 def request(self,method,params):
  id='h:'+str(uuid.uuid4());self.send({'type':'request','id':id,'method':method,'params':params});return id
 def pump(self):
  m=self.read()
  if m['type']=='request':
   self.calls.append(m)
   assert m['method'] in self.reply,'unmocked producer denied: '+m['method']
   if not self.hold:self.send({'type':'response','id':m['id'],'result':self.reply[m['method']]})
  elif m['type']=='response':self.responses[m['id']]=m
  elif m['type']=='cancel':self.cancels.append(m)
  else:assert False,m
  return m
 def wait(self,id):
  while id not in self.responses:self.pump()
  return self.responses.pop(id)
 def call(self,method,params):return self.wait(self.request(method,params))
 def open(self,command,prefs=None):
  id='s-'+str(uuid.uuid4());r=self.call('session.open',{'sessionId':id,'commandName':command,'preferences':{'autoPaste':False,**(prefs or {})},'argumentValues':{}});assert 'result'in r,r;return id,r['result']
 def invoke(self,id,action,form=None):return self.call('session.call',{'sessionId':id,'operation':'extension:invoke-action','payload':{'actionId':action,'formValues':form or {}}})
 def refresh(self,id):return self.call('session.call',{'sessionId':id,'operation':'extension:refresh-session','payload':{}})['result']
 def dispose(self,id):assert 'result' in self.call('session.dispose',{'sessionId':id})
 def close(self):
  self.p.stdin.close();assert self.p.wait(timeout=8)==0,self.p.stderr.read().decode()

def nodes(v):
 def walk(n):
  yield n
  for c in n.get('children',[]):yield from walk(c)
 return list(walk(v['root']))
def values(v):return {n['props']['id']:n['props']['value'] for n in nodes(v) if n['type'] in ['Form.TextField','Form.Dropdown']}
def field(v,key):return next(n['props'] for n in nodes(v) if n.get('props',{}).get('id')==key)
def action(v,title):return next(a['id'] for a in v['actions'] if a['title']==title)
def result(r):assert 'result'in r,r;return r['result']
def check(ok):
 global checks
 assert ok;checks+=1

p=Process()
views=[]
# Executed source TSX layout/actions, including advanced-view order and preferences.
for case in json.loads((ROOT/'qa/ui-oracle.json').read_text()):
 id,v=p.open(case['command'],case['prefs']);views.append(v)
 actual=[{'type':n['type'],'title':n.get('props',{}).get('title'),'value':n.get('props',{}).get('value')} for n in nodes(v)[1:]]
 assert actual==case['fields'],(case['command'],actual,case['fields']);check(True);assert [a['title']for a in v['actions']]==[a['title']for a in case['actions']],([a['title']for a in v['actions']],case['actions']);p.dispose(id)
(BASE/'views.json').write_text(json.dumps(views))
# Apply every finite hook oracle through the production JSONL adapter as well.
for case in json.loads((ROOT/'qa/oracle.json').read_text()):
 prefs={'asciiCommaSeparated':case.get('comma',True),'advanceView':case.get('advance',True)}
 if 'autopaste'in case:prefs['autoPaste']=True;p.reply['host.clipboard.read']=case['autopaste']
 id,v=p.open(case['command'],prefs)
 for step in case.get('steps',[]):
  key=step['key'];v=result(p.invoke(id,field(v,key)['actionId'],{'value':step['value']}))
 if 'throws'in case:check(any(n.get('props',{}).get('error') for n in nodes(v)) and not any(':action:copy-'in a['id'] for a in v['actions']))
 else:
  got=values(v);expected=dict(case['expected'])
  if case['command']=='base-converter':
   output=next((n['props']['text'].removesuffix('\n') for n in nodes(v) if n['type']=='Form.Description' and n['props'].get('title')=='Output'),None)
   expected_output=expected.pop('output')
   if prefs['advanceView']:check(output==expected_output)
   else:expected.pop('input')
   got.pop('fromBase',None);got.pop('toBase',None)
  check(got==expected)
 p.dispose(id)
# A/B/A and delayed per-field revisions: old render IDs still deliver new edits,
# while old COPY IDs never regain authority.
id,v=p.open('code-converter');edit=field(v,'native')['actionId'];oldcopy=action(v,'Copy Native')
a=result(p.invoke(id,edit,{'value':'A','_inputRevision':1}));b=result(p.invoke(id,edit,{'value':'B','_inputRevision':2}));a=result(p.invoke(id,edit,{'value':'A','_inputRevision':3}))
check(values(a)['native']=='A');check('error'in p.invoke(id,edit,{'value':'late','_inputRevision':2}));check('error'in p.invoke(id,oldcopy));check(values(p.refresh(id))['native']=='A')
# Focus-only live events and entire unflushed form draft submitted by copy.
v=p.refresh(id);copy=action(v,'Copy Native');v=result(p.invoke(id,copy,{'native':'unflushed','_lastChangedField':'native','_inputRevision':4}));check(p.calls[-1]['params']=={'text':'unflushed'});check(values(v)['native']=='unflushed')
# Strict ACK rejection, explicit reachable retry, exact bytes, one-use action.
for bad in [None,True,False,{}, {'ok':False},{'ok':True,'message':None},{'ok':True,'extra':1},{'ok':True,'error':'contradiction'}]:
 p.reply['host.clipboard.write']=bad;v=p.refresh(id);old=action(v,'Copy Native');v=result(p.invoke(id,old));check(any('may repeat'in a['title'] for a in v['actions']));check('error'in p.invoke(id,old))
 original=p.calls[-1]['params'];p.reply['host.clipboard.write']={'ok':True};retry=next(a['id']for a in v['actions']if ':action:retry'in a['id']);v=result(p.invoke(id,retry));check(p.calls[-1]['params']==original);check(not any(':action:retry'in a['id']for a in v['actions']))
# Cancellation leaves external effect uncertain, consumes authority outside staged
# state, and refresh provides exact prepared-byte retry without automatic replay.
v=p.refresh(id);old=action(v,'Copy Native');p.hold=True
req=p.request('session.call',{'sessionId':id,'operation':'extension:invoke-action','payload':{'actionId':old}})
while True:
 m=p.pump()
 if m['type']=='request':break
prepared=m['params'];count=len(p.calls);p.send({'type':'cancel','requestId':req,'reason':'QA cancellation'});check('error'in p.wait(req));p.hold=False
check('error'in p.invoke(id,old));v=p.refresh(id);check(len(p.calls)==count);retry=next(a['id']for a in v['actions']if ':action:retry'in a['id']);v=result(p.invoke(id,retry));check(p.calls[-1]['params']==prepared)
# Failed malformed edit cannot reauthorize old output; refresh recovers draft.
v=p.refresh(id);old=action(v,'Copy Native');check('error'in p.invoke(id,field(v,'native')['actionId'],{'value':42}));check('error'in p.invoke(id,old));check(values(p.refresh(id))['native']=='unflushed')
# Conversion failure preserves source draft and blocks old copy; normal edit recovers.
v=p.refresh(id);v=result(p.invoke(id,field(v,'unicode')['actionId'],{'value':'\\ud800'}));check(values(v)['unicode']=='\\ud800');check(not any('copy-'in a['id']for a in v['actions']));v=result(p.invoke(id,field(v,'native')['actionId'],{'value':'recovered'}));check(values(v)['native']=='recovered')
# Bound adaptation is deliberate and reachable, no Number precision substitution.
v=result(p.invoke(id,field(v,'native')['actionId'],{'value':'x'*4097}));check(any(n.get('props',{}).get('error')for n in nodes(v)));v=result(p.invoke(id,field(v,'native')['actionId'],{'value':'within bound'}));check(values(v)['native']=='within bound')
# Latest search uses the SDK's reader-order cancellation.
requests=[p.request('session.call',{'sessionId':id,'operation':'extension:search-text-changed','payload':{'searchText':t}})for t in ['A','B','A']]
results=[p.wait(r)for r in requests];check(values(results[-1]['result'])['native']=='A');check(values(p.refresh(id))['native']=='A')
# Old/fabricated actions and foreign session IDs fail without producers.
v=p.refresh(id);foreign=action(v,'Copy Native');p.dispose(id);id2,v2=p.open('code-converter');check('error'in p.invoke(id2,foreign));p.dispose(id2)
# Cancelled opens may omit dispose. Repeated >128 proves SDK and authority cleanup.
for _ in range(140):
 p.hold=True;session='cancel-'+str(uuid.uuid4());r=p.request('session.open',{'sessionId':session,'commandName':'code-converter','preferences':{'autoPaste':True}})
 while True:
  m=p.pump()
  if m['type']=='request':break
 p.send({'type':'cancel','requestId':r,'reason':'cancel open'});check('error'in p.wait(r));p.hold=False
# Real session capacity plus recovery after dispose.
sessions=[p.open('byte-converter')[0]for _ in range(128)]
r=p.call('session.open',{'sessionId':'over-capacity','commandName':'byte-converter','preferences':{'autoPaste':False}});check('error'in r)
for s in sessions:p.dispose(s)
s,v=p.open('base-converter');check(field(v,'base10')['value']=='');p.dispose(s)
# Wrong clipboard result becomes a visible ordinary failure; string is required.
p.reply['host.clipboard.read']={'text':'fake'};s,v=p.open('code-converter',{'autoPaste':True});check(any('must return a string'in n.get('props',{}).get('text','')for n in nodes(v)))
p.reply['host.clipboard.read']='retried clipboard';retry=next(a['id']for a in v['actions']if ':action:retry'in a['id']);v=result(p.invoke(s,retry));check(values(v)['native']=='retried clipboard')
v=result(p.invoke(s,action(v,'Configure Extension')));check(p.calls[-1]['method']=='host.settings.open' and p.calls[-1]['params']=={'surface':'settings-extensions'});p.dispose(s)
# Clear All preserves source advanced base selections and focused field.
s,v=p.open('base-converter');v=result(p.invoke(s,field(v,'fromBase')['actionId'],{'value':'16'}));v=result(p.invoke(s,field(v,'toBase')['actionId'],{'value':'36'}));v=result(p.invoke(s,field(v,'input')['actionId'],{'value':'ff'}));v=result(p.invoke(s,action(v,'Clear All')));check(values(v)['fromBase']=='16' and values(v)['toBase']=='36' and values(v)['input']=='');check(v['actions'][0]['title']=='Copy Base 16');p.dispose(s)
# Oversized automatic clipboard also keeps a bounded visible error draft.
p.reply['host.clipboard.read']='x'*4097;s,v=p.open('code-converter',{'autoPaste':True});check(values(v)['native']=='x'*4096 and bool(field(v,'native')['error']));p.dispose(s)
p.close()
# EOF while reverse RPC is pending; also reopen a fresh process.
p=Process();p.hold=True;p.request('session.open',{'sessionId':'eof','commandName':'code-converter','preferences':{'autoPaste':True}})
while p.pump()['type']!='request':pass
p.close();p=Process();s,v=p.open('code-converter');check(values(v)['native']=='');p.dispose(s);p.close()
print(json.dumps({'checks':checks,'hook_oracles':1859,'tsx_ui_oracles':9,'cancelled_opens_without_dispose':140,'capacity':128,'binary':str(BINARY)}))
