export function SettingBlock({
  children,
  icon,
  label,
  value,
}: {
  children: React.ReactNode;
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <section className="mb-3 rounded-lg border border-separator bg-surface-secondary/75 p-3">
      <div className="mb-3 flex items-center justify-between gap-3">
        <div className="flex min-w-0 items-center gap-2">
          <div className="grid size-8 shrink-0 place-items-center rounded-lg bg-surface-tertiary text-accent">
            {icon}
          </div>
          <h3 className="truncate text-sm font-semibold">{label}</h3>
        </div>
        <span className="shrink-0 rounded-md bg-surface-tertiary px-2 py-1 font-mono text-xs text-muted">
          {value}
        </span>
      </div>
      {children}
    </section>
  );
}
