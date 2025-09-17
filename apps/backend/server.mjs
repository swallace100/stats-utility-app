import http from "http";

const port = process.env.PORT || 8080;

// basic JSON logger
function log(level, msg, extra = {}) {
  console.log(
    JSON.stringify({ level, msg, ...extra, ts: new Date().toISOString() })
  );
}

const server = http.createServer(async (req, res) => {
  if (req.url === "/healthz") {
    res.writeHead(200, { "content-type": "application/json" });
    return res.end(JSON.stringify({ ok: true }));
  }
  // app routes go here…
  res.end("backend ok");
});

server.listen(port, () => log("info", "backend_started", { port }));
