import http from "http";
http.createServer((_, res) => res.end("backend ok")).listen(8080);
