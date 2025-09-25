import swaggerUi from "swagger-ui-express";
import express from "express";
import { generateOpenApi } from "@your-scope/contracts/src/openapi";

export function registerDocs(app: express.Express) {
  const doc = generateOpenApi();

  // serve raw JSON if you want
  app.get("/openapi.json", (_req, res) => res.json(doc));

  // serve Swagger UI
  app.use("/docs", swaggerUi.serve, swaggerUi.setup(doc, {
    explorer: true,
  }));
}