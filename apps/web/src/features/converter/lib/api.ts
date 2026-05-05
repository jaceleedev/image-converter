import { API_URL } from "../constants";
import type { OutputFormat, ResultState } from "../types";
import { outputFileName } from "./format";

type ConvertImageRequest = {
  file: File;
  format: OutputFormat;
  quality: number;
  maxWidth: string;
  jpegBackground: string;
  onProgress?: (progress: number) => void;
  signal: AbortSignal;
};

export async function requestImageConversion({
  file,
  format,
  jpegBackground,
  maxWidth,
  onProgress,
  quality,
  signal,
}: ConvertImageRequest): Promise<ResultState> {
  const formData = new FormData();
  formData.append("file", file);
  formData.append("format", format);
  formData.append("quality", String(quality));

  const widthValue = maxWidth.trim();
  if (widthValue) {
    formData.append("max_width", widthValue);
  }
  if (format === "jpeg") {
    formData.append("jpeg_background", jpegBackground);
  }

  const response = await fetch(`${API_URL}/v1/convert`, {
    method: "POST",
    body: formData,
    signal,
  });

  if (!response.ok) {
    throw new Error(await readErrorMessage(response));
  }

  onProgress?.(78);
  const blob = await response.blob();
  const url = URL.createObjectURL(blob);

  return {
    url,
    fileName: fileNameFromResponse(response) ?? outputFileName(file.name, format),
    size: blob.size,
    inputSize: numberHeader(response, "x-input-size"),
    inputWidth: numberHeader(response, "x-input-width"),
    inputHeight: numberHeader(response, "x-input-height"),
    outputWidth: numberHeader(response, "x-output-width"),
    outputHeight: numberHeader(response, "x-output-height"),
  };
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

function numberHeader(response: Response, header: string) {
  const value = response.headers.get(header);
  if (!value) {
    return null;
  }
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}
