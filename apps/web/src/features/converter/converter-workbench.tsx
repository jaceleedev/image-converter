"use client";

import { inputAccept } from "./constants";
import { ControlDeck } from "./components/control-deck";
import { PreviewWorkspace } from "./components/preview-workspace";
import { useConverter } from "./hooks/use-converter";

export function ConverterWorkbench() {
  const { actions, derived, inputRef, state } = useConverter();

  return (
    <main className="min-h-screen overflow-hidden px-3 py-3 text-foreground sm:px-4 lg:px-5">
      <input
        ref={inputRef}
        type="file"
        accept={inputAccept}
        disabled={derived.isBusy}
        className="hidden"
        onChange={(event) => {
          void actions.selectFile(event.target.files?.item(0) ?? null);
        }}
      />

      <div className="mx-auto grid w-full max-w-[1560px] gap-4 xl:grid-cols-[420px_minmax(0,1fr)]">
        <ControlDeck
          actions={actions}
          derived={derived}
          inputRef={inputRef}
          state={state}
        />

        <PreviewWorkspace
          actions={actions}
          derived={derived}
          inputRef={inputRef}
          state={state}
        />
      </div>
    </main>
  );
}
