import { useEffect, useState } from "react";
import NavBar from "./components/NavBar";
import { ping } from "./lib/api";

export default function App() {
  const [healthy, setHealthy] = useState<boolean | null>(null);

  useEffect(() => {
    ping().then((r) => setHealthy(r.ok)).catch(() => setHealthy(false));
  }, []);

  return (
    <div className="min-h-screen flex flex-col">
      <NavBar />
      <main className="flex-1 mx-auto max-w-6xl px-4 py-6">
        <h1 className="text-2xl font-semibold mb-2">Welcome 👋</h1>
        <p className="text-sm text-muted-foreground">
          Backend health:{" "}
          <span className={healthy ? "text-green-600" : healthy === false ? "text-red-600" : ""}>
            {healthy === null ? "checking..." : healthy ? "OK" : "DOWN"}
          </span>
        </p>
      </main>
      <footer className="border-t">
        <div className="mx-auto max-w-6xl px-4 h-12 flex items-center text-xs text-muted-foreground">
          © {new Date().getFullYear()} Stats Utility
        </div>
      </footer>
    </div>
  );
}
