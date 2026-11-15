// Document QA only: no product UC, business data, CLI or model calls.
const {chromium} = require('playwright');
const {pathToFileURL} = require('node:url');
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const root = path.resolve(__dirname, '../..');
const output = path.join(root, 'docs/flows/previews');

async function inspect(page) {
  return page.locator('svg').evaluateAll(elements => {
    const problems = [];
    for (const svg of elements) {
      const view = svg.viewBox.baseVal;
      const texts = [...svg.querySelectorAll('text')].map(el => ({text:el.textContent, box:el.getBBox()}));
      for (const {text,box} of texts) {
        if (box.x < 0 || box.y < 0 || box.x+box.width > view.width || box.y+box.height > view.height)
          problems.push({type:'outside-svg',text});
      }
      for (let i=0;i<texts.length;i++) for(let j=i+1;j<texts.length;j++) {
        const a=texts[i].box,b=texts[j].box;
        if(Math.min(a.x+a.width,b.x+b.width)-Math.max(a.x,b.x)>0.5 &&
           Math.min(a.y+a.height,b.y+b.height)-Math.max(a.y,b.y)>0.5)
          problems.push({type:'text-overlap',a:texts[i].text,b:texts[j].text});
      }
    }
    return problems;
  });
}

(async()=>{
  fs.mkdirSync(output,{recursive:true});
  const browser=await chromium.launch({headless:true,...(process.env.HUBA_CHROMIUM_EXECUTABLE?{executablePath:process.env.HUBA_CHROMIUM_EXECUTABLE}:{})});
  const page=await browser.newPage({viewport:{width:1440,height:1000}});
  const errors=[], network=[];
  const artifact = path.join(root,'docs/flows/index.html');
  const hash = () => crypto.createHash('sha256').update(fs.readFileSync(artifact)).digest('hex');
  const startHash = hash();
  page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  page.on('pageerror',e=>errors.push(e.message));
  page.on('requestfailed',r=>errors.push(r.url()+': '+r.failure()?.errorText));
  page.on('request',r=>{if(/^https?:/.test(r.url()))network.push(r.url());});
  try {
    await page.goto(pathToFileURL(path.join(root,'docs/flows/index.html')).href);
    await page.evaluate(()=>document.fonts.ready);
    assert.equal(await page.locator('svg').count(),13);
    assert.equal(await page.locator('#components tbody tr').count(),30);
    assert.equal(await page.locator('#uc tbody tr').count(),16);
    assert.deepEqual(await inspect(page),[]);
    const flowIds=['X0','U1','U2','U3','U4','D1','D2','D3','D4','C1','C2','C3','C4'];
    for (const id of flowIds) {
      await page.getByRole('navigation').getByRole('link',{name:id,exact:true}).click();
      assert.equal(await page.evaluate(()=>location.hash),'#'+id);
      const section=page.locator('#'+id);
      await section.locator('summary').click();
      assert.equal(await section.locator('details').getAttribute('open'),'');
      await section.locator('summary').click();
      await section.screenshot({path:path.join(output,id+'.png')});
    }
    // Full-width overview screenshot, without changing the source document.
    await page.setViewportSize({width:2800,height:1000});
    await page.locator('main').evaluate(el=>{el.style.maxWidth='2800px';});
    await page.locator('#X0').screenshot({path:path.join(output,'X0-full.png')});
    await page.locator('main').evaluate(el=>{el.style.maxWidth='';});
    await page.setViewportSize({width:1440,height:1000});
    await page.getByRole('navigation').getByRole('link',{name:'Dữ liệu đi đường nào?',exact:true}).hover();
    await page.screenshot({path:path.join(output,'hover.png')});
    // Negative control: visibly shift one label over another; detector must reject.
    await page.evaluate(()=>{
      const texts=document.querySelector('#U1 svg').querySelectorAll('text');
      texts[1].setAttribute('x',texts[0].getAttribute('x'));
      texts[1].setAttribute('y',texts[0].getAttribute('y'));
    });
    assert((await inspect(page)).some(p=>p.type==='text-overlap'));
    await page.reload();
    await page.evaluate(()=>document.fonts.ready);
    assert.deepEqual(await inspect(page),[]);
    await page.setViewportSize({width:390,height:844});
    await page.getByRole('navigation').getByRole('link',{name:'C2',exact:true}).click();
    const sizing=await page.evaluate(()=>({page:document.documentElement.scrollWidth,width:innerWidth,
      diagram:document.querySelector('#C2 .diagram').scrollWidth,
      visible:document.querySelector('#C2 .diagram').clientWidth}));
    assert(sizing.page<=sizing.width,JSON.stringify(sizing));
    assert(sizing.diagram>sizing.visible,'Diagram must scroll horizontally on a narrow screen');
    await page.locator('#C2 .diagram').evaluate(el=>{el.scrollLeft=el.scrollWidth;});
    await page.screenshot({path:path.join(output,'narrow.png')});
    assert.deepEqual(await inspect(page),[]);
    assert.deepEqual(errors,[]);
    assert.deepEqual(network,[]);
    assert.equal(hash(),startHash,'Document changed during browser review');
    console.log(JSON.stringify({document_only:true,flows:13,components:30,use_cases:16,
      artifact_sha256:startHash,
      svg_text_overlap:0,negative_overlap_detected:true,console_page_request_errors:errors.length,
      remote_requests:network.length,narrow:sizing,browser:'Chromium; WebKit unavailable'}));
    console.log('GATE_EXIT=0');
  } finally { await browser.close(); }
})().catch(error=>{console.error(error);console.log('GATE_EXIT=1');process.exitCode=1;});
