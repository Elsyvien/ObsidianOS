import { useEffect, useRef, useState } from "react";
import mathJaxScriptUrl from "mathjax-full/es5/tex-svg-full.js?url";

type MathFormulaProps = {
  className?: string;
  display?: boolean;
  latex: string;
  showSource?: boolean;
  sourceClassName?: string;
};

type MathJaxWindow = {
  startup?: {
    promise?: Promise<unknown>;
    typeset?: boolean;
  };
  texReset?: () => void;
  typesetClear?: (elements?: HTMLElement[]) => void;
  typesetPromise?: (elements?: HTMLElement[]) => Promise<unknown>;
};

declare global {
  interface Window {
    MathJax?: MathJaxWindow & Record<string, unknown>;
  }
}

let mathJaxLoadPromise: Promise<void> | null = null;

function joinClasses(...values: Array<string | undefined>) {
  return values.filter(Boolean).join(" ");
}

function unwrapMathDelimiters(value: string) {
  const trimmed = value.trim();
  const wrapped =
    (trimmed.startsWith("$$") && trimmed.endsWith("$$") && trimmed.length > 4)
      ? trimmed.slice(2, -2)
      : (trimmed.startsWith("\\\\[") && trimmed.endsWith("\\\\]") && trimmed.length > 6)
        ? trimmed.slice(3, -3)
        : (trimmed.startsWith("\\[") && trimmed.endsWith("\\]") && trimmed.length > 4)
          ? trimmed.slice(2, -2)
          : (trimmed.startsWith("\\\\(") && trimmed.endsWith("\\\\)") && trimmed.length > 6)
            ? trimmed.slice(3, -3)
            : (trimmed.startsWith("\\(") && trimmed.endsWith("\\)") && trimmed.length > 4)
              ? trimmed.slice(2, -2)
              : null;

  return wrapped === null ? trimmed : wrapped.trim();
}

function shouldUnescapeLatex(value: string) {
  const escapedDelimiterCount = (value.match(/\\\\[\[\]()]/g) ?? []).length;
  const escapedCommandCount = (value.match(/\\\\[A-Za-z]/g) ?? []).length;

  return escapedDelimiterCount > 0 || escapedCommandCount >= 2;
}

function normalizeEscapedLatex(value: string) {
  if (!shouldUnescapeLatex(value)) {
    return value;
  }

  return value.replace(/\\\\/g, "\\");
}

export function normalizeLatexSource(value: string) {
  const unwrapped = unwrapMathDelimiters(value.replace(/\r\n?/g, "\n"));

  return normalizeEscapedLatex(unwrapped)
    .replace(/\\\s+([\[\]\(\)])/g, "\\$1")
    .trim();
}

export function looksLikeLatex(value: string) {
  const normalized = normalizeEscapedLatex(value.trim());

  if (!normalized) {
    return false;
  }

  return /\\[A-Za-z]+|[_^{}]|\$|\\[\[\(]/.test(normalized);
}

export function ensureMathJax() {
  if (window.MathJax?.typesetPromise) {
    return Promise.resolve();
  }

  if (mathJaxLoadPromise) {
    return mathJaxLoadPromise;
  }

  window.MathJax = {
    ...(window.MathJax ?? {}),
    startup: {
      ...(window.MathJax?.startup ?? {}),
      typeset: false,
    },
    svg: {
      fontCache: "none",
    },
    tex: {
      packages: {
        "[+]": ["ams", "newcommand", "noerrors", "noundefined"],
      },
    },
  };

  mathJaxLoadPromise = new Promise<void>((resolve, reject) => {
    const existing = document.querySelector('script[data-mathjax-loader="true"]') as HTMLScriptElement | null;

    const handleReady = () => {
      const startup = window.MathJax?.startup?.promise;
      if (!startup) {
        resolve();
        return;
      }

      startup.then(() => resolve()).catch(reject);
    };

    if (existing) {
      if (window.MathJax?.typesetPromise) {
        handleReady();
        return;
      }

      existing.addEventListener("load", handleReady, { once: true });
      existing.addEventListener("error", () => reject(new Error("Failed to load MathJax script.")), {
        once: true,
      });
      return;
    }

    const script = document.createElement("script");
    script.async = true;
    script.dataset.mathjaxLoader = "true";
    script.src = mathJaxScriptUrl;
    script.addEventListener("load", handleReady, { once: true });
    script.addEventListener("error", () => reject(new Error("Failed to load MathJax script.")), {
      once: true,
    });
    document.head.appendChild(script);
  });

  return mathJaxLoadPromise;
}

export function MathFormula({
  className,
  display = true,
  latex,
  showSource = true,
  sourceClassName,
}: MathFormulaProps) {
  const renderedRef = useRef<HTMLSpanElement | null>(null);
  const [isTypeset, setIsTypeset] = useState(false);
  const sourceClasses = sourceClassName ?? "math-formula__source";
  const normalizedLatex = normalizeLatexSource(latex);

  useEffect(() => {
    let active = true;
    const rendered = renderedRef.current;

    if (!rendered) {
      return;
    }

    setIsTypeset(false);

    void ensureMathJax()
      .then(async () => {
        if (!active || !renderedRef.current) {
          return;
        }

        const element = renderedRef.current;
        element.textContent = display ? `\\[${normalizedLatex}\\]` : `\\(${normalizedLatex}\\)`;
        window.MathJax?.typesetClear?.([element]);
        window.MathJax?.texReset?.();
        await window.MathJax?.typesetPromise?.([element]);

        if (active) {
          setIsTypeset(true);
        }
      })
      .catch((error) => {
        console.error("Failed to render LaTeX formula", { error, latex: normalizedLatex });
      });

    return () => {
      active = false;
      if (renderedRef.current) {
        window.MathJax?.typesetClear?.([renderedRef.current]);
      }
    };
  }, [display, normalizedLatex]);

  return (
    <div className={joinClasses("math-formula", className)}>
      <span
        aria-label={normalizedLatex}
        className="math-formula__rendered"
        data-typeset={isTypeset ? "true" : "false"}
        ref={renderedRef}
      />
      {showSource || !isTypeset ? <code className={sourceClasses}>{normalizedLatex}</code> : null}
    </div>
  );
}
