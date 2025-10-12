import * as React from "react";

type Cell = string | number | boolean | null | undefined;
type ArrayRow = readonly Cell[];
type ObjectRow = Readonly<Record<string, Cell>>;
type Row = ArrayRow | ObjectRow;

type Props = {
  headers: string[] | null;
  rows: Row[];
};

const isObjectRow = (r: Row): r is ObjectRow => !Array.isArray(r);

const getCell = (row: Row, j: number, h: string): Cell =>
  isObjectRow(row) ? (row[h] ?? "") : (row[j] ?? "");

export default function CSVPreview({ headers, rows }: Props) {
  // Hooks must be called unconditionally on every render
  const first20 = React.useMemo(() => rows.slice(0, 20), [rows]);

  const colHeaders: string[] = React.useMemo(() => {
    if (headers) return headers;
    const first = first20[0];
    if (!first) return [];
    return Array.isArray(first)
      ? first.map((_, i) => `Col ${i + 1}`)
      : Object.keys(first);
  }, [headers, first20]);

  // Now it’s safe to return early
  if (rows.length === 0) return null;

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
                <td key={h} className="px-3 py-2">
                  {getCell(row, j, h)}
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
