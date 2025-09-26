import * as React from "react";

type Props = {
  headers: string[] | null;
  rows: Array<Record<string, string> | string[]>;
};

export default function CSVPreview({ headers, rows }: Props) {
  if (!rows.length) return null;

  const first20 = rows.slice(0, 20);

  const colHeaders =
    headers ??
    (Array.isArray(first20[0])
      ? (first20[0] as string[]).map((_, i) => `Col ${i + 1}`)
      : Object.keys(first20[0] as Record<string, string>));

  return (
    <div className="mt-4 overflow-x-auto rounded-xl border">
      <table className="min-w-full text-sm">
        <thead className="bg-muted/40">
          <tr>
            {colHeaders.map((h) => (
              <th key={h} className="px-3 py-2 text-left font-medium">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {first20.map((row, i) => (
            <tr key={i} className="odd:bg-background even:bg-muted/10">
              {colHeaders.map((h, j) => (
                <td key={j} className="px-3 py-2">
                  {Array.isArray(row) ? (row[j] ?? "") : (row as any)[h] ?? ""}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      <div className="px-3 py-2 text-xs text-muted-foreground">
        Showing {first20.length} of {rows.length} rows
      </div>
    </div>
  );
}
