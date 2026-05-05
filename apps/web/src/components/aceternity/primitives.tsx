"use client";

import * as React from "react";

import { cn } from "@/lib/utils";

type BadgeProps = React.HTMLAttributes<HTMLSpanElement> & {
  variant?: "default" | "secondary" | "outline" | "destructive";
};

export function Badge({
  className,
  variant = "default",
  ...props
}: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex h-5 w-fit shrink-0 items-center justify-center rounded-full border px-2 text-xs font-medium whitespace-nowrap",
        variant === "default" &&
          "border-transparent bg-primary text-primary-foreground",
        variant === "secondary" &&
          "border-transparent bg-secondary text-secondary-foreground",
        variant === "outline" && "border-border bg-transparent text-foreground",
        variant === "destructive" &&
          "border-transparent bg-destructive/10 text-destructive",
        className
      )}
      {...props}
    />
  );
}

type ButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  render?: React.ReactElement<React.AnchorHTMLAttributes<HTMLAnchorElement>>;
  variant?: "default" | "outline" | "secondary" | "ghost" | "destructive";
};

export function Button({
  children,
  className,
  render,
  variant = "default",
  ...props
}: ButtonProps) {
  const buttonClassName = cn(
    "inline-flex h-8 shrink-0 items-center justify-center gap-1.5 rounded-lg border px-2.5 text-sm font-medium whitespace-nowrap transition-all outline-none",
    "focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
    variant === "default" &&
      "border-transparent bg-primary text-primary-foreground hover:bg-primary/90",
    variant === "outline" &&
      "border-border bg-background hover:bg-muted hover:text-foreground",
    variant === "secondary" &&
      "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
    variant === "ghost" && "border-transparent hover:bg-muted",
    variant === "destructive" &&
      "border-transparent bg-destructive/10 text-destructive hover:bg-destructive/20",
    className
  );

  if (render) {
    return React.cloneElement(render, {
      className: cn(buttonClassName, render.props.className),
      children,
    });
  }

  return (
    <button className={buttonClassName} {...props}>
      {children}
    </button>
  );
}

export function Input({
  className,
  type,
  ...props
}: React.ComponentProps<"input">) {
  return (
    <input
      type={type}
      className={cn(
        "h-8 w-full min-w-0 rounded-lg border border-input bg-transparent px-2.5 py-1 text-base transition-colors outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 md:text-sm",
        className
      )}
      {...props}
    />
  );
}

type AlertProps = React.HTMLAttributes<HTMLDivElement> & {
  variant?: "default" | "destructive";
};

export function Alert({
  className,
  variant = "default",
  ...props
}: AlertProps) {
  return (
    <div
      role="alert"
      className={cn(
        "relative grid w-full gap-0.5 rounded-lg border px-2.5 py-2 text-left text-sm has-[>svg]:grid-cols-[auto_1fr] has-[>svg]:gap-x-2 [&>svg]:row-span-2 [&>svg]:mt-0.5 [&>svg]:size-4",
        variant === "default" && "bg-card text-card-foreground",
        variant === "destructive" && "bg-card text-destructive",
        className
      )}
      {...props}
    />
  );
}

export function AlertTitle({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("font-medium", className)} {...props} />;
}

export function AlertDescription({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("text-sm text-muted-foreground", className)}
      {...props}
    />
  );
}

export function Progress({
  className,
  value = 0,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & { value?: number }) {
  const safeValue = Math.min(100, Math.max(0, value));

  return (
    <div
      className={cn("h-1 w-full overflow-hidden rounded-full bg-muted", className)}
      {...props}
    >
      <div
        className="h-full rounded-full bg-primary transition-all"
        style={{ width: `${safeValue}%` }}
      />
    </div>
  );
}

export function Separator({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("h-px w-full bg-border", className)} {...props} />;
}

type SliderProps = Omit<
  React.InputHTMLAttributes<HTMLInputElement>,
  "onChange" | "type" | "value"
> & {
  onValueChange?: (value: number) => void;
  value: number;
};

export function Slider({
  className,
  max = 100,
  min = 0,
  onValueChange,
  step = 1,
  value,
  ...props
}: SliderProps) {
  return (
    <input
      type="range"
      className={cn(
        "h-2 w-full cursor-pointer appearance-none rounded-full bg-muted accent-primary disabled:cursor-not-allowed disabled:opacity-50",
        className
      )}
      max={max}
      min={min}
      step={step}
      value={value}
      onChange={(event) => onValueChange?.(Number(event.target.value))}
      {...props}
    />
  );
}

type ToggleGroupContextValue = {
  onValueChange?: (value: string[]) => void;
  value: string[];
};

const ToggleGroupContext = React.createContext<ToggleGroupContextValue>({
  value: [],
});

type ToggleGroupProps = React.HTMLAttributes<HTMLDivElement> & {
  onValueChange?: (value: string[]) => void;
  spacing?: number;
  value: string[];
  variant?: "default" | "outline";
};

export function ToggleGroup({
  children,
  className,
  onValueChange,
  spacing,
  value,
  variant,
  ...props
}: ToggleGroupProps) {
  void spacing;
  void variant;

  return (
    <ToggleGroupContext.Provider value={{ onValueChange, value }}>
      <div className={cn("flex w-fit items-center gap-1", className)} {...props}>
        {children}
      </div>
    </ToggleGroupContext.Provider>
  );
}

type ToggleGroupItemProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  value: string;
};

export function ToggleGroupItem({
  children,
  className,
  value,
  ...props
}: ToggleGroupItemProps) {
  const context = React.useContext(ToggleGroupContext);
  const pressed = context.value.includes(value);

  return (
    <button
      type="button"
      aria-pressed={pressed}
      className={cn(
        "inline-flex shrink-0 items-center justify-center rounded-lg border border-border bg-background px-3 py-2 text-sm font-medium transition-all outline-none hover:bg-muted focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50",
        pressed && "border-primary bg-primary/10 text-primary",
        className
      )}
      onClick={() => {
        context.onValueChange?.(
          pressed
            ? context.value.filter((item) => item !== value)
            : [...context.value, value]
        );
      }}
      {...props}
    >
      {children}
    </button>
  );
}
