// Interactions with the local design fixture only, never live CLI/business data.
const assert=require('node:assert/strict');
const path=require('node:path');

async function footerGeometry(page){
 return page.locator('.desk-shell').evaluate(shell=>{
  const s=shell.getBoundingClientRect(),body=shell.querySelector('.desk-body').getBoundingClientRect(),footer=shell.querySelector('.desk-footer').getBoundingClientRect();
  return {full:Math.abs(s.x-footer.x)<2&&Math.abs(s.width-footer.width)<2,below:footer.y>=body.bottom-1,
   shell:{x:s.x,width:s.width},footer:{x:footer.x,width:footer.width,y:footer.y,bottom:footer.bottom},bodyBottom:body.bottom};
 });
}

async function reviewWorkspace(page,output,suffix='embedded'){
 const root=page.locator('[data-desk-demo]');
 assert.equal(await root.count(),1);
 assert.equal(await root.locator('[data-session]').count(),4);
 assert.equal(await root.locator('textarea').count(),4);
 assert.equal(await root.locator('[data-open]').count(),3);
 for(const id of ['S01','S02','S03','S04']){
  assert(await root.locator(`[data-activity="${id}"]`).isVisible());
  assert(await root.locator(`[data-chat="${id}"]`).isVisible());
  assert(await root.locator(`[data-draft="${id}"]`).isVisible());
 }
 const geometry=await footerGeometry(page);
 assert(geometry.full&&geometry.below,JSON.stringify(geometry));
 // Negative control must fail when B is put inside the central working area.
 await root.locator('.desk-footer').evaluate(footer=>footer.closest('.desk-shell').querySelector('.desk-main').append(footer));
 const broken=await footerGeometry(page);
 assert(!broken.full||!broken.below,'Misplaced footer must be detected');
 await root.locator('.desk-footer').evaluate(footer=>footer.closest('.desk-shell').append(footer));
 const restored=await footerGeometry(page);
 assert(restored.full&&restored.below,'Restored full-width footer must pass');

 await root.locator('[data-draft="S02"]').fill('Nháp riêng S02 — chưa gửi');
 await root.locator('[data-focus="S03"]').click();
 assert.equal(await root.locator('[data-detail]').getAttribute('data-detail-session'),'S03');
 assert.equal(await root.locator('[data-draft="S02"]').inputValue(),'Nháp riêng S02 — chưa gửi');
 assert.equal(await root.locator('[data-draft="S01"]').inputValue(),'');
 await root.locator('[data-focus="S01"]').hover();
 assert.equal(await root.locator('[data-detail]').getAttribute('data-detail-session'),'S03','Hover must not steal focus');
 await root.locator('[data-simulate]').click();
 assert.equal(await root.locator('[data-detail]').getAttribute('data-detail-session'),'S03','Background update must not change selected session');
 assert((await root.locator('[data-activity="S01"]').innerText()).includes('mẫu #1'));
 assert((await root.locator('[data-chat="S01"]').innerText()).includes('không cần focus · mẫu #1'));
 assert((await root.locator('[data-open="S05"]').innerText()).includes('kết quả kiểm tra mẫu #1'));
 assert.equal(await root.locator('[data-open="S05"] .desk-icon').getAttribute('data-state'),'running');
 assert.equal(await root.locator('[data-draft="S02"]').inputValue(),'Nháp riêng S02 — chưa gửi');
 await page.locator('#workspace').screenshot({path:path.join(output,'workspace-'+suffix+'-focused-updated.png')});

 await root.locator('[data-draft="S02"]').focus();
 assert.equal(await root.locator('[data-detail]').getAttribute('data-detail-session'),'S02','Keyboard/input focus must update details');
 const hostile='<img src=x onerror="window.DESIGN_INJECTED=1"> chỉ là chữ mẫu';
 await root.locator('[data-draft="S02"]').fill(hostile);
 await root.locator('[data-compose="S02"] button').click();
 assert((await root.locator('[data-chat="S02"]').innerText()).includes(hostile));
 assert.equal(await root.locator('[data-chat="S02"] img').count(),0);
 assert.equal(await page.evaluate(()=>window.DESIGN_INJECTED),undefined);
 assert((await root.locator('[data-notice="S02"]').innerText()).includes('chưa gửi CLI'));

 await root.locator('[data-open="S05"]').click();
 assert.equal(await root.locator('[data-session]').count(),5);
 assert.equal(await root.locator('[data-open]').count(),2);
 assert.equal(await root.locator('[data-detail]').getAttribute('data-detail-session'),'S05');
 assert((await root.locator('[data-panel-count]').innerText()).includes('5'));
 assert((await root.locator('[data-total-count]').innerText()).includes('A 5 + B 2'));
 await root.locator('[data-minimize="S05"]').click();
 assert.equal(await root.locator('[data-session]').count(),4);
 assert.equal(await root.locator('[data-open]').count(),3);
 assert((await root.locator('[data-open="S05"]').innerText()).includes('mẫu #1'));
 await root.locator('[data-draft="S04"]').fill('Nháp S04 qua thu/mở ô chat');
 await root.locator('[data-minimize="S04"]').click();
 await root.locator('[data-open="S04"]').click();
 assert.equal(await root.locator('[data-draft="S04"]').inputValue(),'Nháp S04 qua thu/mở ô chat');
 assert.equal(await root.locator('[data-detail]').getAttribute('data-detail-session'),'S04');
 await root.locator('[data-open="S05"]').hover();
 await page.locator('#workspace').screenshot({path:path.join(output,'workspace-'+suffix+'-hover-drafts.png')});
 return {panels:4,footer_sessions:3,full_width_footer:true,negative_footer_detected:true,independent_drafts:true,
  focus_details:true,background_updates_without_focus:true,local_demo_send_not_cli:true,source_text_escaped:true};
}

async function reviewPinnedFooter(page,output){
 const measured=[];
 for(const size of [{width:1440,height:900},{width:1280,height:720},{width:1440,height:600}]){
  await page.setViewportSize(size);
  const measure=()=>page.locator('.desk-shell').evaluate(shell=>{
   const f=shell.querySelector('.desk-footer').getBoundingClientRect(),h=shell.querySelector('.desk-header').getBoundingClientRect(),d=shell.querySelector('.desk-detail').getBoundingClientRect(),a=shell.querySelector('.desk-panels');
   return {viewport:innerHeight,page:document.documentElement.scrollHeight,windowY:scrollY,
    footerY:f.y,footerBottom:f.bottom,headerY:h.y,detailY:d.y,detailBottom:d.bottom,aBottom:a.getBoundingClientRect().bottom,aTop:a.scrollTop,aHeight:a.clientHeight,aScroll:a.scrollHeight};
  });
  const before=await measure();
  assert(before.footerY>=0&&before.footerBottom<=size.height+1,JSON.stringify({reason:'B must be visible without page scrolling',size,...before}));
  assert(before.page<=size.height+1&&before.windowY===0,JSON.stringify({reason:'Only A scrolls, not the document',size,...before}));
  assert(before.aBottom<=before.footerY+1&&before.detailBottom<=before.footerY+1,JSON.stringify({reason:'A and details must not spill over pinned B',size,...before}));
  assert(before.aScroll>before.aHeight,'Multiple A panels must overflow inside A at this window height');
  const box=await page.locator('.desk-panels').boundingBox();
  await page.mouse.move(box.x+box.width/2,box.y+20);
  await page.mouse.wheel(0,-3000);
  await page.mouse.wheel(0,420);
  await page.waitForFunction(()=>document.querySelector('.desk-panels').scrollTop>0);
  const after=await measure();
  assert(after.aTop>0,'Wheel must scroll A');
  assert.equal(after.windowY,0);
  assert(Math.abs(before.footerY-after.footerY)<1&&Math.abs(before.footerBottom-after.footerBottom)<1,'A scrolling moved B');
  assert(Math.abs(before.headerY-after.headerY)<1&&Math.abs(before.detailY-after.detailY)<1,'A scrolling moved fixed neighboring regions');
  await page.locator('#workspace').screenshot({path:path.join(output,`pinned-B-A-scrolled-${size.width}x${size.height}.png`)});
  measured.push({size,...after});
 }
 // Negative control: reintroduce an oversized shell; the visibility test must fail.
 const original=await page.locator('.desk-shell').getAttribute('style');
 await page.locator('.desk-shell').evaluate(shell=>{shell.style.height='1400px';shell.style.flex='0 0 1400px';});
 const broken=await page.locator('.desk-footer').boundingBox();
 assert(broken.y+broken.height>600,'Oversized shell must hide B and be detected');
 await page.locator('.desk-shell').evaluate((shell,style)=>{if(style===null)shell.removeAttribute('style');else shell.setAttribute('style',style);},original);
 const restored=await page.locator('.desk-footer').boundingBox();
 assert(restored.y>=0&&restored.y+restored.height<=601,'Restored B must be visible');
 return {document_only:true,B_always_visible:true,only_A_scrolls:true,negative_oversize_detected:true,windows:measured};
}

module.exports={reviewWorkspace,footerGeometry,reviewPinnedFooter};
