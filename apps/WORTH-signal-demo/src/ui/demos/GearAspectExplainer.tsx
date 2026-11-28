import type { ReactNode } from "react";
import { GEAR_ASPECT_CODE } from "virtual:gear-code-evidence";

import "./gearAspectExplainer.css";

const KEYWORDS = new Set(["async", "await", "const", "let", "return"]);
const TOKEN_PATTERN =
  /(\/\/[^\n]*)|("(?:[^"\\]|\\.)*")|(\b\d[\d_]*(?:\.\d+)?\b)|([A-Za-z_$][\w$]*)|(=>|[^\sA-Za-z_$]+)|(\s+)/g;

export function GearAspectExplainer() {
  return (
    <section className="gear-aspect-explainer">
      <div className="gear-aspect-copy">
        <span>Declared aspects</span>
        <h3>Each commit records the edited aspects.</h3>
        <p>Each slider release submits a field change to the selected branch. The history shows which values each commit recorded.</p>
        <div className="gear-aspect-list">
          <AspectReadout description="Extrusion depth" id="thickness" />
          <AspectReadout description="Gear count" id="teeth" />
          <AspectReadout description="Hole size" id="innerRadius" />
        </div>
      </div>

      <figure className="gear-aspect-code">
        <figcaption>
          <span aria-hidden="true"><i /><i /><i /></span>
          <code>production excerpts · slider values stay dynamic</code>
        </figcaption>
        <pre><code>
          {GEAR_ASPECT_CODE.split("\n").map((line, lineIndex) => (
            <span className="gear-code-line" key={lineIndex}>
              <span aria-hidden="true" className="gear-code-line-number">{lineIndex + 1}</span>
              <span>{tokenizeLine(line, `gear-${lineIndex}`)}</span>
            </span>
          ))}
        </code></pre>
      </figure>
    </section>
  );
}

function AspectReadout({
  description,
  id,
}: {
  description: string;
  id: string;
}) {
  return (
    <div>
      <code>{id}</code>
      <span>{description}</span>
      <strong>number · exact</strong>
    </div>
  );
}

function tokenizeLine(line: string, lineKey: string): ReactNode[] {
  const tokens: ReactNode[] = [];
  const pattern = new RegExp(TOKEN_PATTERN.source, "g");
  let match: RegExpExecArray | null;
  let tokenIndex = 0;

  while ((match = pattern.exec(line)) !== null) {
    const [text, comment, string, number, word, punctuation] = match;
    const key = `${lineKey}-${tokenIndex}`;
    tokenIndex += 1;
    const className = comment !== undefined
      ? "comment"
      : string !== undefined
        ? "string"
        : number !== undefined
          ? "number"
          : word !== undefined
            ? classifyWord(word, line.slice(pattern.lastIndex))
            : punctuation !== undefined
              ? "punctuation"
              : "";
    tokens.push(className
      ? <span className={`gear-code-${className}`} key={key}>{text}</span>
      : <span key={key}>{text}</span>);
  }
  return tokens;
}

function classifyWord(word: string, remainingLine: string) {
  if (KEYWORDS.has(word)) return "keyword";
  const nextCharacter = remainingLine.match(/^\s*(.)/)?.[1];
  if (nextCharacter === "(") return "function";
  if (nextCharacter === ":") return "property";
  return "variable";
}
