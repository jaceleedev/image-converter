export function SectionTitle({
  icon,
  label,
}: {
  icon: React.ReactNode;
  label: string;
}) {
  return (
    <div className="flex min-w-0 items-center gap-2">
      <div className="grid size-8 shrink-0 place-items-center rounded-lg bg-surface-tertiary text-accent">
        {icon}
      </div>
      <h2 className="truncate text-sm font-semibold">{label}</h2>
    </div>
  );
}
