import Image from "next/image";
import { Button, Chip, ProgressBar, Spinner, Surface } from "@heroui/react";
import {
  ArrowDownToLineIcon,
  FileImageIcon,
  ImageIcon,
  ImageUpIcon,
  RotateCcwIcon,
  UploadCloudIcon,
  ZapIcon,
} from "lucide-react";
import type * as React from "react";

import { supportedInputExtensions } from "../constants";
import type {
  ConverterActions,
  ConverterDerived,
  ConverterState,
} from "../hooks/use-converter";
import { dimensionLabel, formatBytes } from "../lib/format";
import type { ImageMeta, ResultState } from "../types";
import { DownloadButton } from "./download-button";
import { FileStat } from "./file-stat";
import { ResultCard } from "./result-card";
import { stageDescription, statusColor } from "./status";
import { cn } from "@/lib/utils";

export function PreviewWorkspace({
  actions,
  derived,
  inputRef,
  state,
}: {
  actions: ConverterActions;
  derived: ConverterDerived;
  inputRef: React.RefObject<HTMLInputElement | null>;
  state: ConverterState;
}) {
  const {
    file,
    format,
    isDragging,
    meta,
    previewUrl,
    progress,
    result,
    stage,
  } = state;
  const { isBusy, reduction } = derived;
  const { reset, selectFile, setIsDragging } = actions;

  return (
    <Surface
      className="preview-shell relative flex min-h-[680px] overflow-hidden rounded-lg border border-border bg-surface shadow-surface xl:min-h-[calc(100vh-1.5rem)]"
      variant="default"
    >
      <div className="preview-backdrop pointer-events-none absolute inset-0" />
      <div className="relative flex w-full flex-col">
        <div className="flex flex-col gap-3 border-b border-separator bg-surface/86 px-4 py-3 backdrop-blur sm:flex-row sm:items-center sm:justify-between">
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <h2 className="text-sm font-semibold">스테이지</h2>
              <Chip color={statusColor(stage)} size="sm" variant="soft">
                {stageDescription(stage)}
              </Chip>
            </div>
            <p className="mt-1 truncate text-xs text-muted">
              {file ? file.name : "파일을 선택하거나 여기로 드롭하세요"}
            </p>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            <Chip size="sm" variant="secondary">
              {format.toUpperCase()}
            </Chip>
            <Button
              size="sm"
              type="button"
              variant="secondary"
              onPress={() => {
                if (!isBusy) {
                  inputRef.current?.click();
                }
              }}
            >
              <UploadCloudIcon data-icon="inline-start" />
              {file ? "교체" : "선택"}
            </Button>
          </div>
        </div>

        <button
          type="button"
          aria-busy={isBusy}
          aria-disabled={isBusy}
          aria-label="미리보기 스테이지"
          onClick={() => {
            if (!isBusy) {
              inputRef.current?.click();
            }
          }}
          onDragEnter={(event) => {
            event.preventDefault();
            if (!isBusy) {
              setIsDragging(true);
            }
          }}
          onDragOver={(event) => event.preventDefault()}
          onDragLeave={(event) => {
            event.preventDefault();
            setIsDragging(false);
          }}
          onDrop={(event) => {
            event.preventDefault();
            setIsDragging(false);
            if (!isBusy) {
              void selectFile(event.dataTransfer.files.item(0));
            }
          }}
          className={cn(
            "stage-grid relative flex min-h-[460px] flex-1 items-center justify-center overflow-hidden text-left transition",
            "focus-visible:ring-3 focus-visible:ring-focus/40 focus-visible:outline-hidden",
            isDragging ? "bg-accent/10 ring-2 ring-inset ring-accent" : "",
            isBusy && "cursor-wait"
          )}
        >
          <PreviewContent
            file={file}
            isDragging={isDragging}
            meta={meta}
            previewUrl={previewUrl}
            result={result}
          />
          {isBusy ? <StageProgress progress={progress} /> : null}
        </button>

        <StageTray
          file={file}
          format={format}
          meta={meta}
          reduction={reduction}
          reset={reset}
          result={result}
        />
      </div>
    </Surface>
  );
}

function PreviewContent({
  file,
  isDragging,
  meta,
  previewUrl,
  result,
}: {
  file: File | null;
  isDragging: boolean;
  meta: ImageMeta | null;
  previewUrl: string | null;
  result: ResultState | null;
}) {
  if (previewUrl && meta) {
    return (
      <div className="flex h-full w-full items-center justify-center p-4 sm:p-6 lg:p-8">
        <div className="image-checkerboard preview-frame relative aspect-[4/3] w-full max-w-6xl overflow-hidden rounded-lg border border-border bg-surface shadow-overlay">
          <Image
            src={previewUrl}
            alt={file?.name ?? "업로드 이미지"}
            fill
            sizes="(min-width: 1280px) 920px, (min-width: 768px) 70vw, 100vw"
            unoptimized
            className="object-contain p-3"
          />
          {result ? (
            <div className="absolute right-3 top-3 rounded-lg border border-success/30 bg-surface/90 px-3 py-2 text-xs font-semibold text-success shadow-surface backdrop-blur">
              완료
            </div>
          ) : null}
        </div>
      </div>
    );
  }

  if (file) {
    return (
      <div className="flex h-full w-full items-center justify-center p-6">
        <div className="max-w-2xl text-center">
          <div className="mx-auto grid size-16 place-items-center rounded-lg bg-warning/18 text-warning-foreground shadow-surface">
            <FileImageIcon className="size-8" />
          </div>
          <h3 className="mt-6 break-words text-3xl font-semibold sm:text-5xl">
            {file.name}
          </h3>
          <p className="mt-3 text-sm text-muted">미리보기 없음</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full w-full items-center justify-center p-6">
      <div className="max-w-xl text-center">
        <div className="mx-auto grid size-20 place-items-center rounded-lg bg-foreground text-background shadow-overlay">
          <ImageUpIcon className="size-10" />
        </div>
        <h3 className="mt-6 text-4xl font-semibold tracking-normal sm:text-6xl">
          {isDragging ? "놓으면 시작" : "이미지 없음"}
        </h3>
        <p className="mt-4 text-sm leading-6 text-muted">
          파일을 선택하거나 스테이지에 드롭하세요.
        </p>
      </div>
    </div>
  );
}

function StageProgress({ progress }: { progress: number }) {
  return (
    <div className="absolute inset-x-4 bottom-4 rounded-lg border border-border bg-overlay/92 p-3 shadow-overlay backdrop-blur">
      <div className="mb-2 flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 text-sm font-semibold">
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
    </div>
  );
}

function StageTray({
  file,
  format,
  meta,
  reduction,
  reset,
  result,
}: {
  file: File | null;
  format: ConverterState["format"];
  meta: ImageMeta | null;
  reduction: number | null;
  reset: () => void;
  result: ResultState | null;
}) {
  if (result) {
    return (
      <div className="border-t border-separator bg-overlay/88 p-3 backdrop-blur">
        <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center">
          <div className="grid gap-2 sm:grid-cols-3">
            <ResultCard
              icon={<ArrowDownToLineIcon className="size-5" />}
              label="결과 용량"
              tone="accent"
              value={formatBytes(result.size)}
            />
            <ResultCard
              icon={<ZapIcon className="size-5" />}
              label="감소율"
              tone={reduction !== null && reduction >= 0 ? "success" : "warning"}
              value={reduction === null ? "-" : `${reduction.toFixed(1)}%`}
            />
            <ResultCard
              icon={<ImageIcon className="size-5" />}
              label={`${format.toUpperCase()} 크기`}
              tone="default"
              value={dimensionLabel(result.outputWidth, result.outputHeight)}
            />
          </div>
          <div className="grid grid-cols-[1fr_auto] gap-2 lg:w-56">
            <DownloadButton fullWidth result={result} />
            <Button
              isIconOnly
              aria-label="초기화"
              type="button"
              variant="tertiary"
              onPress={reset}
            >
              <RotateCcwIcon />
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="border-t border-separator bg-surface/86 p-3 backdrop-blur">
      {file ? (
        <div className="grid gap-2 sm:grid-cols-3">
          <FileStat label="파일" value={file.name} />
          <FileStat
            label="크기"
            value={meta ? `${meta.width} x ${meta.height}` : "미리보기 없음"}
          />
          <FileStat label="용량" value={formatBytes(file.size)} />
        </div>
      ) : (
        <div className="flex flex-wrap gap-2">
          {supportedInputExtensions.slice(0, 8).map((extension) => (
            <Chip key={extension} size="sm" variant="secondary">
              .{extension}
            </Chip>
          ))}
          <Chip size="sm" variant="secondary">
            +2
          </Chip>
        </div>
      )}
    </div>
  );
}
