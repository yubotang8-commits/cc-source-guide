const fs = require('fs');
const https = require('https');
const path = require('path');

const files = fs.readFileSync('D:/agents/claude-code-missing.txt', 'utf8').trim().split('\n');
const BASE = 'https://raw.githubusercontent.com/Austin1serb/Anthropic-Leaked-Source-Code/main/';
const OUT = 'D:/agents/claude-code/';
const MAX_RETRIES = 3;

async function downloadFile(file, retries = 0) {
  const parts = file.split('/');
  const encoded = parts.map(p => encodeURIComponent(p)).join('/');
  const url = BASE + encoded;
  const outPath = path.join(OUT, file);

  return new Promise((resolve) => {
    const req = https.get(url, { rejectUnauthorized: false }, (res) => {
      if (res.statusCode === 200) {
        const chunks = [];
        res.on('data', chunk => chunks.push(chunk));
        res.on('end', () => {
          const buf = Buffer.concat(chunks);
          if (buf.length > 0) {
            fs.writeFileSync(outPath, buf);
            resolve({ file, ok: true });
          } else {
            resolve({ file, ok: false, error: 'empty response' });
          }
        });
      } else if (res.statusCode === 429 || res.statusCode === 502 || res.statusCode === 503) {
        res.resume();
        if (retries < MAX_RETRIES) {
          // Rate limited - wait and retry
          const delay = 2000 * (retries + 1);
          setTimeout(() => {
            downloadFile(file, retries + 1).then(resolve);
          }, delay);
        } else {
          resolve({ file, ok: false, status: res.statusCode, retries });
        }
      } else {
        res.resume();
        resolve({ file, ok: false, status: res.statusCode });
      }
    });
    req.on('error', (e) => {
      if (retries < MAX_RETRIES) {
        setTimeout(() => {
          downloadFile(file, retries + 1).then(resolve);
        }, 2000 * (retries + 1));
      } else {
        resolve({ file, ok: false, error: e.message, retries });
      }
    });
    req.setTimeout(30000, () => {
      req.destroy();
      if (retries < MAX_RETRIES) {
        setTimeout(() => {
          downloadFile(file, retries + 1).then(resolve);
        }, 2000 * (retries + 1));
      } else {
        resolve({ file, ok: false, error: 'timeout', retries });
      }
    });
  });
}

async function run() {
  const CONCURRENCY = 10; // Lower concurrency to avoid rate limiting
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
        failedFiles.push(r.file + ' (' + (r.status || r.error) + ', retries: ' + (r.retries || 0) + ')');
      }
    }

    if ((i / CONCURRENCY) % 5 === 0) {
      console.log(`Progress: ${downloaded + failed}/${files.length} (ok: ${downloaded}, fail: ${failed})`);
    }

    // Small delay between batches to avoid rate limiting
    await new Promise(r => setTimeout(r, 500));
  }

  console.log(`\nDone! Downloaded: ${downloaded}, Failed: ${failed}`);
  if (failedFiles.length > 0 && failedFiles.length <= 100) {
    console.log('Failed files:', failedFiles.join('\n'));
  } else if (failedFiles.length > 100) {
    console.log('First 50 failed files:', failedFiles.slice(0, 50).join('\n'));
  }

  // Save still-failed files for another retry
  fs.writeFileSync('D:/agents/claude-code-still-missing.txt', failedFiles.map(f => f.split(' (')[0]).join('\n'));
}

run();
