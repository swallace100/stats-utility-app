import { z } from "zod";
import { extendZodWithOpenApi } from "@asteasolutions/zod-to-openapi";

// Extend BEFORE creating any schemas
extendZodWithOpenApi(z);

export const JobKind = z.enum(["stats", "plot"]);

export const UploadJobInput = z.object({
  kind: JobKind,
  params: z.record(z.string(), z.unknown()).optional(),
});

export const UploadResponse = z.object({
  jobId: z.string(),
  status: z.enum(["queued", "running", "succeeded", "failed"]),
});

export const JobItem = z.object({
  jobId: z.string(),
  kind: JobKind,
  status: z.enum(["queued", "running", "succeeded", "failed"]),
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
  error: z.string().optional(),
  result: z.unknown().optional(),
});

export type TUploadJobInput = z.infer<typeof UploadJobInput>;
export type TUploadResponse = z.infer<typeof UploadResponse>;
export type TJobItem = z.infer<typeof JobItem>;
