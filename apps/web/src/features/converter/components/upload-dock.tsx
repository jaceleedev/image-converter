import { Chip } from "@heroui/react";
import { FileImageIcon, UploadCloudIcon } from "lucide-react";
import type * as React from "react";

import { supportedInputExtensions } from "../constants";
import { dimensionLabel, formatBytes, inputExtensionLabel } from "../lib/format";
import type { ImageMeta } from "../types";
import { cn } from "@/lib/utils";

export function UploadDock({
  file,
  inputRef,
  isBusy,
  isDragging,
  meta,
  selectFile,
  setIsDragging,
}: {
  file: File | null;
  inputRef: React.RefObject<HTMLInputElement | null>;
  isBusy: boolean;
  isDragging: boolean;
  meta: ImageMeta | null;
  selectFile: (file: File | null) => Promise<void>;
  setIsDragging: (value: boolean) => void;
}) {
  return (
    <button
      type="button"
      aria-busy={isBusy}
      aria-disabled={isBusy}
      aria-label="변환할 이미지 선택"
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
        "group w-full rounded-lg border p-4 text-left transition",
        "focus-visible:ring-3 focus-visible:ring-focus/40 focus-visible:outline-hidden",
        isDragging
          ? "border-accent bg-accent/15"
          : "border-border bg-surface-secondary hover:border-accent/70 hover:bg-surface-tertiary",
        isBusy && "cursor-wait opacity-75"
      )}
    >
      <div className="flex items-start gap-3">
        <div className="grid size-11 shrink-0 place-items-center rounded-lg border border-accent/20 bg-accent/10 text-accent shadow-field">
          {file ? (
            <FileImageIcon className="size-5" />
          ) : (
            <UploadCloudIcon className="size-5" />
          )}
        </div>
        <div className="min-w-0 flex-1">
          <div className="truncate text-sm font-semibold">
            {file ? file.name : "이미지 선택"}
          </div>
          <div className="mt-1 text-xs text-muted">
            {file
              ? `${formatBytes(file.size)} · ${dimensionLabel(meta?.width ?? null, meta?.height ?? null)}`
              : "드롭 또는 클릭"}
          </div>
        </div>
      </div>
      <div className="mt-4 flex flex-wrap gap-1.5">
        {file ? (
          <>
            <Chip size="sm" variant="soft">
              {inputExtensionLabel(file.name)}
            </Chip>
            <Chip size="sm" variant="soft">
              {meta ? "미리보기 가능" : "미리보기 없음"}
            </Chip>
          </>
        ) : (
          supportedInputExtensions.slice(0, 6).map((extension) => (
            <Chip key={extension} size="sm" variant="secondary">
              .{extension}
            </Chip>
          ))
        )}
      </div>
    </button>
  );
}
