import { ProgressBar, Spinner, Surface } from "@heroui/react";

export function ProgressPanel({ progress }: { progress: number }) {
  return (
    <Surface
      className="mt-3 animate-panel-in rounded-lg border border-border bg-surface-secondary px-4 py-3"
      variant="default"
    >
      <div className="mb-3 flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 text-sm font-medium">
          <Spinner color="accent" size="sm" />
          변환 중
        </div>
        <span className="font-mono text-xs text-muted">{progress}%</span>
      </div>
      <ProgressBar aria-label="변환 진행률" value={progress}>
        <ProgressBar.Track>
          <ProgressBar.Fill />
        </ProgressBar.Track>
      </ProgressBar>
    </Surface>
  );
}
