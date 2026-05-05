import type { Stage } from "../types";

export function statusLabel(stage: Stage) {
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

export function stageDescription(stage: Stage) {
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

export function statusColor(
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
