import type { FormatOption } from "./types";

export const API_URL =
  process.env.NEXT_PUBLIC_CONVERT_API_URL ?? "http://localhost:4000";

export const supportedInputExtensions = [
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

export const inputAccept = [
  "image/*",
  ...supportedInputExtensions.map((extension) => `.${extension}`),
].join(",");

export const formatOptions: FormatOption[] = [
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

export const widthPresets = ["1280", "1600", "1920"] as const;

export const colorPresets = [
  { label: "흰색", value: "#FFFFFF" },
  { label: "검정", value: "#000000" },
  { label: "회색", value: "#F3F0EA" },
] as const;
