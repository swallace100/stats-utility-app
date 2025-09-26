# Docker

## Running

```bash
docker compose up -d
docker compose logs -f db
docker compose ps
```

## Contracts (TypeScript)

```ts
export const TTestTwoSampleParams = z.object({
  xColumn: z.string(),
  yColumn: z.string(),
  equalVariances: z.boolean().default(false),
  alternative: z.enum(["two-sided", "less", "greater"]).default("two-sided"),
});
```
