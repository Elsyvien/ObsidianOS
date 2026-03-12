import { useEffect, useRef } from "react";

import { ensureMathJax, normalizeLatexSource } from "./MathFormula";

/**
 * Lightweight Markdown + LaTeX renderer.
 *
 * Handles the subset of markdown/LaTeX that commonly appears in Obsidian notes:
 * - Block math: `$$...$$`, `\[...\]`
 * - Inline math: `$...$`, `\(...\)`
 * - Bold: `**...**`
 * - Italic: `*...*`
 * - Obsidian wikilinks: `[[...]]`
 * - Headings: `# ...` through `### ...`
 * - Blockquotes: `> ...`
 *
 * Uses the same MathJax runtime already loaded by `MathFormula`.
 */

type MarkdownContentProps = {
  className?: string;
  text: string;
};

export function MarkdownContent({ className, text }: MarkdownContentProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);

  const html = markdownToHtml(text);

  // After rendering, typeset any LaTeX that ended up in the DOM.
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    let active = true;

    const mathElements = [...el.querySelectorAll<HTMLElement>(".md-math")];
    if (mathElements.length === 0) return;

    void ensureMathJax()
      .then(async () => {
        if (!active || !window.MathJax?.typesetPromise) return;

        for (const element of mathElements) {
          const latex = element.dataset.latex ?? "";
          const display = element.dataset.display === "block";
          element.dataset.typeset = "false";
          element.textContent = display ? `\\[${latex}\\]` : `\\(${latex}\\)`;
        }

        window.MathJax.typesetClear?.(mathElements);
        window.MathJax.texReset?.();
        await window.MathJax.typesetPromise(mathElements);

        if (!active) {
          return;
        }

        for (const element of mathElements) {
          element.dataset.typeset = "true";
        }
      })
      .catch((error) => {
        for (const element of mathElements) {
          element.textContent = element.dataset.latex ?? "";
          element.dataset.typeset = "true";
        }

        console.error("Failed to render markdown LaTeX", { error, text });
      });

    return () => {
      active = false;
      if (window.MathJax?.typesetClear) {
        window.MathJax.typesetClear(mathElements);
      }
    };
  }, [html]);

  return (
    <div
      className={`md-content ${className ?? ""}`}
      ref={containerRef}
      // biome-ignore lint/security/noDangerouslySetInnerHtml: controlled markdown parsing
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

/* ── Markdown → HTML converter ────────────────────── */

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function normalizeMarkdownSource(source: string): string {
  const normalized = source.replace(/\r\n?/g, "\n");
  const escapedDelimiterCount = (normalized.match(/\\\\[\[\]()]/g) ?? []).length;
  const escapedCommandCount = (normalized.match(/\\\\[A-Za-z]/g) ?? []).length;
  const unescaped =
    escapedDelimiterCount > 0 || escapedCommandCount >= 2
      ? normalized.replace(/\\\\/g, "\\")
      : normalized;

  return unescaped.replace(/\\\s+([\[\]\(\)])/g, "\\$1");
}

function renderMathPlaceholder(latex: string, variant: "block" | "inline" | "display-inline") {
  const normalizedLatex = normalizeLatexSource(latex);
  const className =
    variant === "block"
      ? "md-math md-math--block"
      : variant === "display-inline"
        ? "md-math md-math--inline md-math--display-inline"
        : "md-math md-math--inline";
  const tagName = variant === "block" || variant === "display-inline" ? "div" : "span";
  const displayMode = variant === "block" ? "block" : "inline";
  const encodedLatex = escapeHtml(normalizedLatex);

  return `<${tagName} class="${className}" data-display="${displayMode}" data-latex="${encodedLatex}" data-typeset="false" aria-label="${encodedLatex}"></${tagName}>`;
}

function markdownToHtml(source: string): string {
  const normalizedSource = normalizeMarkdownSource(source);

  // First, split by block math ($$...$$, \[...\]) — these are paragraphs on their own.
  const parts = normalizedSource.split(/(\$\$[\s\S]*?\$\$|\\\[[\s\S]*?\\\])/g);

  const rendered = parts.map((part) => {
    // Block math
    if (part.startsWith("$$") && part.endsWith("$$")) {
      const latex = part.slice(2, -2).trim();
      return renderMathPlaceholder(latex, "block");
    }

    if (part.startsWith("\\[") && part.endsWith("\\]")) {
      const latex = part.slice(2, -2).trim();
      return renderMathPlaceholder(latex, "block");
    }

    // Process line-by-line for the rest
    return processInlineBlock(part);
  });

  return rendered.join("");
}

function processInlineBlock(block: string): string {
  const lines = block.split("\n");
  const output: string[] = [];
  const paragraph: string[] = [];
  let inBlockquote = false;

  const flushParagraph = () => {
    if (paragraph.length === 0) {
      return;
    }

    output.push(`<p class="md-p">${processInline(paragraph.join(" "))}</p>`);
    paragraph.length = 0;
  };

  for (const raw of lines) {
    const line = raw.trim();

    // Empty line → close blockquote if open, add spacing
    if (!line.trim()) {
      flushParagraph();
      if (inBlockquote) {
        output.push("</blockquote>");
        inBlockquote = false;
      }
      continue;
    }

    // Headings
    const headingMatch = line.match(/^(#{1,3})\s+(.+)$/);
    if (headingMatch) {
      flushParagraph();
      const level = headingMatch[1].length;
      const content = processInline(headingMatch[2]);
      output.push(`<h${level + 2} class="md-heading">${content}</h${level + 2}>`);
      continue;
    }

    // Blockquote
    if (line.startsWith("> ") || line === ">") {
      flushParagraph();
      const content = processInline(line.slice(2));
      if (!inBlockquote) {
        output.push('<blockquote class="md-blockquote">');
        inBlockquote = true;
      }
      output.push(`<p>${content}</p>`);
      continue;
    }

    // Close open blockquote for non-quote lines
    if (inBlockquote) {
      output.push("</blockquote>");
      inBlockquote = false;
    }

    // List items
    const listMatch = line.match(/^[-*]\s+(.+)$/);
    if (listMatch) {
      flushParagraph();
      output.push(`<div class="md-list-item">• ${processInline(listMatch[1])}</div>`);
      continue;
    }

    // Horizontal rule
    if (/^-{3,}$/.test(line.trim())) {
      flushParagraph();
      output.push('<hr class="md-hr" />');
      continue;
    }

    paragraph.push(line);
  }

  flushParagraph();

  if (inBlockquote) {
    output.push("</blockquote>");
  }

  return output.join("");
}

function processInline(text: string): string {
  let remaining = escapeHtml(text);
  const mathSegments: string[] = [];

  const stashMath = (html: string) => {
    const token = `@@MD_MATH_${mathSegments.length}@@`;
    mathSegments.push(html);
    return token;
  };

  // Process inline patterns in order of priority.
  // Math is stashed first so markdown formatting cannot corrupt LaTeX content.

  // Inline math: $...$  (but not $$)
  remaining = remaining.replace(
    /\$([^$\n]+?)\$/g,
    (_m, latex: string) =>
      stashMath(renderMathPlaceholder(latex, "inline")),
  );

  // Inline math: \[...\]
  remaining = remaining.replace(
    /\\\[([\s\S]+?)\\\]/g,
    (_m, latex: string) =>
      stashMath(renderMathPlaceholder(latex, "display-inline")),
  );

  // Inline math: \(...\)
  remaining = remaining.replace(
    /\\\(([\s\S]+?)\\\)/g,
    (_m, latex: string) =>
      stashMath(renderMathPlaceholder(latex, "inline")),
  );

  // Bold: **...**
  remaining = remaining.replace(
    /\*\*(.+?)\*\*/g,
    (_m, content: string) => `<strong class="md-bold">${content}</strong>`,
  );

  // Italic: *...*
  remaining = remaining.replace(
    /\*(.+?)\*/g,
    (_m, content: string) => `<em class="md-italic">${content}</em>`,
  );

  // Wikilinks: [[...]]
  remaining = remaining.replace(
    /\[\[([^\]]+?)\]\]/g,
    (_m, content: string) => `<span class="md-wikilink">${content}</span>`,
  );

  // Inline code: `...`
  remaining = remaining.replace(
    /`([^`]+?)`/g,
    (_m, content: string) => `<code class="md-code">${content}</code>`,
  );

  return remaining.replace(/@@MD_MATH_(\d+)@@/g, (_m, index: string) => mathSegments[Number(index)] ?? "");
}
