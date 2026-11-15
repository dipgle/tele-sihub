// Prototype/document QA only; no live CLI/model/business-data acceptance.
const {defineConfig}=require('playwright/test');
const path=require('node:path');
module.exports=defineConfig({
  testDir:__dirname,testMatch:'prototype.spec.cjs',workers:1,retries:0,
  reporter:'line',outputDir:path.resolve(__dirname,'../../../.tmp/ui-test-results'),
  timeout:30000,
  use:{viewport:{width:1440,height:1000},headless:true,
    launchOptions:process.env.HUBA_CHROMIUM_EXECUTABLE?
      {executablePath:process.env.HUBA_CHROMIUM_EXECUTABLE}:{}}
});
