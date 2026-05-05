import type { Metadata } from "next";
import "@fontsource/pretendard/400.css";
import "@fontsource/pretendard/500.css";
import "@fontsource/pretendard/600.css";
import "@fontsource/pretendard/700.css";
import "./globals.css";

export const metadata: Metadata = {
  title: "이미지 컨버터",
  description: "브라우저에서 바로 쓰는 로컬 이미지 변환 도구",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="ko"
      className="light h-full antialiased"
      data-theme="light"
    >
      <body className="flex min-h-full flex-col bg-background text-foreground">
        {children}
      </body>
    </html>
  );
}
