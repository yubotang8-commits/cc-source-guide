const http = require("http");
const fs = require("fs");
const path = require("path");
const net = require("net");

const DEFAULT_PORT = 8080;
const GUIDE_DIR = __dirname;

// 从命令行获取端口：node server.js 9000
const argPort = parseInt(process.argv[2], 10);

function isPortFree(port) {
  return new Promise(resolve => {
    const s = net.createServer();
    s.on("error", () => resolve(false));
    s.listen(port, "0.0.0.0", () => { s.close(() => resolve(true)); });
  });
}

async function findFreePort(start) {
  for (let p = start; p < start + 100; p++) {
    if (await isPortFree(p)) return p;
  }
  return null;
}

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".css":  "text/css; charset=utf-8",
  ".js":   "application/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".png":  "image/png",
  ".jpg":  "image/jpeg",
  ".svg":  "image/svg+xml",
  ".ico":  "image/x-icon",
};

const server = http.createServer((req, res) => {
  let urlPath = req.url.split("?")[0];
  if (urlPath === "/") urlPath = "/index.html";

  const filePath = path.join(GUIDE_DIR, decodeURIComponent(urlPath));

  // 防止路径穿越
  if (!filePath.startsWith(GUIDE_DIR)) {
    res.writeHead(403);
    res.end("403 Forbidden");
    return;
  }

  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
      res.end("404 Not Found");
      return;
    }
    const ext = path.extname(filePath).toLowerCase();
    res.writeHead(200, { "Content-Type": MIME[ext] || "application/octet-stream" });
    res.end(data);
  });
});

(async () => {
  const port = argPort || (await findFreePort(DEFAULT_PORT));
  if (!port) { console.error("找不到可用端口"); process.exit(1); }

  server.listen(port, "0.0.0.0", () => {
    const interfaces = require("os").networkInterfaces();
    const ips = [];
    for (const name of Object.keys(interfaces)) {
      for (const iface of interfaces[name]) {
        if (iface.family === "IPv4" && !iface.internal) ips.push(iface.address);
      }
    }
    console.log("=== Claude-Code 源码学习指南 服务器已启动 ===\n");
    console.log("本机访问: http://localhost:" + port);
    ips.forEach(ip => console.log("内网访问: http://" + ip + ":" + port));
    console.log("\n按 Ctrl+C 停止服务器");
  });
})();
