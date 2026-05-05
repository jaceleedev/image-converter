"use client";

import * as React from "react";

import { formatOptions } from "../constants";
import { requestImageConversion } from "../lib/api";
import {
  isHexColor,
  isPositiveInteger,
  isSupportedInputFile,
  projectedOutputWidth,
  reductionRate,
} from "../lib/format";
import { readImageMeta } from "../lib/image";
import type { FormatOption, ImageMeta, OutputFormat, ResultState, Stage } from "../types";

export type ConverterState = {
  file: File | null;
  meta: ImageMeta | null;
  previewUrl: string | null;
  format: OutputFormat;
  quality: number;
  maxWidth: string;
  jpegBackground: string;
  stage: Stage;
  progress: number;
  result: ResultState | null;
  error: string | null;
  isDragging: boolean;
};

export type ConverterDerived = {
  canConvert: boolean;
  isBusy: boolean;
  projectedWidth: string;
  qualityDisabled: boolean;
  reduction: number | null;
  selectedFormat: FormatOption | undefined;
};

export type ConverterActions = {
  convert: () => Promise<void>;
  reset: () => void;
  selectFile: (file: File | null) => Promise<void>;
  setFormat: (format: OutputFormat) => void;
  setIsDragging: (value: boolean) => void;
  setJpegBackground: (value: string) => void;
  setMaxWidth: (value: string) => void;
  setQuality: (value: number) => void;
};

export function useConverter() {
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
  const qualityDisabled = format === "png";
  const projectedWidth = projectedOutputWidth(meta, maxWidth);
  const canConvert = Boolean(file) && !isBusy;

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

    const requestId = requestIdRef.current + 1;
    const controller = new AbortController();
    requestIdRef.current = requestId;
    abortRef.current?.abort();
    abortRef.current = controller;

    setStage("converting");
    setError(null);
    setProgress(18);

    try {
      setProgress(46);
      const nextResult = await requestImageConversion({
        file,
        format,
        quality,
        maxWidth,
        jpegBackground,
        onProgress: setProgress,
        signal: controller.signal,
      });
      if (requestId !== requestIdRef.current) {
        URL.revokeObjectURL(nextResult.url);
        return;
      }

      setProgress(100);
      setResult(nextResult);
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

  return {
    inputRef,
    state: {
      file,
      meta,
      previewUrl,
      format,
      quality,
      maxWidth,
      jpegBackground,
      stage,
      progress,
      result,
      error,
      isDragging,
    },
    derived: {
      canConvert,
      isBusy,
      projectedWidth,
      qualityDisabled,
      reduction,
      selectedFormat,
    },
    actions: {
      convert,
      reset,
      selectFile,
      setFormat,
      setIsDragging,
      setJpegBackground,
      setMaxWidth,
      setQuality,
    },
  } satisfies {
    inputRef: React.RefObject<HTMLInputElement | null>;
    state: ConverterState;
    derived: ConverterDerived;
    actions: ConverterActions;
  };
}
