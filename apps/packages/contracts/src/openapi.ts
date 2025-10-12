import { z } from "zod";
import {
  OpenAPIRegistry,
  OpenApiGeneratorV3,
  extendZodWithOpenApi,
} from "@asteasolutions/zod-to-openapi";
import { UploadJobInput, UploadResponse, JobItem } from "./job";

extendZodWithOpenApi(z);

const registry = new OpenAPIRegistry();

// Register raw schemas
registry.register("UploadJobInput", UploadJobInput);
registry.register("UploadResponse", UploadResponse);
registry.register("JobItem", JobItem);

const JobList = z.array(JobItem);
const JobIdParam = z.object({ jobId: z.string() });

const UploadMultipart = z.object({
  file: z.string().openapi({ type: "string", format: "binary" }),
  metadata: UploadJobInput, // ✅ keep schema itself, not UploadJobInput.openapi()
});

registry.registerPath({
  method: "post",
  path: "/upload",
  request: {
    body: {
      required: true,
      content: {
        "multipart/form-data": { schema: UploadMultipart },
      },
    },
  },
  responses: {
    202: {
      description: "Accepted",
      content: { "application/json": { schema: UploadResponse } },
    },
    400: { description: "Bad Request" },
  },
});

registry.registerPath({
  method: "get",
  path: "/jobs",
  responses: {
    200: {
      description: "OK",
      content: { "application/json": { schema: JobList } },
    },
  },
});

registry.registerPath({
  method: "get",
  path: "/results/{jobId}",
  request: { params: JobIdParam },
  responses: {
    200: {
      description: "OK",
      content: {
        "application/json": {
          schema: z.object({
            jobId: z.string(),
            status: z.enum(["queued", "running", "succeeded", "failed"]),
            result: z.unknown().optional(),
            error: z.string().optional(),
          }),
        },
      },
    },
    404: { description: "Not Found" },
  },
});

export function generateOpenApi() {
  const generator = new OpenApiGeneratorV3(registry.definitions);
  return generator.generateDocument({
    openapi: "3.0.0",
    info: { title: "Stats Utility API", version: "1.0.0" },
    servers: [{ url: "http://localhost:8080" }],
  });
}
