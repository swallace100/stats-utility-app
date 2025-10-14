import express from "express";
import swaggerUi from "swagger-ui-express";

// Minimal OpenAPI document describing all backend routes.
// Later you can generate this dynamically if you have @your-scope/contracts.
const openApiDoc = {
  openapi: "3.0.1",
  info: {
    title: "Stats Utility Backend API",
    version: "1.0.0",
    description:
      "Backend for Stats Utility App — proxies CSV/JSON data to stats_rs and plots_py microservices.",
  },
  paths: {
    "/health": {
      get: {
        summary: "Health check",
        responses: { 200: { description: "OK" } },
      },
    },
    "/upload": {
      post: {
        summary: "Upload CSV + metadata",
        responses: { 202: { description: "Queued" } },
      },
    },
    "/jobs": {
      get: {
        summary: "List recent jobs",
        responses: { 200: { description: "Array of jobs" } },
      },
    },
    "/results/{jobId}": {
      get: {
        summary: "Get job result",
        parameters: [{ name: "jobId", in: "path", required: true }],
        responses: { 200: { description: "Job result" } },
      },
    },
    "/analyze/summary": {
      post: {
        summary: "CSV → Summary stats (stats_rs)",
        responses: { 200: { description: "Summary JSON" } },
      },
    },
    "/analyze/distribution": {
      post: {
        summary: "CSV → Distribution stats (stats_rs)",
        responses: { 200: { description: "Distribution JSON" } },
      },
    },
    "/plot/summary": {
      post: {
        summary: "JSON → Summary plot (plots_py)",
        responses: { 200: { description: "PNG image" } },
      },
    },
    "/plot/distribution": {
      post: {
        summary: "JSON → Distribution plot (plots_py)",
        responses: { 200: { description: "PNG image" } },
      },
    },
    "/plot/ecdf": {
      post: {
        summary: "JSON → ECDF plot (plots_py)",
        responses: { 200: { description: "PNG image" } },
      },
    },
    "/plot/qq": {
      post: {
        summary: "JSON → QQ plot (plots_py)",
        responses: { 200: { description: "PNG image" } },
      },
    },
  },
};

export function registerDocs(app: express.Express) {
  // Serve raw JSON for tooling
  app.get("/openapi.json", (_req, res) => res.json(openApiDoc));

  // Serve Swagger UI at /docs
  app.use(
    "/docs",
    swaggerUi.serve,
    swaggerUi.setup(openApiDoc, {
      explorer: true,
      customSiteTitle: "Stats Utility Docs",
    }),
  );
}
