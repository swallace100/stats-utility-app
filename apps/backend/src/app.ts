import express from "express";

export const app = express();

app.get("/health", (_req, res) => {
  res.json({ status: "ok" });
});

// add other routes here
