// Document QA only: no product UC, business data, CLI or model calls.
const {chromium, webkit} = require('playwright');
const {pathToFileURL} = require('node:url');
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const {reviewWorkspace}=require('./workspace_review.cjs');
const root = path.resolve(__dirname, '../..');
const engine = process.env.HUBA_REVIEW_BROWSER || 'chromium';
assert(['chromium','webkit'].includes(engine),'Unsupported review browser');
const output = path.join(root, 'docs/flows/previews/v3-layout-20261116',engine);

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
  const browser=await (engine==='webkit'?webkit:chromium).launch({headless:true,...(engine==='chromium'&&process.env.HUBA_CHROMIUM_EXECUTABLE?{executablePath:process.env.HUBA_CHROMIUM_EXECUTABLE}:{})});
  const page=await browser.newPage({viewport:{width:1440,height:1000}});
  const errors=[], network=[];
  const artifact = path.join(root,'docs/flows/index.html');
  // Số issue đọc từ nguồn thiết kế, KHÔNG viết cứng ở đây (19/11: thêm P13–P18 mà
  // bản cứng 12 làm bản tổng kết nói dối trong khi trang đã 18).
  const issueCount = JSON.parse(fs.readFileSync(path.join(__dirname,'design.json'),'utf8')).issues.length;
  const hash = () => crypto.createHash('sha256').update(fs.readFileSync(artifact)).digest('hex');
  const startHash = hash();
  page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  page.on('pageerror',e=>errors.push(e.message));
  page.on('requestfailed',r=>errors.push(r.url()+': '+r.failure()?.errorText));
  page.on('request',r=>{if(/^https?:/.test(r.url()))network.push(r.url());});
  try {
    await page.goto(pathToFileURL(path.join(root,'docs/flows/index.html')).href);
    await page.evaluate(()=>document.fonts.ready);
    assert.equal(await page.locator('svg').count(),19);
    assert.equal(await page.locator('#components tbody tr').count(),30);
    assert.equal(await page.locator('#uc tbody tr').count(),22);
    assert.equal(await page.locator('#issues article.issue').count(),issueCount);
    assert.deepEqual(await inspect(page),[]);
    const flowIds=['X0','U1','U2','U3','U4','U5','U6','U7','D1','D2','D3','D4','D5','D6','C1','C2','C3','C4','C5'];
    for (const id of flowIds) {
      await page.getByRole('navigation').getByRole('link',{name:id,exact:true}).click();
      assert.equal(await page.evaluate(()=>location.hash),'#'+id);
      const section=page.locator('#'+id);
      await section.locator('summary').click();
      assert.equal(await section.locator('details').getAttribute('open'),'');
      await section.locator('summary').click();
      await section.screenshot({path:path.join(output,id+'.png')});
    }
    for(const id of ['overview','workspace','terminal']) {
      await page.locator(`nav a[href="#${id}"]`).click();
      assert.equal(await page.evaluate(()=>location.hash),'#'+id);
      await page.locator('#'+id).screenshot({path:path.join(output,id+'.png')});
    }
    await page.locator('nav a[href="#workspace"]').click();
    const workspaceInteractions=await reviewWorkspace(page,output);
    for(let n=1;n<=issueCount;n++) {
      const id='P'+String(n).padStart(2,'0');
      await page.locator('nav a[href="#issues"]').click();
      const issue=page.locator('#'+id);
      const point=issue.locator('a').first();
      const target=await point.getAttribute('href');
      await point.click();
      assert.equal(await page.evaluate(()=>location.hash),target);
      const linkedFlow=target.slice(1).split('-step-')[0];
      await page.locator(`#${linkedFlow} .issue-links a[href="#${id}"]`).click();
      assert.equal(await page.evaluate(()=>location.hash),'#'+id);
      await issue.screenshot({path:path.join(output,id+'.png')});
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
    if(sizing.page>sizing.width) {
      console.error('VISIBLE_OVERFLOW',await page.locator('body *').evaluateAll(elements=>elements.filter(el=>{
        const box=el.getBoundingClientRect();
        if(box.right<=innerWidth||box.width===0) return false;
        for(let p=el.parentElement;p;p=p.parentElement) if(['auto','hidden','scroll','clip'].includes(getComputedStyle(p).overflowX)) return false;
        return true;
      }).map(el=>({tag:el.tagName,id:el.id,class:el.className,text:el.textContent.slice(0,90),right:el.getBoundingClientRect().right})).slice(0,25)));
      await page.screenshot({path:path.join(output,'narrow-failure.png')});
    }
    assert(sizing.page<=sizing.width,JSON.stringify(sizing));
    assert(sizing.diagram>sizing.visible,'Diagram must scroll horizontally on a narrow screen');
    await page.locator('#C2 .diagram').evaluate(el=>{el.scrollLeft=el.scrollWidth;});
    await page.screenshot({path:path.join(output,'narrow.png')});
    await page.locator('nav a[href="#workspace"]').click();
    const workspaceSize=await page.evaluate(()=>({page:document.documentElement.scrollWidth,width:innerWidth,
      wire:document.querySelector('.desk-viewport').scrollWidth,visible:document.querySelector('.desk-viewport').clientWidth}));
    assert(workspaceSize.page<=workspaceSize.width,JSON.stringify(workspaceSize));
    assert(workspaceSize.wire>workspaceSize.visible,'Design wireframe must scroll inside its region');
    await page.screenshot({path:path.join(output,'workspace-narrow.png')});
    await page.setViewportSize({width:1440,height:1100});
    const standalonePath=path.join(root,'docs/flows/workspace.html');
    const standaloneHash=crypto.createHash('sha256').update(fs.readFileSync(standalonePath)).digest('hex');
    await page.goto(pathToFileURL(standalonePath).href);
    await page.evaluate(()=>document.fonts.ready);
    await page.locator('#workspace').screenshot({path:path.join(output,'workspace-standalone.png')});
    const standaloneInteractions=await reviewWorkspace(page,output,'standalone');
    assert.equal(crypto.createHash('sha256').update(fs.readFileSync(standalonePath)).digest('hex'),standaloneHash);
    assert.deepEqual(await inspect(page),[]);
    assert.deepEqual(errors,[]);
    assert.deepEqual(network,[]);
    assert.equal(hash(),startHash,'Document changed during browser review');
    console.log(JSON.stringify({document_only:true,flows:19,components:30,use_cases:22,issues:issueCount,
      artifact_sha256:startHash,
      svg_text_overlap:0,negative_overlap_detected:true,console_page_request_errors:errors.length,
      remote_requests:network.length,narrow:sizing,workspace_narrow:workspaceSize,
      workspace:workspaceInteractions,standalone:standaloneInteractions,standalone_sha256:standaloneHash,browser:engine}));
    console.log('GATE_EXIT=0');
  } finally { await browser.close(); }
})().catch(error=>{console.error(error);console.log('GATE_EXIT=1');process.exitCode=1;});
