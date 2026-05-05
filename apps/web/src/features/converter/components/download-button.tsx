import { Button } from "@heroui/react";
import { ArrowDownToLineIcon } from "lucide-react";
import type { ResultState } from "../types";

export function DownloadButton({
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
