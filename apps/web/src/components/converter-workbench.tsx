"use client";

import Image from "next/image";
import {
  AlertCircleIcon,
  ArrowDownToLineIcon,
  CheckCircle2Icon,
  FileImageIcon,
  ImageUpIcon,
  LoaderCircleIcon,
  RotateCcwIcon,
  SparklesIcon,
} from "lucide-react";
import * as React from "react";

import {
  Alert,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import { Slider } from "@/components/ui/slider";
import {
  ToggleGroup,
  ToggleGroupItem,
} from "@/components/ui/toggle-group";
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

const formatOptions: Array<{
  value: OutputFormat;
  label: string;
  note: string;
}> = [
  { value: "webp", label: "WebP", note: "웹" },
  { value: "avif", label: "AVIF", note: "초압축" },
  { value: "png", label: "PNG", note: "무손실" },
  { value: "jpeg", label: "JPEG", note: "사진" },
];

export function ConverterWorkbench() {
  const inputRef = React.useRef<HTMLInputElement>(null);
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

  React.useEffect(() => {
    return () => {
      if (previewUrl) {
        URL.revokeObjectURL(previewUrl);
      }
      if (result?.url) {
        URL.revokeObjectURL(result.url);
      }
    };
  }, [previewUrl, result?.url]);

  async function selectFile(nextFile: File | null) {
    if (!nextFile) {
      return;
    }
    if (!nextFile.type.startsWith("image/")) {
      setError("이미지 파일만 선택할 수 있습니다.");
      setStage("error");
      return;
    }

    const objectUrl = URL.createObjectURL(nextFile);
    try {
      const dimensions = await readImageMeta(objectUrl);
      if (previewUrl) {
        URL.revokeObjectURL(previewUrl);
      }
      if (result?.url) {
        URL.revokeObjectURL(result.url);
      }
      setFile(nextFile);
      setMeta(dimensions);
      setPreviewUrl(objectUrl);
      setResult(null);
      setError(null);
      setProgress(0);
      setStage("ready");
    } catch {
      URL.revokeObjectURL(objectUrl);
      setError("이미지 정보를 읽을 수 없습니다.");
      setStage("error");
    }
  }

  const convert = async () => {
    if (!file) {
      setError("변환할 이미지를 먼저 선택하세요.");
      setStage("error");
      return;
    }

    const widthValue = maxWidth.trim();
    if (widthValue && Number(widthValue) < 1) {
      setError("최대 가로 크기는 1px 이상이어야 합니다.");
      setStage("error");
      return;
    }

    setStage("converting");
    setError(null);
    setProgress(18);

    const formData = new FormData();
    formData.append("file", file);
    formData.append("format", format);
    formData.append("quality", String(quality));
    if (widthValue) {
      formData.append("max_width", widthValue);
    }
    if (format === "jpeg") {
      formData.append("jpeg_background", jpegBackground);
    }

    try {
      setProgress(46);
      const response = await fetch(`${API_URL}/v1/convert`, {
        method: "POST",
        body: formData,
      });
      setProgress(78);

      if (!response.ok) {
        throw new Error(await readErrorMessage(response));
      }

      const blob = await response.blob();
      const url = URL.createObjectURL(blob);
      if (result?.url) {
        URL.revokeObjectURL(result.url);
      }

      setResult({
        url,
        fileName: fileNameFromResponse(response) ?? outputFileName(file.name, format),
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
      setError(
        cause instanceof Error
          ? cause.message
          : "이미지 변환에 실패했습니다."
      );
      setStage("error");
      setProgress(0);
    }
  };

  const reset = () => {
    if (previewUrl) {
      URL.revokeObjectURL(previewUrl);
    }
    if (result?.url) {
      URL.revokeObjectURL(result.url);
    }
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

  const isBusy = stage === "converting";
  const qualityDisabled = format === "png";
  const reduction = result
    ? reductionRate(result.inputSize ?? file?.size ?? 0, result.size)
    : null;

  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-6 px-4 py-4 sm:px-6 lg:px-8">
        <header className="grid gap-4 border-b border-border py-5 lg:grid-cols-[1fr_auto] lg:items-end">
          <div className="flex flex-col gap-3">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant="secondary">LOCAL API</Badge>
              <Badge variant={stage === "done" ? "default" : "outline"}>
                {statusLabel(stage)}
              </Badge>
            </div>
            <div className="flex flex-col gap-2">
              <h1 className="max-w-3xl text-3xl font-semibold leading-tight sm:text-5xl">
                Image Converter
              </h1>
              <p className="max-w-2xl text-sm leading-6 text-muted-foreground sm:text-base">
                PNG, JPEG, WebP, AVIF를 로컬 Rust API로 변환합니다.
              </p>
            </div>
          </div>
          <div className="grid grid-cols-3 border border-border bg-card text-sm">
            <Metric label="API" value=":4000" />
            <Metric label="Mode" value="Single" />
            <Metric label="Target" value={format.toUpperCase()} />
          </div>
        </header>

        <section className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_360px]">
          <div className="flex min-w-0 flex-col gap-4">
            <button
              type="button"
              onClick={() => inputRef.current?.click()}
              onDragEnter={(event) => {
                event.preventDefault();
                setIsDragging(true);
              }}
              onDragOver={(event) => event.preventDefault()}
              onDragLeave={(event) => {
                event.preventDefault();
                setIsDragging(false);
              }}
              onDrop={(event) => {
                event.preventDefault();
                setIsDragging(false);
                void selectFile(event.dataTransfer.files.item(0));
              }}
              className={cn(
                "relative min-h-[360px] overflow-hidden border border-dashed border-border bg-card text-left transition-colors",
                "focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-hidden",
                isDragging && "border-primary bg-accent"
              )}
            >
              <input
                ref={inputRef}
                type="file"
                accept="image/*"
                className="sr-only"
                onChange={(event) => {
                  void selectFile(event.target.files?.item(0) ?? null);
                }}
              />
              <div className="absolute inset-0 converter-grid opacity-70" />
              {previewUrl && meta ? (
                <div className="relative grid min-h-[360px] grid-rows-[1fr_auto]">
                  <div className="flex items-center justify-center p-4">
                    <div className="relative aspect-[4/3] w-full max-w-3xl overflow-hidden border border-border bg-background">
                      <Image
                        src={previewUrl}
                        alt={file?.name ?? "업로드 이미지"}
                        fill
                        unoptimized
                        className="object-contain"
                      />
                    </div>
                  </div>
                  <div className="grid gap-3 border-t border-border bg-background/90 p-4 sm:grid-cols-3">
                    <FileStat label="파일" value={file?.name ?? "-"} />
                    <FileStat label="크기" value={`${meta.width} x ${meta.height}`} />
                    <FileStat label="용량" value={formatBytes(file?.size ?? 0)} />
                  </div>
                </div>
              ) : (
                <div className="relative flex min-h-[360px] flex-col justify-between p-5">
                  <div className="flex items-center justify-between gap-3">
                    <FileImageIcon />
                    <span className="font-mono text-xs text-muted-foreground">
                      DROP IMAGE
                    </span>
                  </div>
                  <div className="flex max-w-xl flex-col gap-4">
                    <ImageUpIcon className="text-primary" />
                    <div className="flex flex-col gap-2">
                      <h2 className="text-2xl font-semibold sm:text-4xl">
                        이미지를 올려주세요
                      </h2>
                      <p className="text-sm leading-6 text-muted-foreground">
                        파일은 브라우저에서 Rust API로 직접 전송됩니다.
                      </p>
                    </div>
                  </div>
                </div>
              )}
            </button>

            {error ? (
              <Alert variant="destructive">
                <AlertCircleIcon />
                <AlertTitle>변환 실패</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            ) : null}

            {isBusy ? <Progress value={progress} /> : null}

            {result ? (
              <section className="grid gap-4 border border-border bg-card p-4 md:grid-cols-[1fr_auto] md:items-center">
                <div className="grid gap-3 sm:grid-cols-3">
                  <FileStat
                    label="결과 용량"
                    value={formatBytes(result.size)}
                    emphasis
                  />
                  <FileStat
                    label="감소율"
                    value={reduction === null ? "-" : `${reduction.toFixed(1)}%`}
                    emphasis
                  />
                  <FileStat
                    label="출력 크기"
                    value={dimensionLabel(result.outputWidth, result.outputHeight)}
                    emphasis
                  />
                </div>
                <Button render={<a href={result.url} download={result.fileName} />}>
                  <ArrowDownToLineIcon data-icon="inline-start" />
                  다운로드
                </Button>
              </section>
            ) : null}
          </div>

          <aside className="flex flex-col gap-4">
            <section className="border border-border bg-card p-4">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <h2 className="font-semibold">출력 포맷</h2>
                  <p className="text-sm text-muted-foreground">
                    변환 대상 형식
                  </p>
                </div>
                <SparklesIcon className="text-primary" />
              </div>
              <ToggleGroup
                value={[format]}
                onValueChange={(value) => {
                  const next = value.at(-1) as OutputFormat | undefined;
                  if (next) {
                    setFormat(next);
                  }
                }}
                className="mt-4 grid w-full grid-cols-2"
                variant="outline"
                spacing={1}
              >
                {formatOptions.map((option) => (
                  <ToggleGroupItem
                    key={option.value}
                    value={option.value}
                    className="h-16 flex-col items-start justify-center gap-1 px-3"
                  >
                    <span>{option.label}</span>
                    <span className="text-xs text-muted-foreground">
                      {option.note}
                    </span>
                  </ToggleGroupItem>
                ))}
              </ToggleGroup>
            </section>

            <section className="border border-border bg-card p-4">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <h2 className="font-semibold">품질</h2>
                <p className="text-sm text-muted-foreground">
                  {qualityDisabled ? "PNG는 무손실" : `${quality}%`}
                </p>
              </div>
              {qualityDisabled ? (
                <span className="font-mono text-2xl tabular-nums">∞</span>
              ) : (
                <Input
                  aria-label="품질 직접 입력"
                  inputMode="numeric"
                  min={1}
                  max={100}
                  type="number"
                  value={quality}
                  onChange={(event) => {
                    setQuality(clampQuality(event.target.value));
                  }}
                  className="w-20 text-right font-mono text-lg tabular-nums"
                />
              )}
            </div>
              <Slider
                className="mt-5"
                min={1}
                max={100}
                step={1}
                value={quality}
                disabled={qualityDisabled}
                onValueChange={(value) => {
                  if (typeof value === "number") {
                    setQuality(value);
                  }
                }}
              />
            </section>

            <section className="border border-border bg-card p-4">
              <h2 className="font-semibold">크기</h2>
              <div className="mt-4 flex flex-col gap-2">
                <label
                  htmlFor="max-width"
                  className="text-sm text-muted-foreground"
                >
                  최대 가로(px)
                </label>
                <Input
                  id="max-width"
                  inputMode="numeric"
                  pattern="[0-9]*"
                  placeholder="예: 1600"
                  value={maxWidth}
                  onChange={(event) => setMaxWidth(event.target.value)}
                />
              </div>
              {format === "jpeg" ? (
                <div className="mt-4 flex flex-col gap-2">
                  <label
                    htmlFor="jpeg-background"
                    className="text-sm text-muted-foreground"
                  >
                    JPEG 배경색
                  </label>
                  <div className="flex gap-2">
                    <Input
                      id="jpeg-background"
                      value={jpegBackground}
                      onChange={(event) =>
                        setJpegBackground(event.target.value)
                      }
                    />
                    <input
                      aria-label="JPEG 배경색 선택"
                      type="color"
                      value={safeColor(jpegBackground)}
                      onChange={(event) =>
                        setJpegBackground(event.target.value.toUpperCase())
                      }
                      className="h-8 w-12 border border-input bg-transparent"
                    />
                  </div>
                </div>
              ) : null}
            </section>

            <section className="border border-border bg-card p-4">
              <div className="flex flex-col gap-3">
                <Button
                  type="button"
                  onClick={convert}
                  disabled={!file || isBusy}
                  className="w-full"
                >
                  {isBusy ? (
                    <LoaderCircleIcon
                      data-icon="inline-start"
                      className="animate-spin"
                    />
                  ) : (
                    <CheckCircle2Icon data-icon="inline-start" />
                  )}
                  변환하기
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={reset}
                  disabled={isBusy}
                  className="w-full"
                >
                  <RotateCcwIcon data-icon="inline-start" />
                  초기화
                </Button>
              </div>
              <Separator className="my-4" />
              <div className="grid grid-cols-2 gap-3 text-sm">
                <FileStat
                  label="입력"
                  value={
                    result
                      ? dimensionLabel(result.inputWidth, result.inputHeight)
                      : meta
                        ? `${meta.width} x ${meta.height}`
                        : "-"
                  }
                />
                <FileStat
                  label="출력"
                  value={
                    result
                      ? dimensionLabel(result.outputWidth, result.outputHeight)
                      : format.toUpperCase()
                  }
                />
              </div>
            </section>
          </aside>
        </section>
      </div>
    </main>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-24 border-r border-border px-3 py-2 last:border-r-0">
      <div className="font-mono text-xs text-muted-foreground">{label}</div>
      <div className="mt-1 font-semibold">{value}</div>
    </div>
  );
}

function FileStat({
  label,
  value,
  emphasis = false,
}: {
  label: string;
  value: string;
  emphasis?: boolean;
}) {
  return (
    <div className="min-w-0">
      <div className="font-mono text-xs text-muted-foreground">{label}</div>
      <div
        className={cn(
          "mt-1 truncate text-sm",
          emphasis && "text-lg font-semibold"
        )}
        title={value}
      >
        {value}
      </div>
    </div>
  );
}

function statusLabel(stage: Stage) {
  switch (stage) {
    case "ready":
      return "READY";
    case "converting":
      return "RUNNING";
    case "done":
      return "DONE";
    case "error":
      return "ERROR";
    default:
      return "WAITING";
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
  return /^#[0-9a-f]{6}$/i.test(value) ? value : "#ffffff";
}

function clampQuality(value: string) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    return 1;
  }
  return Math.min(100, Math.max(1, Math.round(parsed)));
}
