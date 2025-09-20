import React from "react";

const apiBase = import.meta.env.VITE_API_URL ?? "http://localhost:8080";

export default function App() {
  return (
    <div style={{ fontFamily: "sans-serif", padding: 16 }}>
      <h1>Stats Utility (React + Vite)</h1>
      <p>Backend API base: <code>{apiBase}</code></p>
    </div>
  );
}
