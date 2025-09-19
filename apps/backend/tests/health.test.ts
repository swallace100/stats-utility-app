import request from "supertest";
import { app } from "../src/app";

describe("GET /health", () => {
  it("returns 200 with ok payload", async () => {
    const res = await request(app).get("/health");
    expect(res.status).toBe(200);
    expect(res.body).toEqual({ status: "ok" });
  });
});
