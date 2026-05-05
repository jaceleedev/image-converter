"use client";

import Image from "next/image";
import {
  AlertCircleIcon,
  ArrowDownToLineIcon,
  FileImageIcon,
  GaugeIcon,
  ImageIcon,
  ImageUpIcon,
  PaletteIcon,
  RotateCcwIcon,
  RulerIcon,
  SlidersHorizontalIcon,
  SparklesIcon,
  UploadCloudIcon,
  ZapIcon,
} from "lucide-react";
import * as React from "react";

import {
  Alert,
  Button,
  Card,
  Chip,
  Input,
  Label,
  ProgressBar,
  Radio,
  RadioGroup,
  Separator,
  Slider,
  Spinner,
  Surface,
  Tooltip,
} from "@heroui/react";
import { cn } from "@/lib/utils";

type OutputFormat = "webp" | "avif" | "png" | "jpeg";
type Stage = "idle" | "ready" | "converting" | "done" | "error";

type ImageMeta = {
  width: number;
  height: number;
};

type ResultState = {
  url: string;
  fileName: string;
  size: number;
  inputSize: number | null;
  inputWidth: number | null;
  inputHeight: number | null;
  outputWidth: number | null;
  outputHeight: number | null;
};

const API_URL =
  process.env.NEXT_PUBLIC_CONVERT_API_URL ?? "http://localhost:4000";

const supportedInputExtensions = [
  "png",
  "jpg",
  "jpeg",
  "webp",
  "avif",
  "heic",
  "heif",
  "tiff",
  "tif",
  "bmp",
  "ico",
] as const;

const inputAccept = [
  "image/*",
  ...supportedInputExtensions.map((extension) => `.${extension}`),
].join(",");

const formatOptions: Array<{
  value: OutputFormat;
  label: string;
  note: string;
  detail: string;
  tone: string;
}> = [
  {
    value: "webp",
    label: "WebP",
    note: "웹 권장",
    detail: "품질과 용량 균형",
    tone: "bg-accent/12 text-accent",
  },
  {
    value: "avif",
    label: "AVIF",
    note: "초압축",
    detail: "가장 작은 결과",
    tone: "bg-success/14 text-success",
  },
  {
    value: "png",
    label: "PNG",
    note: "무손실",
    detail: "투명도 보존",
    tone: "bg-warning/18 text-warning-foreground",
  },
  {
    value: "jpeg",
    label: "JPEG",
    note: "사진 호환",
    detail: "배경색 합성",
    tone: "bg-rose/14 text-rose",
  },
];

const widthPresets = ["1280", "1600", "1920"] as const;

const colorPresets = [
  { label: "흰색", value: "#FFFFFF" },
  { label: "검정", value: "#000000" },
  { label: "회색", value: "#F3F0EA" },
] as const;

export function ConverterWorkbench() {
  const inputRef = React.useRef<HTMLInputElement>(null);
  const abortRef = React.useRef<AbortController | null>(null);
  const requestIdRef = React.useRef(0);
  const [file, setFile] = React.useState<File | null>(null);
  const [meta, setMeta] = React.useState<ImageMeta | null>(null);
  const [previewUrl, setPreviewUrl] = React.useState<string | null>(null);
  const [format, setFormat] = React.useState<OutputFormat>("webp");
  const [quality, setQuality] = React.useState(90);
  const [maxWidth, setMaxWidth] = React.useState("");
  const [jpegBackground, setJpegBackground] = React.useState("#FFFFFF");
  const [stage, setStage] = React.useState<Stage>("idle");
  const [progress, setProgress] = React.useState(0);
  const [result, setResult] = React.useState<ResultState | null>(null);
  const [error, setError] = React.useState<string | null>(null);
  const [isDragging, setIsDragging] = React.useState(false);
  const isBusy = stage === "converting";
  const selectedFormat = formatOptions.find((option) => option.value === format);
  const reduction = result
    ? reductionRate(result.inputSize ?? file?.size ?? 0, result.size)
    : null;

  React.useEffect(() => {
    return () => {
      if (previewUrl) {
        URL.revokeObjectURL(previewUrl);
      }
    };
  }, [previewUrl]);

  React.useEffect(() => {
    return () => {
      if (result?.url) {
        URL.revokeObjectURL(result.url);
      }
    };
  }, [result?.url]);

  React.useEffect(() => {
    return () => {
      requestIdRef.current += 1;
      abortRef.current?.abort();
    };
  }, []);

  async function selectFile(nextFile: File | null) {
    if (!nextFile || isBusy) {
      return;
    }
    if (!isSupportedInputFile(nextFile)) {
      setError("지원하는 이미지 파일만 선택할 수 있습니다.");
      setStage("error");
      return;
    }

    const selectionId = requestIdRef.current + 1;
    requestIdRef.current = selectionId;
    const objectUrl = URL.createObjectURL(nextFile);
    let dimensions: ImageMeta | null = null;
    let nextPreviewUrl: string | null = objectUrl;

    try {
      dimensions = await readImageMeta(objectUrl);
    } catch {
      URL.revokeObjectURL(objectUrl);
      nextPreviewUrl = null;
    }

    if (selectionId !== requestIdRef.current) {
      if (nextPreviewUrl) {
        URL.revokeObjectURL(nextPreviewUrl);
      }
      return;
    }

    setFile(nextFile);
    setMeta(dimensions);
    setPreviewUrl(nextPreviewUrl);
    setResult(null);
    setError(null);
    setProgress(0);
    setStage("ready");
  }

  const convert = async () => {
    if (!file) {
      setError("변환할 이미지를 먼저 선택하세요.");
      setStage("error");
      return;
    }

    const widthValue = maxWidth.trim();
    if (widthValue && !isPositiveInteger(widthValue)) {
      setError("최대 가로 크기는 1px 이상이어야 합니다.");
      setStage("error");
      return;
    }
    if (format === "jpeg" && !isHexColor(jpegBackground)) {
      setError("JPEG 배경색은 #RRGGBB 형식이어야 합니다.");
      setStage("error");
      return;
    }

    const selectedFile = file;
    const outputFormat = format;
    const requestId = requestIdRef.current + 1;
    const controller = new AbortController();
    requestIdRef.current = requestId;
    abortRef.current?.abort();
    abortRef.current = controller;

    setStage("converting");
    setError(null);
    setProgress(18);

    const formData = new FormData();
    formData.append("file", selectedFile);
    formData.append("format", outputFormat);
    formData.append("quality", String(quality));
    if (widthValue) {
      formData.append("max_width", widthValue);
    }
    if (outputFormat === "jpeg") {
      formData.append("jpeg_background", jpegBackground);
    }

    try {
      setProgress(46);
      const response = await fetch(`${API_URL}/v1/convert`, {
        method: "POST",
        body: formData,
        signal: controller.signal,
      });
      if (requestId !== requestIdRef.current) {
        return;
      }
      setProgress(78);

      if (!response.ok) {
        throw new Error(await readErrorMessage(response));
      }

      const blob = await response.blob();
      const url = URL.createObjectURL(blob);
      if (requestId !== requestIdRef.current) {
        URL.revokeObjectURL(url);
        return;
      }

      setResult({
        url,
        fileName:
          fileNameFromResponse(response) ??
          outputFileName(selectedFile.name, outputFormat),
        size: blob.size,
        inputSize: numberHeader(response, "x-input-size"),
        inputWidth: numberHeader(response, "x-input-width"),
        inputHeight: numberHeader(response, "x-input-height"),
        outputWidth: numberHeader(response, "x-output-width"),
        outputHeight: numberHeader(response, "x-output-height"),
      });
      setProgress(100);
      setStage("done");
    } catch (cause) {
      if (requestId !== requestIdRef.current) {
        return;
      }
      setError(
        cause instanceof Error
          ? cause.message
          : "이미지 변환에 실패했습니다."
      );
      setStage("error");
      setProgress(0);
    } finally {
      if (abortRef.current === controller) {
        abortRef.current = null;
      }
    }
  };

  const reset = () => {
    requestIdRef.current += 1;
    abortRef.current?.abort();
    abortRef.current = null;
    setFile(null);
    setMeta(null);
    setPreviewUrl(null);
    setResult(null);
    setError(null);
    setStage("idle");
    setProgress(0);
    if (inputRef.current) {
      inputRef.current.value = "";
    }
  };

  const qualityDisabled = format === "png";
  const projectedWidth = projectedOutputWidth(meta, maxWidth);
  const canConvert = Boolean(file) && !isBusy;

  return (
    <main className="min-h-screen overflow-hidden px-3 py-3 text-foreground sm:px-4 lg:px-5">
      <input
        ref={inputRef}
        type="file"
        accept={inputAccept}
        disabled={isBusy}
        className="hidden"
        onChange={(event) => {
          void selectFile(event.target.files?.item(0) ?? null);
        }}
      />

      <div className="mx-auto grid w-full max-w-[1560px] gap-4 xl:grid-cols-[420px_minmax(0,1fr)]">
        <ControlDeck
          canConvert={canConvert}
          convert={convert}
          error={error}
          file={file}
          format={format}
          inputRef={inputRef}
          isBusy={isBusy}
          isDragging={isDragging}
          jpegBackground={jpegBackground}
          maxWidth={maxWidth}
          meta={meta}
          progress={progress}
          projectedWidth={projectedWidth}
          quality={quality}
          qualityDisabled={qualityDisabled}
          reduction={reduction}
          reset={reset}
          result={result}
          selectFile={selectFile}
          selectedFormat={selectedFormat}
          setFormat={setFormat}
          setIsDragging={setIsDragging}
          setJpegBackground={setJpegBackground}
          setMaxWidth={setMaxWidth}
          setQuality={setQuality}
          stage={stage}
        />

        <PreviewWorkspace
          file={file}
          format={format}
          inputRef={inputRef}
          isBusy={isBusy}
          isDragging={isDragging}
          meta={meta}
          previewUrl={previewUrl}
          progress={progress}
          reduction={reduction}
          reset={reset}
          result={result}
          selectFile={selectFile}
          setIsDragging={setIsDragging}
          stage={stage}
        />
      </div>
    </main>
  );
}

function ControlDeck({
  canConvert,
  convert,
  error,
  file,
  format,
  inputRef,
  isBusy,
  isDragging,
  jpegBackground,
  maxWidth,
  meta,
  progress,
  projectedWidth,
  quality,
  qualityDisabled,
  reduction,
  reset,
  result,
  selectFile,
  selectedFormat,
  setFormat,
  setIsDragging,
  setJpegBackground,
  setMaxWidth,
  setQuality,
  stage,
}: {
  canConvert: boolean;
  convert: () => Promise<void>;
  error: string | null;
  file: File | null;
  format: OutputFormat;
  inputRef: React.RefObject<HTMLInputElement | null>;
  isBusy: boolean;
  isDragging: boolean;
  jpegBackground: string;
  maxWidth: string;
  meta: ImageMeta | null;
  progress: number;
  projectedWidth: string;
  quality: number;
  qualityDisabled: boolean;
  reduction: number | null;
  reset: () => void;
  result: ResultState | null;
  selectFile: (file: File | null) => Promise<void>;
  selectedFormat:
    | {
        label: string;
        note: string;
        tone: string;
      }
    | undefined;
  setFormat: (format: OutputFormat) => void;
  setIsDragging: (value: boolean) => void;
  setJpegBackground: (value: string) => void;
  setMaxWidth: (value: string) => void;
  setQuality: (value: number) => void;
  stage: Stage;
}) {
  return (
    <Surface
      className="control-deck flex min-h-[640px] flex-col overflow-hidden rounded-lg border border-border bg-overlay/92 shadow-surface xl:sticky xl:top-3 xl:h-[calc(100vh-1.5rem)]"
      variant="default"
    >
      <div className="flex items-start justify-between gap-4 border-b border-separator px-5 py-5">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <div className="grid size-9 shrink-0 place-items-center rounded-lg border border-accent/20 bg-accent/10 text-accent">
              <FileImageIcon className="size-5" />
            </div>
            <div className="min-w-0">
              <h1 className="truncate text-xl font-semibold tracking-normal">
                이미지 컨버터
              </h1>
              <p className="mt-0.5 text-xs text-muted">PNG, JPEG, WebP, AVIF</p>
            </div>
          </div>
        </div>
        <Chip color={statusColor(stage)} size="sm" variant="soft">
          {statusLabel(stage)}
        </Chip>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-4 py-4">
        <UploadDock
          file={file}
          inputRef={inputRef}
          isBusy={isBusy}
          isDragging={isDragging}
          meta={meta}
          selectFile={selectFile}
          setIsDragging={setIsDragging}
        />

        {error ? (
          <Alert status="danger" className="mt-3 animate-panel-in">
            <Alert.Indicator>
              <AlertCircleIcon />
            </Alert.Indicator>
            <Alert.Content>
              <Alert.Title>변환 실패</Alert.Title>
              <Alert.Description>{error}</Alert.Description>
            </Alert.Content>
          </Alert>
        ) : null}

        {isBusy ? (
          <ProgressPanel progress={progress} />
        ) : null}

        <div className="mt-5 flex items-center justify-between gap-3">
          <SectionTitle
            icon={<SlidersHorizontalIcon className="size-4" />}
            label="출력"
          />
          <span
            className={cn(
              "rounded-md px-2 py-1 text-[11px] font-semibold",
              selectedFormat?.tone
            )}
          >
            {selectedFormat?.label ?? format.toUpperCase()}
          </span>
        </div>

        <FormatPicker format={format} setFormat={setFormat} />

        <Separator className="my-4" />

        <SettingBlock
          icon={<GaugeIcon className="size-4" />}
          label="품질"
          value={qualityDisabled ? "무손실" : `${quality}%`}
        >
          <div className="flex items-center gap-3">
            <Slider
              aria-label="출력 품질"
              className="min-w-0 flex-1"
              minValue={1}
              maxValue={100}
              step={1}
              value={quality}
              isDisabled={qualityDisabled}
              onChange={(value) => {
                if (typeof value === "number") {
                  setQuality(value);
                }
              }}
            >
              <Slider.Track>
                <Slider.Fill />
                <Slider.Thumb />
              </Slider.Track>
            </Slider>
            <Input
              aria-label="품질 직접 입력"
              inputMode="numeric"
              min={1}
              max={100}
              type="number"
              value={String(quality)}
              disabled={qualityDisabled}
              onChange={(event) => {
                setQuality(clampQuality(event.target.value));
              }}
              className="w-20 shrink-0 text-right font-mono tabular-nums"
            />
          </div>
        </SettingBlock>

        <SettingBlock
          icon={<RulerIcon className="size-4" />}
          label="가로 크기"
          value={projectedWidth}
        >
          <div className="flex flex-col gap-3">
            <Input
              aria-label="최대 가로 크기"
              inputMode="numeric"
              pattern="[0-9]*"
              placeholder="최대 가로(px)"
              value={maxWidth}
              onChange={(event) => setMaxWidth(event.target.value)}
            />
            <div className="grid grid-cols-3 gap-2">
              {widthPresets.map((preset) => (
                <Button
                  key={preset}
                  size="sm"
                  type="button"
                  variant={maxWidth === preset ? "secondary" : "tertiary"}
                  onPress={() => setMaxWidth(preset)}
                >
                  {preset}
                </Button>
              ))}
            </div>
          </div>
        </SettingBlock>

        {format === "jpeg" ? (
          <SettingBlock
            icon={<PaletteIcon className="size-4" />}
            label="배경색"
            value={safeColor(jpegBackground).toUpperCase()}
          >
            <div className="flex flex-col gap-3">
              <div className="flex gap-2">
                <Input
                  aria-label="JPEG 배경색"
                  value={jpegBackground}
                  onChange={(event) =>
                    setJpegBackground(event.target.value.toUpperCase())
                  }
                />
                <label className="relative grid size-10 shrink-0 cursor-pointer place-items-center overflow-hidden rounded-lg border border-field-border bg-field shadow-field">
                  <span
                    className="size-5 rounded-md border border-border"
                    style={{ backgroundColor: safeColor(jpegBackground) }}
                  />
                  <input
                    aria-label="JPEG 배경색 선택"
                    type="color"
                    value={safeColor(jpegBackground)}
                    onChange={(event) =>
                      setJpegBackground(event.target.value.toUpperCase())
                    }
                    className="absolute inset-0 cursor-pointer opacity-0"
                  />
                </label>
              </div>
              <div className="grid grid-cols-3 gap-2">
                {colorPresets.map((preset) => (
                  <button
                    key={preset.value}
                    type="button"
                    aria-label={`${preset.label} 배경색`}
                    aria-pressed={
                      safeColor(jpegBackground).toUpperCase() === preset.value
                    }
                    className="flex h-9 items-center justify-center rounded-lg border border-border bg-surface-tertiary transition hover:border-accent aria-pressed:border-accent"
                    onClick={() => setJpegBackground(preset.value)}
                  >
                    <span
                      className="size-4 rounded-sm border border-border"
                      style={{ backgroundColor: preset.value }}
                    />
                  </button>
                ))}
              </div>
            </div>
          </SettingBlock>
        ) : null}
      </div>

      <div className="border-t border-separator bg-surface-secondary/70 p-4">
        <Button
          fullWidth
          size="lg"
          type="button"
          isDisabled={!canConvert}
          isPending={isBusy}
          onPress={convert}
        >
          {({ isPending }) => (
            <>
              {isPending ? (
                <Spinner color="current" size="sm" />
              ) : (
                <SparklesIcon data-icon="inline-start" />
              )}
              {isPending ? "변환 중" : "변환하기"}
            </>
          )}
        </Button>

        <div className="mt-2 grid grid-cols-[1fr_auto] gap-2">
          {result ? (
            <DownloadButton fullWidth result={result} variant="secondary" />
          ) : (
            <Button fullWidth isDisabled type="button" variant="secondary">
              <ArrowDownToLineIcon data-icon="inline-start" />
              다운로드
            </Button>
          )}
          <Tooltip delay={120}>
            <Button
              isIconOnly
              aria-label="초기화"
              type="button"
              variant="tertiary"
              isDisabled={isBusy}
              onPress={reset}
            >
              <RotateCcwIcon />
            </Button>
            <Tooltip.Content showArrow>
              <Tooltip.Arrow />
              <p>초기화</p>
            </Tooltip.Content>
          </Tooltip>
        </div>

        {result ? (
          <p className="mt-2 truncate text-xs text-muted">
            {result.fileName}
            {reduction === null ? "" : ` · ${reduction.toFixed(1)}%`}
          </p>
        ) : null}
      </div>
    </Surface>
  );
}

function UploadDock({
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

function PreviewWorkspace({
  file,
  format,
  inputRef,
  isBusy,
  isDragging,
  meta,
  previewUrl,
  progress,
  reduction,
  reset,
  result,
  selectFile,
  setIsDragging,
  stage,
}: {
  file: File | null;
  format: OutputFormat;
  inputRef: React.RefObject<HTMLInputElement | null>;
  isBusy: boolean;
  isDragging: boolean;
  meta: ImageMeta | null;
  previewUrl: string | null;
  progress: number;
  reduction: number | null;
  reset: () => void;
  result: ResultState | null;
  selectFile: (file: File | null) => Promise<void>;
  setIsDragging: (value: boolean) => void;
  stage: Stage;
}) {
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
  format: OutputFormat;
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

function ProgressPanel({ progress }: { progress: number }) {
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

function FormatPicker({
  format,
  setFormat,
}: {
  format: OutputFormat;
  setFormat: (format: OutputFormat) => void;
}) {
  return (
    <RadioGroup
      aria-label="출력 포맷"
      value={format}
      onChange={(value) => setFormat(value as OutputFormat)}
      className="mt-3 grid grid-cols-2 gap-2"
      orientation="horizontal"
      variant="secondary"
    >
      {formatOptions.map((option) => (
        <Radio
          key={option.value}
          value={option.value}
          className="group relative min-h-24 cursor-pointer rounded-lg border border-border bg-surface-secondary p-3 transition data-[selected=true]:border-accent data-[selected=true]:bg-accent/12"
        >
          <Radio.Control className="absolute right-3 top-3">
            <Radio.Indicator />
          </Radio.Control>
          <Radio.Content className="gap-2 pr-5">
            <span
              className={cn(
                "inline-flex w-fit rounded-md px-2 py-1 text-[11px] font-semibold",
                option.tone
              )}
            >
              {option.label}
            </span>
            <Label className="text-sm font-semibold">{option.note}</Label>
            <span className="text-xs leading-4 text-muted">{option.detail}</span>
          </Radio.Content>
        </Radio>
      ))}
    </RadioGroup>
  );
}

function SectionTitle({
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

function SettingBlock({
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

function ResultCard({
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

function DownloadButton({
  fullWidth,
  result,
  size = "md",
  variant = "primary",
}: {
  fullWidth?: boolean;
  result: ResultState;
  size?: "sm" | "md" | "lg";
  variant?: "primary" | "secondary" | "tertiary" | "outline" | "ghost" | "danger";
}) {
  return (
    <Button
      fullWidth={fullWidth}
      size={size}
      variant={variant}
      render={(props) => {
        const anchorProps = { ...props };
        delete (anchorProps as { ref?: unknown }).ref;

        return (
          <a
            {...(anchorProps as React.AnchorHTMLAttributes<HTMLAnchorElement>)}
            href={result.url}
            download={result.fileName}
          />
        );
      }}
    >
      <ArrowDownToLineIcon data-icon="inline-start" />
      다운로드
    </Button>
  );
}

function FileStat({
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

function statusLabel(stage: Stage) {
  switch (stage) {
    case "ready":
      return "준비됨";
    case "converting":
      return "변환 중";
    case "done":
      return "완료";
    case "error":
      return "오류";
    default:
      return "대기";
  }
}

function stageDescription(stage: Stage) {
  switch (stage) {
    case "ready":
      return "파일 준비";
    case "converting":
      return "처리 중";
    case "done":
      return "다운로드 가능";
    case "error":
      return "확인 필요";
    default:
      return "대기";
  }
}

function statusColor(
  stage: Stage
): "accent" | "danger" | "default" | "success" | "warning" {
  switch (stage) {
    case "ready":
      return "warning";
    case "converting":
      return "accent";
    case "done":
      return "success";
    case "error":
      return "danger";
    default:
      return "default";
  }
}

function readImageMeta(url: string): Promise<ImageMeta> {
  return new Promise((resolve, reject) => {
    const image = new window.Image();
    image.onload = () => {
      resolve({
        width: image.naturalWidth,
        height: image.naturalHeight,
      });
    };
    image.onerror = reject;
    image.src = url;
  });
}

function isSupportedInputFile(file: File) {
  const extension = fileExtension(file.name);
  return extension !== null && isSupportedInputExtension(extension);
}

function isSupportedInputExtension(extension: string) {
  return supportedInputExtensions.includes(
    extension.toLowerCase() as (typeof supportedInputExtensions)[number]
  );
}

function fileExtension(fileName: string) {
  const match = /\.([^.]+)$/.exec(fileName);
  return match?.[1]?.toLowerCase() ?? null;
}

function inputExtensionLabel(fileName: string) {
  const extension = fileExtension(fileName);
  return extension ? `.${extension}` : "확장자 없음";
}

async function readErrorMessage(response: Response) {
  try {
    const data = (await response.json()) as {
      error?: { message?: string };
    };
    return data.error?.message ?? `요청 실패: ${response.status}`;
  } catch {
    return `요청 실패: ${response.status}`;
  }
}

function fileNameFromResponse(response: Response) {
  const disposition = response.headers.get("content-disposition");
  const match = disposition?.match(/filename="([^"]+)"/);
  return match?.[1] ?? null;
}

function outputFileName(fileName: string, format: OutputFormat) {
  const stem = fileName.replace(/\.[^/.]+$/, "") || "converted";
  const extension = format === "jpeg" ? "jpg" : format;
  return `${stem}.${extension}`;
}

function numberHeader(response: Response, header: string) {
  const value = response.headers.get(header);
  if (!value) {
    return null;
  }
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function reductionRate(inputSize: number, outputSize: number) {
  if (!inputSize) {
    return 0;
  }
  return ((inputSize - outputSize) / inputSize) * 100;
}

function dimensionLabel(width: number | null, height: number | null) {
  if (!width || !height) {
    return "-";
  }
  return `${width} x ${height}`;
}

function projectedOutputWidth(meta: ImageMeta | null, maxWidth: string) {
  const trimmed = maxWidth.trim();
  if (!meta || !trimmed || !isPositiveInteger(trimmed)) {
    return "원본 유지";
  }
  const parsed = Number(trimmed);
  return parsed < meta.width ? `${parsed}px` : "원본 유지";
}

function formatBytes(bytes: number) {
  if (bytes === 0) {
    return "0 B";
  }
  const units = ["B", "KB", "MB", "GB"];
  const exponent = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1
  );
  return `${(bytes / 1024 ** exponent).toFixed(exponent === 0 ? 0 : 2)} ${
    units[exponent]
  }`;
}

function safeColor(value: string) {
  return isHexColor(value) ? value : "#ffffff";
}

function isHexColor(value: string) {
  return /^#[0-9a-f]{6}$/i.test(value);
}

function isPositiveInteger(value: string) {
  return /^[1-9]\d*$/.test(value);
}

function clampQuality(value: string) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    return 1;
  }
  return Math.min(100, Math.max(1, Math.round(parsed)));
}
