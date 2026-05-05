"use client";

import { div as MotionDiv } from "motion/react-client";
import * as React from "react";

import { cn } from "@/lib/utils";

type AnimatedContainerProps = React.ComponentProps<typeof MotionDiv> & {
  delay?: number;
};

export function AnimatedContainer({
  children,
  className,
  delay = 0,
  ...props
}: AnimatedContainerProps) {
  return (
    <MotionDiv
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.45, delay, ease: "easeOut" }}
      className={cn(className)}
      {...props}
    >
      {children}
    </MotionDiv>
  );
}
