import asyncio,json,pathlib,urllib.request
from playwright.async_api import async_playwright
ROOT=pathlib.Path(__file__).resolve().parents[1]
BASE='http://127.0.0.1:8787'
def state():return json.load(urllib.request.urlopen(BASE+'/api/v1/state'))
async def main():
 async with async_playwright() as p:
  browser=await p.chromium.launch(headless=True,args=['--no-sandbox'])
  page=await browser.new_page(viewport={'width':1440,'height':1000},device_scale_factor=1)
  errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
  req=urllib.request.Request(BASE+'/api/v1/command',data=json.dumps({'type':'preset','index':0}).encode(),headers={'Content-Type':'application/json'});urllib.request.urlopen(req).close()
  await asyncio.sleep(.3)
  await page.goto(BASE);await page.get_by_text('RUST ENGINE CONNECTED',exact=False).wait_for()
  assert state()['activeVoiceMask']==0
  key=page.get_by_role('button',name='MIDI note 60',exact=True)
  await key.scroll_into_view_if_needed();await key.hover();await page.mouse.down()
  await page.wait_for_timeout(400)
  playing=state();assert playing['activeVoiceMask'].bit_count()==1
  assert max(abs(v) for v in playing['scope'])>0.001
  cutoff=page.get_by_role('slider',name='Filter Cutoff',exact=True)
  before=playing['values'][23];await cutoff.focus();await cutoff.press('ArrowUp');await page.wait_for_timeout(150)
  assert state()['values'][23]!=before
  await page.screenshot(path=str(ROOT/'evidence/ui-desktop.png'),full_page=True)
  await page.mouse.up();await page.wait_for_timeout(1500)
  assert state()['activeVoiceMask']==0
  await page.get_by_role('combobox',name='Factory preset').select_option('7');await page.wait_for_timeout(150)
  await key.scroll_into_view_if_needed();await key.hover();await page.mouse.down();await page.wait_for_timeout(300)
  assert state()['activeVoiceMask'].bit_count()==12
  await page.mouse.up();await page.get_by_role('button',name='ALL NOTES OFF',exact=True).click();await page.wait_for_timeout(1600)
  assert state()['activeVoiceMask']==0
  await page.set_viewport_size({'width':390,'height':844});await page.screenshot(path=str(ROOT/'evidence/ui-mobile.png'),full_page=True)
  assert not errors,errors
  report={'status':'PASS','checks':['React connects to Rust','virtual NoteOn produces nonzero Rust samples','cutoff changes engine parameter','NoteOff releases voice','factory unison activates 12 voices','All Notes Off releases all voices','desktop and mobile screenshots','no browser runtime errors'],'browser':browser.version,'errors':errors}
  (ROOT/'evidence/ui-test.json').write_text(json.dumps(report,indent=2));print(json.dumps(report))
  await browser.close()
asyncio.run(main())
