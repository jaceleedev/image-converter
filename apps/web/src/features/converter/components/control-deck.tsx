import {
  Alert,
  Button,
  Chip,
  Input,
  Separator,
  Slider,
  Spinner,
  Surface,
  Tooltip,
} from "@heroui/react";
import {
  AlertCircleIcon,
  ArrowDownToLineIcon,
  FileImageIcon,
  GaugeIcon,
  PaletteIcon,
  RotateCcwIcon,
  RulerIcon,
  SlidersHorizontalIcon,
  SparklesIcon,
} from "lucide-react";
import type * as React from "react";

import { colorPresets, widthPresets } from "../constants";
import type {
  ConverterActions,
  ConverterDerived,
  ConverterState,
} from "../hooks/use-converter";
import {
  clampQuality,
  safeColor,
} from "../lib/format";
import { DownloadButton } from "./download-button";
import { FormatPicker } from "./format-picker";
import { ProgressPanel } from "./progress-panel";
import { SectionTitle } from "./section-title";
import { statusColor, statusLabel } from "./status";
import { SettingBlock } from "./setting-block";
import { UploadDock } from "./upload-dock";
import { cn } from "@/lib/utils";

export function ControlDeck({
  actions,
  derived,
  inputRef,
  state,
}: {
  actions: ConverterActions;
  derived: ConverterDerived;
  inputRef: React.RefObject<HTMLInputElement | null>;
  state: ConverterState;
}) {
  const {
    error,
    file,
    format,
    isDragging,
    jpegBackground,
    maxWidth,
    meta,
    progress,
    quality,
    result,
    stage,
  } = state;
  const {
    canConvert,
    isBusy,
    projectedWidth,
    qualityDisabled,
    reduction,
    selectedFormat,
  } = derived;
  const {
    convert,
    reset,
    selectFile,
    setFormat,
    setIsDragging,
    setJpegBackground,
    setMaxWidth,
    setQuality,
  } = actions;

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

        {isBusy ? <ProgressPanel progress={progress} /> : null}

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
