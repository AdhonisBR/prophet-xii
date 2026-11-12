import urllib.request,urllib.error,json,time,pathlib
base='http://127.0.0.1:8787'
def send(x):
 req=urllib.request.Request(base+'/api/v1/command',data=json.dumps(x).encode(),headers={'Content-Type':'application/json'})
 try:
  with urllib.request.urlopen(req) as r:return r.status
 except urllib.error.HTTPError as e:return e.code
assert send({'type':'param','id':999,'value':1})==400
assert send({'type':'midi','status':144,'data1':255,'data2':100})==400
assert send({'type':'snapshot','values':[0]})==400
assert send({'type':'preset','index':0})==202
time.sleep(.15)
assert send({'type':'learn','id':23})==202
assert send({'type':'midi','status':176,'data1':16,'data2':64})==202
time.sleep(.2)
state=json.load(urllib.request.urlopen(base+'/api/v1/state'))
expected=20*900**(64/127)
assert abs(state['values'][23]-expected)<1.0
assert len(state['values'])==83
req=urllib.request.Request(base+'/api/v1/filter',data=json.dumps(state['values']).encode(),headers={'Content-Type':'application/json'})
response=json.load(urllib.request.urlopen(req));assert len(response['db'])==256 and all(isinstance(x,(int,float))for x in response['db'])
print('PASS: control validation, 83-parameter snapshot, MIDI Learn CC-to-cutoff scaling, measured Rust VCF response')
