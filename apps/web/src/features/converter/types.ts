export type OutputFormat = "webp" | "avif" | "png" | "jpeg";

export type Stage = "idle" | "ready" | "converting" | "done" | "error";

export type ImageMeta = {
  width: number;
  height: number;
};

export type ResultState = {
  url: string;
  fileName: string;
  size: number;
  inputSize: number | null;
  inputWidth: number | null;
  inputHeight: number | null;
  outputWidth: number | null;
  outputHeight: number | null;
};

export type FormatOption = {
  value: OutputFormat;
  label: string;
  note: string;
  detail: string;
  tone: string;
};
