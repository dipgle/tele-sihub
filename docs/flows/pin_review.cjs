// Regression for viewport-pinned B, on the local prototype, not a product UC.
const {chromium,webkit}=require('playwright');
const {pathToFileURL}=require('node:url');
const fs=require('node:fs');
const path=require('node:path');
const assert=require('node:assert/strict');
const crypto=require('node:crypto');
const {reviewPinnedFooter}=require('./workspace_review.cjs');
(async()=>{
 const engine=process.env.HUBA_REVIEW_BROWSER||'chromium';
 assert(['chromium','webkit'].includes(engine));
 const artifact=path.join(__dirname,'workspace.html');
 const digest=()=>crypto.createHash('sha256').update(fs.readFileSync(artifact)).digest('hex');
 const initial=digest();
 const output=path.join(__dirname,'previews/pinned-B-20261116',engine);
 fs.mkdirSync(output,{recursive:true});
 const browser=await (engine==='webkit'?webkit:chromium).launch({headless:true});
 try{
  const page=await browser.newPage(),errors=[],remote=[];
  page.on('pageerror',e=>errors.push(e.message));
  page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  page.on('requestfailed',r=>errors.push(r.url()));
  page.on('request',r=>{if(/^https?:/.test(r.url()))remote.push(r.url());});
  await page.goto(pathToFileURL(artifact).href);
  await page.evaluate(()=>document.fonts.ready);
  const result=await reviewPinnedFooter(page,output);
  assert.deepEqual(errors,[]);assert.deepEqual(remote,[]);assert.equal(digest(),initial);
  console.log(JSON.stringify({...result,browser:engine,artifact_sha256:initial,console_page_request_errors:errors.length,remote_requests:remote.length}));
  console.log('GATE_EXIT=0');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);console.log('GATE_EXIT=1');process.exitCode=1;});
