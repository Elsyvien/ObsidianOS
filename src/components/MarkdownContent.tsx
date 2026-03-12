import { Fragment, useMemo, type ReactNode } from "react";

import { MathFormula, normalizeLatexSource } from "./MathFormula";

type MarkdownContentProps = {
  className?: string;
  text: string;
};

type InlineMathVariant = "inline" | "display-inline";

export function MarkdownContent({ className, text }: MarkdownContentProps) {
  const content = useMemo(() => markdownToNodes(text), [text]);

  return <div className={`md-content ${className ?? ""}`}>{content}</div>;
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

function markdownToNodes(source: string): ReactNode[] {
  const normalizedSource = normalizeMarkdownSource(source);
  const parts = normalizedSource.split(/(\$\$[\s\S]*?\$\$|\\\[[\s\S]*?\\\])/g);

  const output: ReactNode[] = [];

  parts.forEach((part, index) => {
    if (!part) {
      return;
    }

    if (part.startsWith("$$") && part.endsWith("$$")) {
      output.push(renderBlockMath(part.slice(2, -2), `block-math-${index}`));
      return;
    }

    if (part.startsWith("\\[") && part.endsWith("\\]")) {
      output.push(renderBlockMath(part.slice(2, -2), `slash-block-math-${index}`));
      return;
    }

    output.push(...processInlineBlock(part, `block-${index}`));
  });

  return output;
}

function renderBlockMath(latex: string, key: string) {
  return (
    <MathFormula
      key={key}
      className="md-math md-math--block"
      latex={normalizeLatexSource(latex)}
      showSource={false}
    />
  );
}

function renderInlineMath(latex: string, key: string, variant: InlineMathVariant = "inline") {
  const className =
    variant === "display-inline"
      ? "md-math md-math--inline md-math--display-inline"
      : "md-math md-math--inline";
  const inline = variant === "inline";

  return (
    <MathFormula
      key={key}
      className={className}
      display={false}
      inline={inline}
      latex={normalizeLatexSource(latex)}
      showSource={false}
    />
  );
}

function processInlineBlock(block: string, keyPrefix: string): ReactNode[] {
  const lines = block.split("\n");
  const output: ReactNode[] = [];
  const paragraph: string[] = [];
  const blockquote: string[] = [];

  const flushParagraph = () => {
    if (paragraph.length === 0) {
      return;
    }

    output.push(
      <p className="md-p" key={`${keyPrefix}-p-${output.length}`}>
        {parseInline(paragraph.join(" "), `${keyPrefix}-p-${output.length}`)}
      </p>,
    );
    paragraph.length = 0;
  };

  const flushBlockquote = () => {
    if (blockquote.length === 0) {
      return;
    }

    output.push(
      <blockquote className="md-blockquote" key={`${keyPrefix}-quote-${output.length}`}>
        {blockquote.map((entry, index) => (
          <p key={`${keyPrefix}-quote-line-${index}`}>
            {parseInline(entry, `${keyPrefix}-quote-line-${index}`)}
          </p>
        ))}
      </blockquote>,
    );
    blockquote.length = 0;
  };

  for (const raw of lines) {
    const line = raw.trim();

    if (!line) {
      flushParagraph();
      flushBlockquote();
      continue;
    }

    const headingMatch = line.match(/^(#{1,3})\s+(.+)$/);
    if (headingMatch) {
      flushParagraph();
      flushBlockquote();
      const level = headingMatch[1].length + 2;
      const headingContent = parseInline(headingMatch[2], `${keyPrefix}-heading-${output.length}`);

      if (level === 3) {
        output.push(
          <h3 className="md-heading" key={`${keyPrefix}-heading-${output.length}`}>
            {headingContent}
          </h3>,
        );
      } else if (level === 4) {
        output.push(
          <h4 className="md-heading" key={`${keyPrefix}-heading-${output.length}`}>
            {headingContent}
          </h4>,
        );
      } else {
        output.push(
          <h5 className="md-heading" key={`${keyPrefix}-heading-${output.length}`}>
            {headingContent}
          </h5>,
        );
      }
      continue;
    }

    if (line.startsWith("> ") || line === ">") {
      flushParagraph();
      blockquote.push(line === ">" ? "" : line.slice(2));
      continue;
    }

    flushBlockquote();

    const listMatch = line.match(/^[-*]\s+(.+)$/);
    if (listMatch) {
      flushParagraph();
      output.push(
        <div className="md-list-item" key={`${keyPrefix}-list-${output.length}`}>
          {"• "}
          {parseInline(listMatch[1], `${keyPrefix}-list-${output.length}`)}
        </div>,
      );
      continue;
    }

    if (/^-{3,}$/.test(line)) {
      flushParagraph();
      output.push(<hr className="md-hr" key={`${keyPrefix}-hr-${output.length}`} />);
      continue;
    }

    paragraph.push(line);
  }

  flushParagraph();
  flushBlockquote();

  return output;
}

function parseInline(text: string, keyPrefix: string): ReactNode[] {
  const output: ReactNode[] = [];
  const pattern =
    /(\$\$[\s\S]*?\$\$|\$[^$\n]+?\$|\\\[([\s\S]+?)\\\]|\\\(([\s\S]+?)\\\)|\*\*(.+?)\*\*|\*(.+?)\*|\[\[([^\]]+?)\]\]|`([^`]+?)`)/g;

  let lastIndex = 0;
  let matchIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(text)) !== null) {
    if (match.index > lastIndex) {
      output.push(text.slice(lastIndex, match.index));
    }

    const fullMatch = match[0];
    const key = `${keyPrefix}-${matchIndex}`;

    if (fullMatch.startsWith("$$") && fullMatch.endsWith("$$")) {
      output.push(renderInlineMath(fullMatch.slice(2, -2), key, "display-inline"));
    } else if (fullMatch.startsWith("$") && fullMatch.endsWith("$")) {
      output.push(renderInlineMath(fullMatch.slice(1, -1), key));
    } else if (fullMatch.startsWith("\\[") && fullMatch.endsWith("\\]")) {
      output.push(renderInlineMath(fullMatch.slice(2, -2), key, "display-inline"));
    } else if (fullMatch.startsWith("\\(") && fullMatch.endsWith("\\)")) {
      output.push(renderInlineMath(fullMatch.slice(2, -2), key));
    } else if (fullMatch.startsWith("**") && fullMatch.endsWith("**")) {
      output.push(
        <strong className="md-bold" key={key}>
          {parseInline(fullMatch.slice(2, -2), `${key}-bold`)}
        </strong>,
      );
    } else if (fullMatch.startsWith("*") && fullMatch.endsWith("*")) {
      output.push(
        <em className="md-italic" key={key}>
          {parseInline(fullMatch.slice(1, -1), `${key}-italic`)}
        </em>,
      );
    } else if (fullMatch.startsWith("[[") && fullMatch.endsWith("]]")) {
      output.push(
        <span className="md-wikilink" key={key}>
          {fullMatch.slice(2, -2)}
        </span>,
      );
    } else if (fullMatch.startsWith("`") && fullMatch.endsWith("`")) {
      output.push(
        <code className="md-code" key={key}>
          {fullMatch.slice(1, -1)}
        </code>,
      );
    } else {
      output.push(fullMatch);
    }

    lastIndex = match.index + fullMatch.length;
    matchIndex += 1;
  }

  if (lastIndex < text.length) {
    output.push(text.slice(lastIndex));
  }

  return output.map((node, index) =>
    typeof node === "string" ? <Fragment key={`${keyPrefix}-text-${index}`}>{node}</Fragment> : node,
  );
}
