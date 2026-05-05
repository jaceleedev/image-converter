import { supportedInputExtensions } from "../constants";
import type { ImageMeta, OutputFormat } from "../types";

export function isSupportedInputFile(file: File) {
  const extension = fileExtension(file.name);
  return extension !== null && isSupportedInputExtension(extension);
}

export function isSupportedInputExtension(extension: string) {
  return supportedInputExtensions.includes(
    extension.toLowerCase() as (typeof supportedInputExtensions)[number]
  );
}

export function fileExtension(fileName: string) {
  const match = /\.([^.]+)$/.exec(fileName);
  return match?.[1]?.toLowerCase() ?? null;
}

export function inputExtensionLabel(fileName: string) {
  const extension = fileExtension(fileName);
  return extension ? `.${extension}` : "확장자 없음";
}

export function outputFileName(fileName: string, format: OutputFormat) {
  const stem = fileName.replace(/\.[^/.]+$/, "") || "converted";
  const extension = format === "jpeg" ? "jpg" : format;
  return `${stem}.${extension}`;
}

export function reductionRate(inputSize: number, outputSize: number) {
  if (!inputSize) {
    return 0;
  }
  return ((inputSize - outputSize) / inputSize) * 100;
}

export function dimensionLabel(width: number | null, height: number | null) {
  if (!width || !height) {
    return "-";
  }
  return `${width} x ${height}`;
}

export function projectedOutputWidth(meta: ImageMeta | null, maxWidth: string) {
  const trimmed = maxWidth.trim();
  if (!meta || !trimmed || !isPositiveInteger(trimmed)) {
    return "원본 유지";
  }
  const parsed = Number(trimmed);
  return parsed < meta.width ? `${parsed}px` : "원본 유지";
}

export function formatBytes(bytes: number) {
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

export function safeColor(value: string) {
  return isHexColor(value) ? value : "#ffffff";
}

export function isHexColor(value: string) {
  return /^#[0-9a-f]{6}$/i.test(value);
}

export function isPositiveInteger(value: string) {
  return /^[1-9]\d*$/.test(value);
}

export function clampQuality(value: string) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    return 1;
  }
  return Math.min(100, Math.max(1, Math.round(parsed)));
}
