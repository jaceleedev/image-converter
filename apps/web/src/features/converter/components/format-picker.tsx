import { Label, Radio, RadioGroup } from "@heroui/react";

import { formatOptions } from "../constants";
import type { OutputFormat } from "../types";
import { cn } from "@/lib/utils";

export function FormatPicker({
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
