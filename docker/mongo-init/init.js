// Uses root auth from env; runs once on first init
// Default DB name aligns with your .env: stats_artifacts
const dbName = process.env.MONGO_INITDB_DATABASE || "stats_artifacts";
const admin = connect("mongodb://127.0.0.1:27017/admin");

print(`Initializing Mongo database: ${dbName}`);
db = admin.getSiblingDB(dbName);

// Collections we expect
db.createCollection("results_blobs"); // full Rust outputs
db.createCollection("plot_specs"); // chart specs sent to Python
db.createCollection("run_contexts"); // versions, env, etc.

// Indexes (adjust as you like)
db.results_blobs.createIndex({ jobId: 1 }, { unique: true });
db.results_blobs.createIndex({ "result.p": 1 });
db.plot_specs.createIndex({ jobId: 1 }, { unique: true });
db.run_contexts.createIndex({ createdAt: -1 });

// Optional TTL example for ephemeral contexts (24h)
// db.run_contexts.createIndex({ createdAt: 1 }, { expireAfterSeconds: 86400 });

print("Mongo init complete.");
