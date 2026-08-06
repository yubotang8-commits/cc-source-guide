const fs = require('fs');
const https = require('https');
const path = require('path');

const files = [
  'commands/mobile/index.ts',
  'commands/plan/plan.tsx',
  'commands/plugin/UnifiedInstalledCell.tsx',
  'commands/remote-setup/index.ts',
  'commands/reset-limits/index.js',
  'commands/resume/index.ts',
  'commands/review/ultrareviewEnabled.ts',
  'services/PromptSuggestion/promptSuggestion.ts',
  'services/SessionMemory/prompts.ts',
  'services/autoDream/autoDream.ts',
  'utils/model/bedrock.ts',
  'utils/model/contextWindowUpgradeCheck.ts',
  'utils/permissions/dangerousPatterns.ts',
  'utils/permissions/filesystem.ts',
  'utils/plugins/officialMarketplace.ts',
  'utils/processUserInput/processTextPrompt.ts',
  'utils/proxy.ts',
  'utils/screenshotClipboard.ts',
  'utils/sessionFileAccessHooks.ts',
  'utils/settings/pluginOnlyPolicy.ts',
  'utils/statsCache.ts',
];

const BASE = 'https://raw.githubusercontent.com/Austin1serb/Anthropic-Leaked-Source-Code/main/';
const OUT = 'D:/agents/claude-code/';

function downloadOne(file) {
  return new Promise((resolve) => {
    const parts = file.split('/');
    const encoded = parts.map(p => encodeURIComponent(p)).join('/');
    const url = BASE + encoded;
    const outPath = path.join(OUT, file);

    const req = https.get(url, { rejectUnauthorized: false }, (res) => {
      if (res.statusCode === 200) {
        const chunks = [];
        res.on('data', chunk => chunks.push(chunk));
        res.on('end', () => {
          const buf = Buffer.concat(chunks);
          if (buf.length > 0) {
            fs.writeFileSync(outPath, buf);
            resolve({ file, ok: true, size: buf.length });
          } else {
            resolve({ file, ok: false, error: 'empty' });
          }
        });
      } else {
        res.resume();
        resolve({ file, ok: false, status: res.statusCode });
      }
    });
    req.on('error', (e) => resolve({ file, ok: false, error: e.message }));
    req.setTimeout(60000, () => { req.destroy(); resolve({ file, ok: false, error: 'timeout' }); });
  });
}

async function run() {
  // Download one at a time with 5s delay between each
  let ok = 0, fail = 0;
  const stillFailing = [];

  for (const file of files) {
    // Try up to 5 times
    let result = null;
    for (let attempt = 0; attempt < 5; attempt++) {
      result = await downloadOne(file);
      if (result.ok) break;
      console.log(`  Attempt ${attempt + 1} failed for ${file}: ${result.status || result.error}`);
      await new Promise(r => setTimeout(r, 5000));
    }
    if (result.ok) {
      ok++;
      console.log(`OK: ${file} (${result.size} bytes)`);
    } else {
      fail++;
      stillFailing.push(file);
      console.log(`FAIL: ${file}`);
    }
    await new Promise(r => setTimeout(r, 3000));
  }

  console.log(`\nFinal: Downloaded ${ok}, Failed ${fail}`);
  if (stillFailing.length > 0) {
    console.log('Still missing:', stillFailing.join('\n'));
  }
}

run();
