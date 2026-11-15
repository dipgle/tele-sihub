// Fixture-only regression tests. Entry URL only; screens reached by UI clicks.
const {test,expect}=require('playwright/test');
const path=require('node:path');
const {pathToFileURL}=require('node:url');
const root=path.resolve(__dirname,'..');

function capture(page){
  const errors=[];
  page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  page.on('pageerror',e=>errors.push(e.message));
  page.on('requestfailed',r=>errors.push(r.url()+': '+r.failure()?.errorText));
  return errors;
}
async function menu(page,name){
  await page.getByRole('navigation').getByRole('link',{name,exact:true}).click();
}

test('Prototype: drafts, locked account, navigation, empty/offline',async({page})=>{
  const errors=capture(page);
  await page.goto(pathToFileURL(path.join(root,'index.html')).href);
  await expect(page.getByRole('heading',{name:'Thiết kế ứng dụng desktop'})).toBeVisible();
  await page.locator('#composer-text').fill('Nháp A');
  await page.locator('[data-action="select-session"][data-id="demo-review"]').click();
  await expect(page.locator('#composer-text')).toHaveValue('');
  await page.locator('#composer-text').fill('Nháp B');
  await page.locator('[data-action="select-session"][data-id="demo-ui"]').click();
  await expect(page.locator('#composer-text')).toHaveValue('Nháp A');
  for(const name of ['Phiên','Tài khoản & model','Kết nối','Dự án','Hoạt động','Cài đặt'])await menu(page,name);
  await menu(page,'Tài khoản & model');
  const locked=page.getByRole('row').filter({hasText:'Tài khoản D'});
  await locked.getByRole('button',{name:'Quản lý'}).click();
  await page.getByLabel('Model mặc định',{exact:true}).fill('Model minh họa');
  await page.getByRole('button',{name:'Lưu bản mẫu',exact:true}).click();
  await expect(locked).toContainText('Đang khoá');
  await menu(page,'Chat');
  await page.locator('#scenario').selectOption('empty');
  await expect(page.locator('#composer-text')).toBeDisabled();
  await expect(page.getByRole('button',{name:'↗ Terminal',exact:true})).toHaveCount(0);
  await page.locator('#scenario').selectOption('offline');
  await expect(page.getByText('Mất kết nối dịch vụ',{exact:false})).toBeVisible();
  await expect(page.locator('#composer-text')).toBeDisabled();
  expect(errors).toEqual([]);
});

test('Token fixture: charts, coverage, filters, CSV, unknown and errors',async({page})=>{
  const errors=capture(page);
  await page.goto(pathToFileURL(path.join(root,'index.html')).href);
  await menu(page,'Báo cáo token');
  await expect(page.locator('#chart-daily svg')).toBeVisible();
  await expect(page.locator('#chart-models svg')).toBeVisible();
  await expect(page.locator('.report-coverage').last()).toContainText('17/18');
  await expect(page.locator('.report-kpi').first()).toContainText('≥ 859.100');
  await page.locator('#chart-models path[fill="#438f7b"]').first().click();
  await page.getByRole('dialog').getByRole('button',{name:'Nguồn',exact:true}).first().click();
  await expect(page.locator('.report-raw')).toContainText('"source": "fixture"');
  await page.getByRole('button',{name:'Đóng hộp thoại',exact:true}).click();
  await page.getByLabel('Tài khoản',{exact:true}).selectOption({label:'Tài khoản A'});
  await expect(page.locator('.report-coverage').last()).toContainText('9/9');
  await expect(page.locator('.report-kpi').first()).toContainText('450.700');
  const pending=page.waitForEvent('download');
  await page.getByRole('button',{name:'↓ Xuất CSV mẫu',exact:true}).click();
  const stream=await(await pending).createReadStream();
  let csv='';for await(const chunk of stream)csv+=chunk.toString();
  expect(csv.trim().split(/\r?\n/)).toHaveLength(10);
  expect(csv).not.toContain('Tài khoản B');
  expect(csv).toContain('inputIncludesCache');
  await page.getByLabel('Tài khoản',{exact:true}).selectOption('all');
  await page.getByLabel('Khoảng thời gian',{exact:true}).selectOption('1');
  await page.getByLabel('Model',{exact:true}).selectOption({label:'Model B'});
  await expect(page.locator('.report-kpi strong').first()).toHaveText('Chưa đo');
  await expect(page.locator('.report-chart svg')).toHaveCount(0);
  await page.getByLabel('CLI',{exact:true}).selectOption({label:'Claude'});
  await expect(page.getByRole('heading',{name:'Chưa có dữ liệu token',exact:true})).toBeVisible();
  await page.locator('#scenario').selectOption('error');
  await expect(page.getByText('Chưa cập nhật được usage.',{exact:false})).toBeVisible();
  expect(errors).toEqual([]);
});

test('Export regression: embedded ECharts stays valid and fully offline',async({page})=>{
  const errors=capture(page),requests=[];
  page.on('request',r=>requests.push(r.url()));
  await page.goto(pathToFileURL(path.resolve(root,'../../.tmp/huba-desktop-design.html')).href);
  await menu(page,'Báo cáo token');
  await expect(page.locator('#chart-daily svg')).toBeVisible();
  await expect(page.locator('.report-kpi').first()).toContainText('≥ 859.100');
  expect(requests.length).toBeGreaterThan(0);
  expect(requests.every(url=>url.startsWith('file:')&&url.endsWith('huba-desktop-design.html'))).toBeTruthy();
  expect(errors).toEqual([]);
});
