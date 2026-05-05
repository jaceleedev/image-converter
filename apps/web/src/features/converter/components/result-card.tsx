import { Card } from "@heroui/react";
import { cn } from "@/lib/utils";

export function ResultCard({
  icon,
  label,
  tone,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  tone: "accent" | "default" | "success" | "warning";
  value: string;
}) {
  const toneClass = {
    accent: "bg-accent/10 text-accent",
    default: "bg-surface-secondary text-foreground",
    success: "bg-success/12 text-success",
    warning: "bg-warning/16 text-warning-foreground",
  }[tone];

  return (
    <Card className="rounded-lg border border-border bg-surface p-3 shadow-surface">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="font-mono text-[11px] uppercase text-muted">
            {label}
          </div>
          <div className="mt-1 truncate text-xl font-semibold tabular-nums">
            {value}
          </div>
        </div>
        <div
          className={cn(
            "grid size-9 shrink-0 place-items-center rounded-lg",
            toneClass
          )}
        >
          {icon}
        </div>
      </div>
    </Card>
  );
}
