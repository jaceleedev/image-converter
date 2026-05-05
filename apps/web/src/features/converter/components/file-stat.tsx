export function FileStat({
  label,
  value,
}: {
  label: string;
  value: string;
}) {
  return (
    <div className="min-w-0 rounded-lg border border-separator bg-surface-secondary px-3 py-2">
      <div className="font-mono text-[11px] uppercase text-muted">{label}</div>
      <div className="mt-1 truncate text-sm font-medium" title={value}>
        {value}
      </div>
    </div>
  );
}
