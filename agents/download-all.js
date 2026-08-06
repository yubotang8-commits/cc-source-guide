const fs = require('fs');
const https = require('https');
const path = require('path');

const files = fs.readFileSync('D:/agents/claude-code-filelist.txt', 'utf8').trim().split('\n');
const BASE = 'https://raw.githubusercontent.com/Austin1serb/Anthropic-Leaked-Source-Code/main/';
const OUT = 'D:/agents/claude-code/';

async function downloadFile(file) {
  const url = BASE + encodeURIComponent(file).replace(/%2F/g, '/');
  const outPath = path.join(OUT, file);
  
  return new Promise((resolve) => {
    const req = https.get(url, { rejectUnauthorized: false }, (res) => {
      if (res.statusCode === 200) {
        const chunks = [];
        res.on('data', chunk => chunks.push(chunk));
        res.on('end', () => {
          fs.writeFileSync(outPath, Buffer.concat(chunks));
          resolve({ file, ok: true });
        });
      } else {
        res.resume();
        resolve({ file, ok: false, status: res.statusCode });
      }
    });
    req.on('error', (e) => resolve({ file, ok: false, error: e.message }));
    req.setTimeout(15000, () => { req.destroy(); resolve({ file, ok: false, error: 'timeout' }); });
  });
}

async function run() {
  const CONCURRENCY = 30;
  let downloaded = 0;
  let failed = 0;
  const failedFiles = [];
  
  for (let i = 0; i < files.length; i += CONCURRENCY) {
    const batch = files.slice(i, i + CONCURRENCY);
    const results = await Promise.all(batch.map(f => downloadFile(f)));
    
    for (const r of results) {
      if (r.ok) {
        downloaded++;
      } else {
        failed++;
        failedFiles.push(r.file + ' (' + (r.status || r.error) + ')');
      }
    }
    
    if ((i / CONCURRENCY) % 10 === 0) {
      console.log(`Progress: ${downloaded + failed}/${files.length} (ok: ${downloaded}, fail: ${failed})`);
    }
  }
  
  console.log(`\nDone! Downloaded: ${downloaded}, Failed: ${failed}`);
  if (failedFiles.length > 0 && failedFiles.length <= 50) {
    console.log('Failed files:', failedFiles.join('\n'));
  } else if (failedFiles.length > 50) {
    console.log('First 50 failed files:', failedFiles.slice(0, 50).join('\n'));
  }
}

run();
