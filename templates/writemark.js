/*
 * <writemark-editor> v1.3.1 live inline Markdown editor.
 * Dependency-free. No network calls. Markdown source is canonical.
 */

const TAG_NAME = "writemark-editor";
const LEGACY_TAG_NAME = "md-live-editor";

const DEFAULTS = Object.freeze({
  mode: "live",
  markdownFlavor: "gfm",
  tabBehavior: "accessibility-first",
  indentString: "  ",
  placeholder: "Write markdown...",
  renderDebounceMs: 100,
  smallDocChars: 20_000,
  largeDocChars: 100_000,
  linkTarget: "_self",
  allowRawHtml: false,
  sanitize: true,
  emptyRequiredTrim: true,
});

const REFLECTED_ATTRIBUTES = [
  "name",
  "value",
  "label",
  "placeholder",
  "mode",
  "markdown-flavor",
  "tab-behavior",
  "indent-string",
  "required",
  "disabled",
  "readonly",
  "spellcheck",
  "maxlength",
  "minlength",
  "aria-label",
  "aria-labelledby",
  "dir",
];

const LANGUAGES = [
  "python", "javascript", "typescript", "tsx", "jsx", "html", "css", "json", "bash", "shell", "sh",
  "sql", "yaml", "toml", "xml", "markdown", "text", "go", "rust", "java", "c", "cpp", "csharp",
  "php", "ruby", "swift", "kotlin", "r", "scala", "dockerfile", "nginx", "graphql", "regex"
];

const ALIASES = new Map([
  ["py", "python"],
  ["js", "javascript"],
  ["ts", "typescript"],
  ["yml", "yaml"],
  ["md", "markdown"],
  ["rb", "ruby"],
  ["rs", "rust"],
  ["cs", "csharp"],
  ["kt", "kotlin"],
]);

const LIVE_ARROW_KEYS = new Set(["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"]);
const NAVIGATION_KEYS = new Set(["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End", "PageUp", "PageDown"]);
const ESCAPABLE_PUNCTUATION = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

// Nonograph-flavored inline markup. Unlike CommonMark emphasis, Nonograph uses
// literal paired same-delimiter wrapping processed left-to-right in a fixed
// order (mirrors safe_replace in src/parser.rs). Each entry maps a delimiter to
// the HTML element the server produces.
const NONOGRAPH_INLINE_RULES = [
  { marker: "**", tag: "strong" },
  { marker: "*", tag: "em" },
  { marker: "_", tag: "u" },
  { marker: "~", tag: "del" },
  { marker: "^", tag: "sup" },
  { marker: "==", tag: "mark" },
  { marker: "#", tag: "span", className: "secret" },
];

// Replicate one safe_replace pass over `source`, returning the wrapped ranges as
// { openStart, openEnd, closeStart, closeEnd } in source offsets. Semantics: find
// the first delimiter, then the next matching delimiter; if the enclosed content
// is non-empty and single-line, it is a pair; otherwise the opener is literal and
// scanning continues past it. Regions already claimed by earlier (higher-priority)
// passes are skipped so markers inside them are treated as literal text, matching
// the server which never re-wraps inside a produced tag's markers.
function nonographPairsForMarker(source, marker, claimed) {
  const pairs = [];
  const len = source.length;
  const isClaimed = pos => {
    for (const [from, to] of claimed) if (pos >= from && pos < to) return true;
    return false;
  };
  let i = 0;
  while (i <= len - marker.length) {
    if (isClaimed(i) || source.startsWith(marker, i) === false) { i += 1; continue; }
    const openStart = i;
    const contentStart = i + marker.length;
    // Find next matching marker after the opener that is not inside a claimed region.
    let j = contentStart;
    let closeStart = -1;
    while (j <= len - marker.length) {
      if (!isClaimed(j) && source.startsWith(marker, j)) { closeStart = j; break; }
      j += 1;
    }
    if (closeStart === -1) { i = contentStart; continue; }
    const content = source.slice(contentStart, closeStart);
    if (content.length === 0 || content.includes("\n")) { i = contentStart; continue; }
    const closeEnd = closeStart + marker.length;
    pairs.push({ openStart, openEnd: contentStart, closeStart, closeEnd });
    i = closeEnd;
  }
  return pairs;
}

// Compute all Nonograph inline emphasis spans for a single line of source text,
// in the server's processing order. Returns a flat list of { from, to, html }
// insertion points suitable for interleaving with the escaped source, where each
// span carries its opening/closing HTML and (for live mode) the retained marker.
function nonographEmphasisSpans(source, markerHtml = () => "") {
  const claimed = [];
  const spans = [];
  for (const rule of NONOGRAPH_INLINE_RULES) {
    const pairs = nonographPairsForMarker(source, rule.marker, claimed);
    for (const pair of pairs) {
      const openTag = rule.className
        ? `<${rule.tag} class="${rule.className}">`
        : `<${rule.tag}>`;
      const closeTag = `</${rule.tag}>`;
      spans.push({ from: pair.openStart, to: pair.openEnd, html: `${markerHtml(rule.marker)}${openTag}` });
      spans.push({ from: pair.closeStart, to: pair.closeEnd, html: `${closeTag}${markerHtml(rule.marker)}` });
      // Claim the entire wrapped region (markers included) so later, lower-priority
      // passes treat any delimiters inside as literal text.
      claimed.push([pair.openStart, pair.closeEnd]);
    }
  }
  return spans;
}

// Render `source` with Nonograph inline emphasis. Mirrors renderEmphasisMarkdown's
// output contract: escapes text, interleaves span HTML, and (via markerHtml) can
// retain the raw delimiters for live editing.
function renderNonographEmphasis(source, markerHtml = () => "") {
  const spans = nonographEmphasisSpans(source, markerHtml);
  spans.sort((a, b) => a.from - b.from || b.to - a.to);
  let html = "";
  let cursor = 0;
  for (const span of spans) {
    if (span.from < cursor) continue;
    html += escapeHtml(source.slice(cursor, span.from));
    html += span.html;
    cursor = span.to;
  }
  html += escapeHtml(source.slice(cursor));
  return html;
}

function now() { return Date.now(); }
function clamp(n, min, max) { return Math.max(min, Math.min(max, n)); }
function normalizeLineEndings(value) { return String(value ?? "").replace(/\r\n?/g, "\n"); }
function parseLengthConstraint(value) {
  if (value == null || !/^\d+$/.test(String(value).trim())) return null;
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) && parsed <= 2_147_483_647 ? parsed : null;
}
function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
function escapeAttribute(value) { return escapeHtml(value).replace(/`/g, "&#96;"); }
function stripHtml(value) { return String(value ?? "").replace(/<[^>]*>/g, ""); }
function isProbablyUrl(text) { return /^(https?:\/\/|mailto:|tel:|\/|#|\.\/|\.\.\/)[^\s]+$/i.test(String(text ?? "").trim()); }
function isSafeUrl(url, { allowDataImage = false } = {}) {
  const raw = String(url ?? "").trim();
  if (!raw) return false;
  const compact = raw.replace(/[\u0000-\u001F\u007F\s]+/g, "").toLowerCase();
  if (compact.startsWith("javascript:") || compact.startsWith("vbscript:") || compact.startsWith("file:")) return false;
  if (compact.startsWith("data:")) return allowDataImage && /^data:image\/(png|gif|jpe?g|webp);/i.test(compact);
  if (/^[a-z][a-z0-9+.-]*:/i.test(raw)) return /^(https?:|mailto:|tel:)/i.test(raw);
  return true;
}
function safeHref(url, opts = {}) { const raw = String(url ?? "").trim(); return raw === "" ? "" : isSafeUrl(raw, opts) ? raw : "#"; }
function headingSlug(value) {
  const text = String(value ?? "")
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/\\([\\`*_[\]{}()#+\-.!])/g, "$1")
    .replace(/[`*_~]/g, "")
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, "")
    .trim()
    .replace(/\s+/g, "-");
  return text || "section";
}
function assignHeadingIds(blocks) {
  const seen = new Map();
  for (const block of blocks) {
    if (block.type !== "heading" || !block.heading) continue;
    const base = headingSlug(block.heading.content);
    const count = seen.get(base) ?? 0;
    seen.set(base, count + 1);
    block.heading.id = count === 0 ? base : `${base}-${count}`;
  }
  return blocks;
}
function isEscapablePunctuation(char) { return Boolean(char) && ESCAPABLE_PUNCTUATION.includes(char); }
function isBackslashEscaped(source, index) {
  let slashes = 0;
  for (let i = index - 1; i >= 0 && source[i] === "\\"; i -= 1) slashes += 1;
  return slashes % 2 === 1;
}
function parseCodeSpanAt(source, index) {
  if (source[index] !== "`" || source[index - 1] === "`" || isBackslashEscaped(source, index)) return null;
  let markerLength = 1;
  while (source[index + markerLength] === "`") markerLength += 1;
  let cursor = index + markerLength;
  while (cursor < source.length) {
    if (source[cursor] !== "`") { cursor += 1; continue; }
    let closingLength = 1;
    while (source[cursor + closingLength] === "`") closingLength += 1;
    if (closingLength === markerLength) {
      return {
        from: index,
        to: cursor + closingLength,
        marker: source.slice(index, index + markerLength),
        content: source.slice(index + markerLength, cursor),
        contentStart: index + markerLength,
        contentEnd: cursor,
      };
    }
    cursor += closingLength;
  }
  return null;
}
function normalizeCodeSpanContent(content) {
  let value = String(content ?? "").replace(/\n/g, " ");
  if (/^\s[\s\S]*\s$/.test(value) && /\S/.test(value)) value = value.slice(1, -1);
  return value;
}

function splitLinkDestinationAndTitle(raw) {
  const value = String(raw ?? "").trim();
  if (!value) return { url: "", title: "" };
  const angle = /^<([^<>\n]*)>(?:\s+(["'])(.*?)\2)?\s*$/.exec(value);
  if (angle) return { url: angle[1], title: angle[3] || "" };
  const quoted = /^(.*?)\s+(["'])(.*?)\2\s*$/.exec(value);
  if (quoted && quoted[1].trim()) return { url: quoted[1].trim(), title: quoted[3] };
  return { url: value, title: "" };
}

function parseInlineLinkAt(text, start = 0) {
  const source = String(text ?? "");
  if (isBackslashEscaped(source, start)) return null;
  const bang = source[start] === "!" ? "!" : "";
  let i = start + bang.length;
  if (source[i] !== "[" || isBackslashEscaped(source, i)) return null;
  let escaped = false;
  let labelEnd = -1;
  let labelDepth = 1;
  for (let j = i + 1; j < source.length; j += 1) {
    const ch = source[j];
    if (escaped) { escaped = false; continue; }
    if (ch === "\\") { escaped = true; continue; }
    if (ch === "[") { labelDepth += 1; continue; }
    if (ch === "]") {
      labelDepth -= 1;
      if (labelDepth === 0) { labelEnd = j; break; }
    }
  }
  if (labelEnd === -1 || source[labelEnd + 1] !== "(") return null;
  const label = source.slice(i + 1, labelEnd);
  const destStart = labelEnd + 2;
  let depth = 0;
  let quote = "";
  escaped = false;
  for (let j = destStart; j < source.length; j += 1) {
    const ch = source[j];
    if (escaped) { escaped = false; continue; }
    if (ch === "\\") { escaped = true; continue; }
    if (quote) {
      if (ch === quote) quote = "";
      continue;
    }
    if (ch === "\"" || ch === "'") { quote = ch; continue; }
    if (ch === "(") { depth += 1; continue; }
    if (ch === ")") {
      if (depth > 0) { depth -= 1; continue; }
      const destination = splitLinkDestinationAndTitle(source.slice(destStart, j));
      return {
        bang,
        label,
        url: destination.url,
        title: destination.title,
        from: start,
        to: j + 1,
        labelStart: i + 1,
        labelEnd,
        full: source.slice(start, j + 1),
      };
    }
  }
  return null;
}

// Nonograph bare bracket link: [https://example.com] renders as a link whose
// text is the URL itself. Mirrors process_links in src/parser.rs: only fires
// when the bracket content starts with "http" and contains no line break, and
// is not an image (`![...]`) or a `[label](url)`/`[label][ref]` link.
function parseBareLinkAt(text, start = 0) {
  const source = String(text ?? "");
  if (source[start] !== "[" || isBackslashEscaped(source, start)) return null;
  if (source[start - 1] === "!") return null;
  let escaped = false;
  let end = -1;
  for (let j = start + 1; j < source.length; j += 1) {
    const ch = source[j];
    if (escaped) { escaped = false; continue; }
    if (ch === "\\") { escaped = true; continue; }
    if (ch === "\n") return null;
    if (ch === "]") { end = j; break; }
  }
  if (end === -1) return null;
  // A following "(" means this is a [label](url) link; defer to that parser.
  if (source[end + 1] === "(") return null;
  const url = source.slice(start + 1, end);
  if (!/^https?:\/\//i.test(url)) return null;
  return {
    url,
    from: start,
    to: end + 1,
    labelStart: start + 1,
    labelEnd: end,
    full: source.slice(start, end + 1),
  };
}

function normalizeReferenceLabel(label) {
  return String(label ?? "")
    .replace(/\\([!-/:-@[-`{-~])/g, "$1")
    .trim()
    .replace(/\s+/g, " ")
    .toLowerCase();
}

function parseReferenceDefinition(line) {
  const match = /^ {0,3}\[([^\]\n]+)\]:[ \t]*(.+?)\s*$/.exec(String(line ?? ""));
  if (!match) return null;
  const label = normalizeReferenceLabel(match[1]);
  if (!label) return null;
  const destination = splitLinkDestinationAndTitle(match[2]);
  if (!destination.url) return null;
  return { label, ...destination };
}

function extractReferenceDefinitions(markdown, inherited = null) {
  const references = new Map(inherited instanceof Map ? inherited : []);
  const lines = normalizeLineEndings(markdown).split("\n");
  const output = lines.map(line => {
    const definition = parseReferenceDefinition(line);
    if (!definition) return line;
    if (!references.has(definition.label)) references.set(definition.label, definition);
    return "";
  });
  return { markdown: output.join("\n"), references };
}

function parseReferenceLinkAt(text, references, start = 0) {
  if (!(references instanceof Map) || references.size === 0) return null;
  const source = String(text ?? "");
  if (isBackslashEscaped(source, start)) return null;
  const bang = source[start] === "!" ? "!" : "";
  const labelStartMarker = start + bang.length;
  if (source[labelStartMarker] !== "[" || isBackslashEscaped(source, labelStartMarker)) return null;
  let escaped = false;
  let depth = 1;
  let labelEnd = -1;
  for (let i = labelStartMarker + 1; i < source.length; i += 1) {
    const char = source[i];
    if (escaped) { escaped = false; continue; }
    if (char === "\\") { escaped = true; continue; }
    if (char === "[") { depth += 1; continue; }
    if (char !== "]") continue;
    depth -= 1;
    if (depth === 0) { labelEnd = i; break; }
  }
  if (labelEnd === -1 || source[labelEnd + 1] === "(") return null;
  const label = source.slice(labelStartMarker + 1, labelEnd);
  let referenceLabel = label;
  let to = labelEnd + 1;
  if (source[to] === "[") {
    const referenceEnd = source.indexOf("]", to + 1);
    if (referenceEnd === -1 || isBackslashEscaped(source, referenceEnd)) return null;
    referenceLabel = source.slice(to + 1, referenceEnd) || label;
    to = referenceEnd + 1;
  }
  const reference = references.get(normalizeReferenceLabel(referenceLabel));
  if (!reference) return null;
  return {
    bang,
    label,
    url: reference.url,
    title: reference.title,
    from: start,
    to,
    labelStart: labelStartMarker + 1,
    labelEnd,
    full: source.slice(start, to),
  };
}

function isInlineWhitespace(char) {
  return !char || /\s/u.test(char);
}

function isInlinePunctuation(char) {
  return Boolean(char) && /[\p{P}\p{S}]/u.test(char);
}

// Emphasis is handled by the Nonograph inline engine defined near the top of
// this file (nonographEmphasisSpans / renderNonographEmphasis).

function htmlToMarkdown(html) {
  const template = document.createElement("template");
  template.innerHTML = String(html ?? "");
  const escapeMd = text => String(text ?? "").replace(/\u00a0/g, " ").replace(/\n{3,}/g, "\n\n");
  const walk = node => {
    if (node.nodeType === Node.TEXT_NODE) return escapeMd(node.nodeValue);
    if (node.nodeType !== Node.ELEMENT_NODE) return "";
    const tag = node.tagName.toLowerCase();
    const children = () => Array.from(node.childNodes).map(walk).join("");
    const block = text => `\n\n${text.trim()}\n\n`;
    if (tag === "br") return "\n";
    if (/^h[1-6]$/.test(tag)) return block(`${"#".repeat(Math.min(4, Number(tag[1])))} ${children().trim()}`);
    if (tag === "strong" || tag === "b") return `**${children()}**`;
    if (tag === "em" || tag === "i") return `*${children()}*`;
    if (tag === "code" && node.parentElement?.tagName?.toLowerCase() !== "pre") return `\`${children()}\``;
    if (tag === "pre") return block(`\`\`\`\n${node.textContent.replace(/\n+$/g, "")}\n\`\`\``);
    if (tag === "blockquote") return block(children().trim().split("\n").map(line => `> ${line}`).join("\n"));
    if (tag === "a") { const href = node.getAttribute("href") || ""; const label = children().trim() || href; return href && isSafeUrl(href) ? `[${label}](${href})` : label; }
    if (tag === "img") { const src = node.getAttribute("src") || ""; const alt = node.getAttribute("alt") || ""; return src && isSafeUrl(src, { allowDataImage: false }) ? `![${alt}](${src})` : alt; }
    if (tag === "ul" || tag === "ol") {
      const items = Array.from(node.children).filter(el => el.tagName.toLowerCase() === "li");
      return block(items.map((li, i) => `${tag === "ol" ? `${i + 1}.` : "-"} ${Array.from(li.childNodes).map(walk).join("").trim()}`).join("\n"));
    }
    if (tag === "table") {
      const rows = Array.from(node.querySelectorAll("tr")).map(tr => Array.from(tr.children).map(cell => Array.from(cell.childNodes).map(walk).join("").replace(/\|/g, "\\|").trim()));
      if (!rows.length) return "";
      const cols = Math.max(...rows.map(r => r.length));
      const pad = r => Array.from({ length: cols }, (_, i) => r[i] || "");
      const header = pad(rows[0]);
      const body = rows.slice(1).map(pad);
      return block([`| ${header.join(" | ")} |`, `| ${Array.from({ length: cols }, () => "---").join(" | ")} |`, ...body.map(r => `| ${r.join(" | ")} |`)].join("\n"));
    }
    if (["p", "div", "section", "article"].includes(tag)) return block(children());
    return children();
  };
  return Array.from(template.content.childNodes).map(walk).join("").replace(/\n{3,}/g, "\n\n").trim();
}
function tsvToMarkdownTable(text) {
  const rows = normalizeLineEndings(text).split("\n").filter(row => row.length > 0).map(row => row.split("\t").map(cell => cell.replace(/\|/g, "\\|").trim()));
  if (rows.length < 2 || rows.every(row => row.length < 2)) return null;
  const cols = Math.max(...rows.map(row => row.length));
  const pad = row => Array.from({ length: cols }, (_, i) => row[i] || "");
  const header = pad(rows[0]);
  const body = rows.slice(1).map(pad);
  return [`| ${header.join(" | ")} |`, `| ${Array.from({ length: cols }, () => "---").join(" | ")} |`, ...body.map(row => `| ${row.join(" | ")} |`)].join("\n");
}

function safeClipboardGet(clipboard, type) {
  try { return clipboard?.getData?.(type) || ""; } catch { return ""; }
}
function looksLikeMarkdown(text) {
  const source = normalizeLineEndings(text).trim();
  if (!source) return false;
  return looksLikeBlockMarkdown(source)
    || /(^|\s)(\*\*[^*]+\*\*|__[^_]+__|~~[^~]+~~|`[^`\n]+`|!\[[^\]]*\]\([^)]+\)|\[[^\]]+\]\([^)]+\))/m.test(source);
}
function looksLikeBlockMarkdown(text) {
  const source = normalizeLineEndings(text).trim();
  if (!source) return false;
  const lines = source.split("\n");
  if (lines.some(line => /^(\s{0,3}#{1,4}\s+|\s*([-+*])\s+|\s*\d+[.)]\s+|\s*[-+*]\s+\[(?: |x|X)\]\s+|\s*>\s?|\s*```|\s*~~~)/.test(line))) return true;
  if (lines.some(line => isHorizontalRule(line))) return true;
  if (lines.length >= 2 && isLikelyTableRow(lines[0]) && isTableDelimiter(lines[1])) return true;
  return false;
}
function markdownFromClipboardData(clipboard) {
  const explicit = safeClipboardGet(clipboard, "text/markdown") || safeClipboardGet(clipboard, "text/x-markdown");
  const text = normalizeLineEndings(safeClipboardGet(clipboard, "text/plain"));
  const html = safeClipboardGet(clipboard, "text/html");
  const table = text ? tsvToMarkdownTable(text) : null;
  if (table) return { markdown: table, kind: "table" };
  if (explicit) return { markdown: normalizeLineEndings(explicit), kind: "markdown" };
  if (html && (!text || !looksLikeMarkdown(text))) {
    const converted = htmlToMarkdown(html);
    if (converted) return { markdown: normalizeLineEndings(converted), kind: "html" };
  }
  if (text) return { markdown: text, kind: looksLikeMarkdown(text) ? "markdown" : "text" };
  if (html) {
    const converted = htmlToMarkdown(html);
    if (converted) return { markdown: normalizeLineEndings(converted), kind: "html" };
  }
  return { markdown: "", kind: "empty" };
}
function collectInlineMarkdownRanges(source) {
  const ranges = [];
  const addMatches = (regex, openLength, closeLength, labelGroup = 1, markerOffsetGroup = null) => {
    regex.lastIndex = 0;
    let match;
    while ((match = regex.exec(source))) {
      const label = match[labelGroup] ?? "";
      const markerOffset = markerOffsetGroup == null ? 0 : (match[markerOffsetGroup]?.length ?? 0);
      const from = match.index + markerOffset;
      const to = match.index + match[0].length;
      const innerFrom = from + openLength;
      const innerTo = to - closeLength;
      if (isBackslashEscaped(source, from) || isBackslashEscaped(source, to - closeLength)) continue;
      if (innerTo >= innerFrom && label.length >= 0) ranges.push({ from, to, innerFrom, innerTo });
      if (match.index === regex.lastIndex) regex.lastIndex += 1;
    }
  };
  // Nonograph inline markers (see NONOGRAPH_INLINE_RULES).
  addMatches(/\*\*([^*\n]+)\*\*/g, 2, 2);
  addMatches(/==([^=\n]+)==/g, 2, 2);
  addMatches(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, 1, 1, 2, 1);
  addMatches(/_([^_\n]+)_/g, 1, 1);
  addMatches(/~([^~\n]+)~/g, 1, 1);
  addMatches(/\^([^^\n]+)\^/g, 1, 1);
  addMatches(/#([^#\n]+)#/g, 1, 1);
  for (let i = 0; i < source.length; i += 1) {
    const code = parseCodeSpanAt(source, i);
    if (!code) continue;
    ranges.push({ from: code.from, to: code.to, innerFrom: code.contentStart, innerTo: code.contentEnd });
    i = code.to - 1;
  }
  for (let i = 0; i < source.length; i += 1) {
    const link = parseInlineLinkAt(source, i) || parseBareLinkAt(source, i);
    if (!link) continue;
    ranges.push({ from: link.from, to: link.to, innerFrom: link.labelStart, innerTo: link.labelEnd });
    i = link.to - 1;
  }
  return ranges.sort((a, b) => (a.to - a.from) - (b.to - b.from));
}
function expandMarkdownFormattingRange(value, start, end) {
  let s = clamp(start, 0, value.length);
  let e = clamp(end, 0, value.length);
  if (s > e) [s, e] = [e, s];
  if (s === e) return { start: s, end: e };
  let changed = true;
  while (changed) {
    changed = false;
    const startLine = getLineRange(value, s);
    const endLine = getLineRange(value, e);
    if (startLine.start === endLine.start) {
      const lineInfo = makeLineInfo(startLine.start, startLine.end, startLine.text);
      const list = parseListItem(lineInfo.text);
      const heading = parseHeading(lineInfo.text);
      const quote = parseBlockquote(lineInfo.text);
      const contentStart = heading?.contentStart ?? list?.contentStart ?? quote?.contentStart;
      if (Number.isFinite(contentStart) && s === lineInfo.start + contentStart && e === lineInfo.end) {
        s = lineInfo.start;
        e = lineInfo.end;
        changed = true;
        continue;
      }
      const localStart = s - lineInfo.start;
      const localEnd = e - lineInfo.start;
      for (const r of collectInlineMarkdownRanges(lineInfo.text)) {
        if (localStart === r.innerFrom && localEnd === r.innerTo) {
          s = lineInfo.start + r.from;
          e = lineInfo.start + r.to;
          changed = true;
          break;
        }
      }
    }
  }
  return { start: s, end: e };
}
function normalizeIndentAttribute(value) {
  if (value === "tab" || value === "\\t") return "\t";
  if (value === "4" || value === "4-spaces") return "    ";
  if (value === "2" || value === "2-spaces") return "  ";
  if (value === "\t" || value === "  " || value === "    ") return value;
  return DEFAULTS.indentString;
}
function displayShortcut(shortcut) {
  if (!shortcut) return "";
  const isMac = /Mac|iPhone|iPad|iPod/.test(globalThis.navigator?.platform ?? "");
  return shortcut.replace(/Mod/g, isMac ? "⌘" : "Ctrl").replace(/Alt/g, isMac ? "⌥" : "Alt").replace(/Shift/g, isMac ? "⇧" : "Shift");
}
function uid(prefix = "mfe") { return `${prefix}-${Math.random().toString(36).slice(2)}`; }

function getLines(value) {
  const source = normalizeLineEndings(value);
  const lines = [];
  let start = 0;
  for (let i = 0; i <= source.length; i += 1) {
    if (i === source.length || source[i] === "\n") {
      lines.push({ index: lines.length, start, end: i, text: source.slice(start, i), newlineEnd: i < source.length ? i + 1 : i });
      start = i + 1;
    }
  }
  if (source.length === 0) lines.length = 0;
  return lines;
}

function getLineRange(value, offset) {
  const source = normalizeLineEndings(value);
  const safe = clamp(offset, 0, source.length);
  const before = source.lastIndexOf("\n", Math.max(0, safe - 1));
  const start = before === -1 ? 0 : before + 1;
  const after = source.indexOf("\n", safe);
  const end = after === -1 ? source.length : after;
  return { start, end, text: source.slice(start, end) };
}

function getSelectedLineRanges(value, selectionStart, selectionEnd, opts = {}) {
  const startLine = getLineRange(value, selectionStart);
  const endProbe = selectionEnd > selectionStart && value[selectionEnd - 1] === "\n" ? selectionEnd - 1 : selectionEnd;
  const endLine = getLineRange(value, endProbe);
  const out = [];
  let cursor = startLine.start;
  while (cursor <= endLine.start) {
    const line = getLineRange(value, cursor);
    out.push(makeLineInfo(line.start, line.end, line.text, opts));
    if (line.end >= value.length) break;
    cursor = line.end + 1;
  }
  return out;
}

function makeLineInfo(start, end, text, opts = {}) {
  const indent = (/^(\s*)/.exec(text) || ["", ""])[1];
  const list = parseListItem(text, opts);
  const contentStart = list ? start + list.contentStart : start + indent.length;
  return { start, end, text, indent, marker: list?.markerText ?? null, contentStart };
}

function usesGfm(opts = {}) { return opts.gfm ?? opts.markdownFlavor !== "commonmark"; }

function parseListItem(line, opts = {}) {
  if (usesGfm(opts)) {
    const task = /^(\s*)([-+*])\s+\[( |x|X)\]\s+(.*)$/.exec(line);
    if (task) {
      const markerText = `${task[2]} [${task[3]}] `;
      return { kind: "task-list-item", listType: "ul", indent: task[1], marker: task[2], markerText, checked: task[3].toLowerCase() === "x", content: task[4], contentStart: task[1].length + markerText.length, fullMarkerStart: task[1].length, fullMarkerEnd: task[1].length + markerText.length };
    }
  }
  const ordered = /^(\s*)(\d+)([.)])\s+(.*)$/.exec(line);
  if (ordered) {
    const markerText = `${ordered[2]}${ordered[3]} `;
    return { kind: "ordered-list-item", listType: "ol", indent: ordered[1], marker: ordered[2], number: Number(ordered[2]), delimiter: ordered[3], markerText, content: ordered[4], contentStart: ordered[1].length + markerText.length, fullMarkerStart: ordered[1].length, fullMarkerEnd: ordered[1].length + markerText.length };
  }
  const bullet = /^(\s*)([-+*])\s+(.*)$/.exec(line);
  if (bullet) {
    const markerText = `${bullet[2]} `;
    return { kind: "bullet-list-item", listType: "ul", indent: bullet[1], marker: bullet[2], markerText, content: bullet[3], contentStart: bullet[1].length + markerText.length, fullMarkerStart: bullet[1].length, fullMarkerEnd: bullet[1].length + markerText.length };
  }
  return null;
}

function parseHeading(line) {
  const m = /^(\s{0,3})(#{1,4})(?:([ \t]+)(.*)|[ \t]*)$/.exec(line);
  if (!m) return null;
  let content = m[4] ?? "";
  if (/^#+[ \t]*$/.test(content)) content = "";
  else {
    const closing = /^(.*?)[ \t]+#+[ \t]*$/.exec(content);
    content = closing ? closing[1] : content.replace(/[ \t]+$/, "");
  }
  const separator = m[3] ?? "";
  return {
    indent: m[1],
    level: m[2].length,
    markerText: `${m[2]}${separator}`,
    content,
    contentStart: m[1].length + m[2].length + separator.length,
  };
}
function parseBlockquote(line) {
  const m = /^(\s*>\s?)(.*)$/.exec(line);
  if (!m) return null;
  let depth = 1;
  let fullContentStart = m[1].length;
  let nestedContent = m[2];
  while (true) {
    const nested = /^(\s*>\s?)(.*)$/.exec(nestedContent);
    if (!nested) break;
    depth += 1;
    fullContentStart += nested[1].length;
    nestedContent = nested[2];
  }
  return {
    markerText: m[1],
    content: m[2],
    contentStart: m[1].length,
    depth,
    fullContentStart,
  };
}
// Nonograph dividers are exactly ***, -*-, ---, or === on their own line
// (mirrors process_dividers in src/parser.rs). Note: **** (four stars, e.g. from
// bolding an empty line) is intentionally NOT a divider.
function isHorizontalRule(line) { const t = line.trim(); return t === "***" || t === "-*-" || t === "---" || t === "==="; }
function getFenceInfo(line) {
  const m = /^(\s{0,3})(`{3,}|~{3,})[ \t]*(.*)$/.exec(line);
  if (!m) return null;
  const marker = m[2][0];
  const info = m[3].trim();
  if (marker === "`" && info.includes("`")) return null;
  return { marker, length: m[2].length, sequence: m[2], info, language: info.split(/[ \t]+/, 1)[0] || "" };
}
function isFenceLine(line) { return Boolean(getFenceInfo(line)); }
function isFenceCloseLine(line, opener) {
  const info = getFenceInfo(line);
  return Boolean(info && opener && info.marker === opener.marker && info.length >= opener.length && info.language === "");
}
function isFenceOpenerLine(line) { return Boolean(getFenceInfo(line)); }
function parseSetextHeadingLevel(line) {
  const m = /^\s{0,3}(=+|-+)\s*$/.exec(line);
  if (!m) return null;
  return m[1][0] === "=" ? 1 : 2;
}
function isInsideInlineCode(lineBeforeCursor) { return ((lineBeforeCursor.match(/(?<!\\)`/g) || []).length % 2) === 1; }

function trimTableCellPadding(cell) {
  let value = String(cell ?? "");
  if (value.startsWith(" ")) value = value.slice(1);
  if (value.endsWith(" ")) value = value.slice(0, -1);
  return value;
}
function splitTableRow(line) {
  let row = String(line ?? "").trim();
  if (row.startsWith("|")) row = row.slice(1);
  if (row.endsWith("|")) row = row.slice(0, -1);
  const cells = [];
  let current = "";
  let escaped = false;
  for (const char of row) {
    if (escaped) { current += char; escaped = false; continue; }
    if (char === "\\") { current += char; escaped = true; continue; }
    if (char === "|") { cells.push(trimTableCellPadding(current)); current = ""; continue; }
    current += char;
  }
  cells.push(trimTableCellPadding(current));
  return cells;
}
function isTableDelimiter(line) {
  const t = String(line ?? "").trim();
  if (!t.includes("|")) return false;
  const cells = splitTableRow(t);
  return cells.length >= 2 && cells.every(cell => /^:?-{3,}:?$/.test(cell.trim()));
}
function isLikelyTableRow(line) {
  const t = String(line ?? "").trim();
  return t.includes("|") && splitTableRow(t).length >= 2;
}
function parseTableLineRanges(line, absoluteStart) {
  const raw = String(line ?? "");
  const cells = [];
  let start = 0;
  let end = raw.length;
  if (raw[start] === "|") start += 1;
  if (raw[end - 1] === "|") end -= 1;
  let cellStart = start;
  let escaped = false;
  for (let i = start; i <= end; i += 1) {
    const atEnd = i === end;
    const ch = raw[i];
    if (!atEnd && escaped) { escaped = false; continue; }
    if (!atEnd && ch === "\\") { escaped = true; continue; }
    if (atEnd || ch === "|") {
      const rawCellStart = cellStart;
      const rawCellEnd = i;
      let from = cellStart;
      let to = i;
      if (from < to && raw[from] === " ") from += 1;
      if (to > from && raw[to - 1] === " ") to -= 1;
      if (from === to && rawCellEnd > rawCellStart && raw[rawCellStart] === " ") {
        from = Math.min(rawCellStart + 1, rawCellEnd);
        to = from;
      }
      cells.push({ text: raw.slice(from, to), from: absoluteStart + from, to: absoluteStart + to });
      cellStart = i + 1;
    }
  }
  return cells;
}
function tableAlignmentFromDelimiter(cellText) {
  const value = String(cellText ?? "").trim();
  const left = value.startsWith(":");
  const right = value.endsWith(":");
  if (left && right) return "center";
  if (right) return "right";
  if (left) return "left";
  return "";
}
function tableAlignmentStyle(alignment) {
  return alignment ? ` style="text-align:${alignment}"` : "";
}
function unescapeTableCellText(cellText) {
  return String(cellText ?? "").replace(/\\([\\|])/g, "$1");
}

function classifyLine(value, offset, lineInfo, opts = {}) {
  if (isInsideFence(value, offset)) return { kind: "fenced-code" };
  const list = parseListItem(lineInfo.text, opts); if (list) return { kind: list.kind, list };
  const heading = parseHeading(lineInfo.text); if (heading) return { kind: "heading", heading };
  const quote = parseBlockquote(lineInfo.text); if (quote) return { kind: "blockquote", blockquote: quote };
  if (isHorizontalRule(lineInfo.text)) return { kind: "horizontal-rule" };
  if (usesGfm(opts) && isLikelyTableRow(lineInfo.text)) return { kind: "table" };
  return { kind: "paragraph" };
}
function isInsideFence(value, offset) {
  const source = normalizeLineEndings(value);
  const lines = getLines(source);
  let opener = null;
  for (const line of lines) {
    if (line.start >= offset) break;
    if (!opener) {
      const info = getFenceInfo(line.text);
      if (!info) continue;
      if (offset <= line.end) break;
      opener = info;
      continue;
    }
    if (isFenceCloseLine(line.text, opener)) {
      if (offset <= line.end) break;
      opener = null;
    }
  }
  return Boolean(opener);
}
function hasClosingFenceAfter(value, lineEnd, opener) {
  const rest = normalizeLineEndings(value).slice(lineEnd + 1);
  return getLines(rest).some(line => isFenceCloseLine(line.text, opener));
}

function applyTextChanges(value, changes) {
  const sorted = [...changes].sort((a, b) => b.from - a.from);
  let out = value;
  for (const c of sorted) {
    const from = clamp(c.from, 0, out.length);
    const to = clamp(c.to, from, out.length);
    out = out.slice(0, from) + normalizeLineEndings(c.insert ?? "") + out.slice(to);
  }
  return out;
}
function diffTextChange(before, after) {
  const oldValue = normalizeLineEndings(before ?? "");
  const newValue = normalizeLineEndings(after ?? "");
  if (oldValue === newValue) return [];
  let start = 0;
  const maxStart = Math.min(oldValue.length, newValue.length);
  while (start < maxStart && oldValue[start] === newValue[start]) start += 1;
  let oldEnd = oldValue.length;
  let newEnd = newValue.length;
  while (oldEnd > start && newEnd > start && oldValue[oldEnd - 1] === newValue[newEnd - 1]) {
    oldEnd -= 1;
    newEnd -= 1;
  }
  return [{ from: start, to: oldEnd, insert: newValue.slice(start, newEnd) }];
}
function normalizeChanges(changes = []) {
  return [...changes]
    .map(change => ({
      from: Number(change.from) || 0,
      to: Number(change.to) || Number(change.from) || 0,
      insert: normalizeLineEndings(change.insert ?? ""),
    }))
    .sort((a, b) => a.from - b.from || a.to - b.to);
}
function changedSpans(changes = []) {
  const sorted = normalizeChanges(changes);
  if (!sorted.length) return null;
  let oldStart = Infinity;
  let oldEnd = -Infinity;
  let newStart = Infinity;
  let newEnd = -Infinity;
  let shift = 0;
  for (const change of sorted) {
    oldStart = Math.min(oldStart, change.from);
    oldEnd = Math.max(oldEnd, change.to);
    const newFrom = change.from + shift;
    const newTo = newFrom + change.insert.length;
    newStart = Math.min(newStart, newFrom);
    newEnd = Math.max(newEnd, newTo);
    shift += change.insert.length - (change.to - change.from);
  }
  return { oldStart, oldEnd, newStart, newEnd, delta: shift };
}
function sameSelection(a, b) { return a && b && a.start === b.start && a.end === b.end && (a.direction || "none") === (b.direction || "none"); }
function makeSnapshot(value, selectionStart, selectionEnd, direction = "none") { return { value, selection: { start: selectionStart, end: selectionEnd, direction } }; }
function tx(ctx, actionId, changes, selectionAfter, undoGroup = actionId) {
  return { changes, selectionBefore: { start: ctx.selectionStart, end: ctx.selectionEnd, direction: ctx.selectionDirection ?? "none" }, selectionAfter, source: "api", actionId, undoGroup, timestamp: now() };
}
function ok(transaction, announcement) { return { ok: true, transaction, announcement }; }
function okNoop(announcement, preventDefault = false) { return { ok: true, announcement, preventDefault }; }
function fail(reason, message) { return { ok: false, reason, message }; }
function insertionTransaction(ctx, actionId, insert, selectionOffset = insert.length, undoGroup = actionId) {
  const from = ctx.selectionStart; const to = ctx.selectionEnd; const cursor = from + selectionOffset;
  return ok(tx(ctx, actionId, [{ from, to, insert }], { start: cursor, end: cursor, direction: "none" }, undoGroup));
}
function removePrefixFromLine(ctx, actionId, prefixEndOffset, announcement) {
  const from = ctx.currentLine.start; const to = ctx.currentLine.start + prefixEndOffset;
  return ok(tx(ctx, actionId, [{ from, to, insert: "" }], { start: from, end: from, direction: "none" }, actionId), announcement);
}

function parseBlocks(markdown, opts = {}) {
  const source = normalizeLineEndings(markdown);
  const lines = getLines(source);
  const blocks = [];
  const gfm = usesGfm(opts);
  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    if (gfm && i + 1 < lines.length && isLikelyTableRow(line.text) && isTableDelimiter(lines[i + 1].text)) {
      const header = { ...lines[i], cells: parseTableLineRanges(lines[i].text, lines[i].start) };
      const delimiter = { ...lines[i + 1], cells: parseTableLineRanges(lines[i + 1].text, lines[i + 1].start) };
      const rows = [];
      let j = i + 2;
      while (j < lines.length && lines[j].text.trim() && isLikelyTableRow(lines[j].text)) {
        rows.push({ ...lines[j], cells: parseTableLineRanges(lines[j].text, lines[j].start) });
        j += 1;
      }
      blocks.push({ type: "table", from: line.start, to: (rows.at(-1) ?? delimiter).end, newlineEnd: (rows.at(-1) ?? delimiter).newlineEnd, header, delimiter, rows });
      i = j - 1;
      continue;
    }
    const fenceInfo = getFenceInfo(line.text);
    if (fenceInfo && (line.newlineEnd > line.end || i + 1 < lines.length)) {
      const codeLines = [];
      let j = i + 1;
      while (j < lines.length && !isFenceCloseLine(lines[j].text, fenceInfo)) { codeLines.push(lines[j]); j += 1; }
      const closing = j < lines.length ? lines[j] : null;
      blocks.push({ type: "code-fence", from: line.start, to: (closing ?? codeLines.at(-1) ?? line).end, newlineEnd: (closing ?? codeLines.at(-1) ?? line).newlineEnd, opening: line, closing, codeLines, language: fenceInfo.language, fence: fenceInfo });
      i = closing ? j : j - 1;
      continue;
    }
    const setextLevel = i + 1 < lines.length ? parseSetextHeadingLevel(lines[i + 1].text) : null;
    const canSetext = setextLevel
      && line.text.trim()
      && !parseHeading(line.text)
      && !parseListItem(line.text, opts)
      && !parseBlockquote(line.text)
      && !isHorizontalRule(line.text);
    if (canSetext) {
      blocks.push({ type: "heading", from: line.start, to: lines[i + 1].end, newlineEnd: lines[i + 1].newlineEnd, line, heading: { indent: "", level: setextLevel, markerText: lines[i + 1].text, content: line.text.trim(), contentStart: 0 }, setext: lines[i + 1] });
      i += 1;
      continue;
    }
    const heading = parseHeading(line.text);
    if (heading) { blocks.push({ type: "heading", from: line.start, to: line.end, newlineEnd: line.newlineEnd, line, heading }); continue; }
    const list = parseListItem(line.text, opts);
    if (list) { blocks.push({ type: list.kind, from: line.start, to: line.end, newlineEnd: line.newlineEnd, line, list }); continue; }
    const quote = parseBlockquote(line.text);
    if (quote) { blocks.push({ type: "blockquote", from: line.start, to: line.end, newlineEnd: line.newlineEnd, line, quote }); continue; }
    if (isHorizontalRule(line.text)) { blocks.push({ type: "horizontal-rule", from: line.start, to: line.end, newlineEnd: line.newlineEnd, line }); continue; }
    blocks.push({ type: line.text.trim() ? "paragraph" : "blank", from: line.start, to: line.end, newlineEnd: line.newlineEnd, line });
  }
  if (blocks.length === 0) blocks.push({ type: "blank", from: 0, to: 0, newlineEnd: 0, line: { start: 0, end: 0, newlineEnd: 0, text: "" } });
  return assignHeadingIds(blocks);
}

function decorateInline(raw, opts = {}) {
  const text = String(raw ?? "");
  const tokens = [];
  const reserve = html => {
    const placeholder = `\uE000${tokens.length}\uE001`;
    tokens.push([placeholder, html]);
    return placeholder;
  };
  const token = t => `<span class="md-token">${escapeHtml(t)}</span>`;
  let prepared = "";
  for (let i = 0; i < text.length; i += 1) {
    if (text[i] === "`") {
      const code = parseCodeSpanAt(text, i);
      if (code) {
        prepared += reserve(`${token(code.marker)}<code>${escapeHtml(code.content)}</code>${token(code.marker)}`);
        i = code.to - 1;
        continue;
      }
    }
    if (text[i] === "\\" && isEscapablePunctuation(text[i + 1])) {
      prepared += reserve(token("\\") + escapeHtml(text[i + 1]));
      i += 2;
      i -= 1;
      continue;
    }
    const bareLink = parseBareLinkAt(text, i);
    if (bareLink) {
      const safe = safeHref(bareLink.url);
      const rendered = `${token("[")}<a href="${escapeAttribute(safe)}" tabindex="-1">${escapeHtml(bareLink.url)}</a>${token("]")}`;
      prepared += reserve(rendered);
      i = bareLink.to - 1;
      continue;
    }
    const link = parseInlineLinkAt(text, i)
      || parseReferenceLinkAt(text, opts.references, i);
    if (link) {
      const safe = safeHref(link.url);
      // Empty labels must render as "" (not the <br> that decorateInline uses for
      // empty lines), otherwise `![]()`/`[]()` visually breaks across two lines.
      const labelHtml = link.label ? decorateInline(link.label, opts) : "";
      const prefix = token(`${link.bang}[`);
      const inline = text[link.labelEnd + 1] === "(";
      const suffix = inline
        ? `${token("](")}<span class="md-url">${escapeHtml(link.url)}</span>${token(")")}`
        : token(text.slice(link.labelEnd, link.to));
      const rendered = link.bang
        ? `${prefix}${labelHtml}${suffix}`
        : `${prefix}<a href="${escapeAttribute(safe)}" tabindex="-1">${labelHtml}</a>${suffix}`;
      prepared += reserve(rendered);
      i = link.to - 1;
      continue;
    }
    prepared += text[i];
  }
  const html = renderNonographEmphasis(prepared, token);
  let restored = html;
  for (const [placeholder, reserved] of tokens) {
    restored = restored.replaceAll(escapeHtml(placeholder), reserved).replaceAll(placeholder, reserved);
  }
  return restored || "<br>";
}

function renderInlineMarkdown(source, opts = {}) {
  // Preview renderer: sanitize by construction. Unlike decorateInline, markdown delimiters are not retained.
  let text = String(source ?? "");
  const tokens = [];
  const reserve = html => { const token = `\uE000${tokens.length}\uE001`; tokens.push([token, html]); return token; };
  let codeReserved = "";
  for (let i = 0; i < text.length; i += 1) {
    const code = parseCodeSpanAt(text, i);
    if (!code) { codeReserved += text[i]; continue; }
    codeReserved += reserve(`<code>${escapeHtml(normalizeCodeSpanContent(code.content))}</code>`);
    i = code.to - 1;
  }
  text = codeReserved;
  let escapesReserved = "";
  for (let i = 0; i < text.length; i += 1) {
    if (text[i] === "\\" && isEscapablePunctuation(text[i + 1])) {
      escapesReserved += reserve(escapeHtml(text[i + 1]));
      i += 1;
    } else escapesReserved += text[i];
  }
  text = escapesReserved;
  let linked = "";
  for (let i = 0; i < text.length; i += 1) {
    const bareLink = parseBareLinkAt(text, i);
    if (bareLink) {
      const safe = safeHref(bareLink.url);
      const target = opts.linkTarget === "_blank" ? " target=\"_blank\" rel=\"noopener noreferrer\"" : "";
      linked += (safe === "#")
        ? bareLink.full
        : reserve(`<a href="${escapeAttribute(safe)}"${target}>${escapeHtml(bareLink.url)}</a>`);
      i = bareLink.to - 1;
      continue;
    }
    const link = parseInlineLinkAt(text, i)
      || parseReferenceLinkAt(text, opts.references, i);
    if (!link) { linked += text[i]; continue; }
    if (link.bang) {
      const safe = safeHref(link.url, { allowDataImage: false });
      linked += (safe === "#" && String(link.url).trim() !== "#")
        ? link.full
        : reserve(`<img src="${escapeAttribute(safe)}" alt="${escapeAttribute(link.label)}"${link.title ? ` title="${escapeAttribute(link.title)}"` : ""}>`);
    } else {
      const safe = safeHref(link.url);
      const target = opts.linkTarget === "_blank" ? " target=\"_blank\" rel=\"noopener noreferrer\"" : "";
      linked += (safe === "#" && String(link.url).trim() !== "#")
        ? link.label
        : reserve(`<a href="${escapeAttribute(safe)}"${target}${link.title ? ` title="${escapeAttribute(link.title)}"` : ""}>${renderInlineMarkdown(link.label, opts)}</a>`);
    }
    i = link.to - 1;
  }
  text = linked;
  text = text.replace(/(^|[\s(])((?:https?:\/\/)[^\s<]+[^\s<.,;:!?\])}])/g, (match, prefix, url) =>
    `${prefix}${reserve(`<a href="${escapeAttribute(safeHref(url))}">${escapeHtml(url)}</a>`)}`);
  text = text.replace(/(?: {2,}|\\)\n/g, () => `${reserve("<br>")}\n`);
  text = renderNonographEmphasis(text);
  for (const [token, html] of tokens) text = text.replaceAll(escapeHtml(token), html).replaceAll(token, html);
  return text;
}

function isListBlock(block) {
  return ["bullet-list-item", "ordered-list-item", "task-list-item"].includes(block?.type);
}

function listIndentWidth(block) {
  return String(block?.list?.indent ?? "").replace(/\t/g, "    ").length;
}

function lineIndentWidth(line) {
  return (/^[ \t]*/.exec(String(line?.text ?? ""))?.[0] ?? "").replace(/\t/g, "    ").length;
}

function stripContinuationIndent(text, width) {
  let remaining = Math.max(0, width);
  let index = 0;
  const source = String(text ?? "");
  while (index < source.length && remaining > 0) {
    if (source[index] === " ") { index += 1; remaining -= 1; continue; }
    if (source[index] === "\t") { index += 1; remaining = Math.max(0, remaining - 4); continue; }
    break;
  }
  return source.slice(index);
}

function renderListFromBlocks(blocks, start, options) {
  const first = blocks[start];
  const baseIndent = listIndentWidth(first);
  const listType = first.list.listType;
  const sameLevel = block =>
    isListBlock(block)
    && block.list.listType === listType
    && listIndentWidth(block) === baseIndent;
  const items = [];
  let loose = false;
  let index = start;
  let stopList = false;

  while (sameLevel(blocks[index])) {
    const block = blocks[index];
    const contentIndent = baseIndent + block.list.contentStart - block.list.indent.length;
    const item = {
      block,
      paragraphs: [[block.list.content]],
      children: [],
    };
    index += 1;
    let pendingBlank = false;

    while (index < blocks.length) {
      const next = blocks[index];
      if (next.type === "blank") {
        const after = blocks[index + 1];
        if (sameLevel(after)) {
          loose = true;
          index += 1;
          break;
        }
        if (isListBlock(after) && listIndentWidth(after) > baseIndent) {
          loose = true;
          pendingBlank = true;
          index += 1;
          continue;
        }
        if (after?.type === "paragraph" && lineIndentWidth(after.line) >= contentIndent) {
          loose = true;
          pendingBlank = true;
          index += 1;
          continue;
        }
        stopList = true;
        break;
      }
      if (sameLevel(next)) break;
      if (isListBlock(next)) {
        if (listIndentWidth(next) <= baseIndent) {
          stopList = true;
          break;
        }
        const child = renderListFromBlocks(blocks, index, options);
        item.children.push(child.html);
        index = child.nextIndex;
        pendingBlank = false;
        continue;
      }
      if (next.type === "paragraph") {
        if (pendingBlank) {
          if (lineIndentWidth(next.line) < contentIndent) {
            stopList = true;
            break;
          }
          item.paragraphs.push([stripContinuationIndent(next.line.text, contentIndent)]);
          pendingBlank = false;
        } else {
          item.paragraphs.at(-1).push(stripContinuationIndent(next.line.text, contentIndent));
        }
        index += 1;
        continue;
      }
      stopList = true;
      break;
    }
    items.push(item);
    if (stopList || !sameLevel(blocks[index])) break;
  }

  const tag = listType === "ol" ? "ol" : "ul";
  const startNumber = first.list.number;
  const startAttribute = tag === "ol" && Number.isFinite(startNumber) && startNumber !== 1
    ? ` start="${startNumber}"`
    : "";
  const html = items.map(item => {
    const checkbox = item.block.list.kind === "task-list-item"
      ? `<input type="checkbox" disabled${item.block.list.checked ? " checked" : ""}> `
      : "";
    const paragraphs = item.paragraphs.map((lines, paragraphIndex) => {
      const prefix = paragraphIndex === 0 ? checkbox : "";
      const content = `${prefix}${renderInlineMarkdown(lines.join("\n"), options)}`;
      return loose || paragraphIndex > 0 ? `<p>${content}</p>` : content;
    }).join("");
    const children = item.children.length ? `\n${item.children.join("\n")}\n` : "";
    return `<li>${paragraphs}${children}</li>`;
  }).join("");
  return { html: `<${tag}${startAttribute}>${html}</${tag}>`, nextIndex: index };
}

function renderMarkdown(markdown, opts = {}) {
  const extracted = extractReferenceDefinitions(markdown, opts.references);
  const options = { ...DEFAULTS, ...opts, references: extracted.references };
  const blocks = parseBlocks(extracted.markdown, options);
  const out = [];
  for (let i = 0; i < blocks.length; i += 1) {
    const block = blocks[i];
    if (block.type === "blank") continue;
    if (block.type === "heading") { out.push(`<h${block.heading.level} id="${escapeAttribute(block.heading.id)}">${renderInlineMarkdown(block.heading.content, options)}</h${block.heading.level}>`); continue; }
    if (block.type === "horizontal-rule") { out.push("<hr>"); continue; }
    if (block.type === "blockquote") {
      const quotes = [block];
      let j = i + 1;
      while (j < blocks.length && blocks[j].type === "blockquote") { quotes.push(blocks[j]); j += 1; }
      const body = quotes.map(q => q.quote.content).join("\n");
      out.push(`<blockquote>${renderMarkdown(body, options)}</blockquote>`);
      i = j - 1;
      continue;
    }
    if (isListBlock(block)) {
      const rendered = renderListFromBlocks(blocks, i, options);
      out.push(rendered.html);
      i = rendered.nextIndex - 1;
      continue;
    }
    if (block.type === "code-fence") {
      const lang = block.language ? ` class="language-${escapeAttribute(block.language)}"` : "";
      out.push(`<pre><code${lang}>${escapeHtml(block.codeLines.map(l => l.text).join("\n"))}</code></pre>`); continue;
    }
    if (block.type === "table") {
      const header = block.header.cells;
      const rows = block.rows;
      const alignments = block.delimiter.cells.map(cell => tableAlignmentFromDelimiter(cell.text));
      out.push(`<div class="md-table-wrap"><table><thead><tr>${header.map((c, i) => `<th${tableAlignmentStyle(alignments[i])}>${renderInlineMarkdown(unescapeTableCellText(c.text), options)}</th>`).join("")}</tr></thead><tbody>${rows.map(r => `<tr>${header.map((_, i) => `<td${tableAlignmentStyle(alignments[i])}>${renderInlineMarkdown(unescapeTableCellText(r.cells[i]?.text ?? ""), options)}</td>`).join("")}</tr>`).join("")}</tbody></table></div>`); continue;
    }
    if (block.type === "paragraph") {
      const paragraphs = [block];
      let j = i + 1;
      while (j < blocks.length && blocks[j].type === "paragraph") { paragraphs.push(blocks[j]); j += 1; }
      out.push(`<p>${renderInlineMarkdown(paragraphs.map(p => p.line.text).join("\n"), options)}</p>`);
      i = j - 1;
      continue;
    }
    out.push(`<p>${renderInlineMarkdown(block.line.text, options)}</p>`);
  }
  return out.join("\n");
}
function textFromMarkdown(markdown, opts = {}) {
  const extracted = extractReferenceDefinitions(markdown, opts.references);
  const options = { ...DEFAULTS, ...opts, references: extracted.references };
  const inlineText = text => stripHtml(renderInlineMarkdown(text, options));
  const lines = [];
  for (const block of parseBlocks(extracted.markdown, options)) {
    if (block.type === "blank") { lines.push(""); continue; }
    if (block.type === "heading") { lines.push(inlineText(block.heading.content)); continue; }
    if (block.type === "horizontal-rule") continue;
    if (block.type === "blockquote") { lines.push(inlineText(block.quote.content)); continue; }
    if (block.type === "bullet-list-item" || block.type === "ordered-list-item" || block.type === "task-list-item") { lines.push(inlineText(block.list.content)); continue; }
    if (block.type === "code-fence") { lines.push(block.codeLines.map(l => l.text).join("\n")); continue; }
    if (block.type === "table") {
      const tableRows = [block.header, ...block.rows];
      for (const row of tableRows) lines.push(row.cells.map(cell => inlineText(unescapeTableCellText(cell.text))).join("\t"));
      continue;
    }
    lines.push(inlineText(block.line.text));
  }
  return lines.join("\n").replace(/\n{3,}/g, "\n\n").trim();
}

class WritemarkEditorElement extends HTMLElement {
  static formAssociated = true;
  static get observedAttributes() { return REFLECTED_ATTRIBUTES; }

  constructor() {
    super();
    this._internals = this.attachInternals?.() ?? null;
    this._shadow = this.attachShadow({ mode: "open", delegatesFocus: true });
    this._value = "";
    this._defaultValue = "";
    this._selection = { start: 0, end: 0, direction: "none" };
    this._dirty = false;
    this._formDisabled = false;
    this._hasConnected = false;
    this._isComposing = false;
    this._beforeInputSnapshot = null;
    this._beforeInputTarget = null;
    this._liveSelectionAPI = null;
    this._fallbackEditable = null;
    this._fallbackSelectionPending = false;
    this._undoStack = [];
    this._redoStack = [];
    this._maxUndo = 300;
    this._selectAllLevel = 0;
    this._structuredSelection = null;
    this._ignoreSelectionChangeCount = 0;
    this._pointerSelection = null;
    this._suppressLiveClick = false;
    this._focusWithin = false;
    this._blockCacheValue = null;
    this._blockCache = null;
    this._liveBlocks = [];
    this._liveEditablesCache = [];
    this._liveNavigationCache = [];
    this._liveIndexDirty = true;
    this._liveDirty = true;
    this._validationVisible = false;
    this._completionUpdateFrame = 0;
    this._virtualState = { active: false, start: 0, end: 0, total: 0, lineHeight: 24 };
    this._virtualScrollFrame = 0;
    this._actions = new Map();
    this._providers = new Map();
    this._completion = { open: false, providerId: null, match: null, items: [], activeIndex: 0, requestId: 0, abort: null };
    this._ids = { label: uid("mfe-label"), source: uid("mfe-source"), live: uid("mfe-live"), completion: uid("mfe-completion"), status: uid("mfe-status"), validation: uid("mfe-validation"), toolbar: uid("mfe-toolbar") };
    // Buttons shown in the floating selection toolbar (bubble menu), in order.
    // Icons are inline SVGs styled white via currentColor.
    const svg = inner => `<svg viewBox="0 0 24 24" aria-hidden="true">${inner}</svg>`;
    this._selectionToolbarActions = [
      { actionId: "inline.bold", label: "Bold", html: svg('<path d="M7 5h6a3.5 3.5 0 0 1 0 7H7z"/><path d="M7 12h7a3.5 3.5 0 0 1 0 7H7z"/>') },
      { actionId: "inline.italic", label: "Italic", html: svg('<line x1="10" y1="5" x2="18" y2="5"/><line x1="6" y1="19" x2="14" y2="19"/><line x1="14" y1="5" x2="10" y2="19"/>') },
      { actionId: "inline.underline", label: "Underline", html: svg('<path d="M7 4v6a5 5 0 0 0 10 0V4"/><line x1="5" y1="20" x2="19" y2="20"/>') },
      { actionId: "inline.strikethrough", label: "Strikethrough", html: svg('<path d="M18 6.5c-1.2-2.2-3.4-3-5.8-3C9 3.5 6.5 5.2 6.5 7.8c0 5.4 11 2.7 11 8.2 0 2.5-2.4 4.2-5.5 4.2-2.5 0-4.9-1-6.2-3.2"/><line x1="3" y1="12" x2="21" y2="12"/>') },
      { actionId: "inline.highlight", label: "Highlight", html: svg('<path d="M15 4l5 5-9 9-5 1 1-5z"/><line x1="4" y1="21" x2="20" y2="21"/>') },
      { actionId: "inline.code", label: "Inline code", html: svg('<polyline points="8 6 2 12 8 18"/><polyline points="16 6 22 12 16 18"/>') },
      { actionId: "inline.link", label: "Link", html: svg('<path d="M10 14a4 4 0 0 0 5.66 0l3-3a4 4 0 1 0-5.66-5.66l-1.5 1.5"/><path d="M14 10a4 4 0 0 0-5.66 0l-3 3a4 4 0 1 0 5.66 5.66l1.5-1.5"/>') },
    ];
    this._installBuiltInActions();
    this._installBuiltInProviders();
  }

  connectedCallback() {
    if (!this._hasConnected) {
      this._upgradeProperties();
      this._renderShell();
      this._bindEvents();
      this._hasConnected = true;
      const initial = this.getAttribute("value") ?? this._value ?? "";
      this._defaultValue = normalizeLineEndings(initial);
      this._setValueInternal(this._defaultValue, { source: "init", silent: true, recordUndo: false, preserveSelection: false });
      this._syncAttributesToControls();
      this._renderAll({ restoreSelection: false });
      this._updateFormValue();
      this._updateValidity();
    } else {
      this._syncAttributesToControls();
      this._renderAll({ restoreSelection: true });
      this._updateFormValue();
      this._updateValidity();
    }
  }
  disconnectedCallback() {
    this._completion.abort?.abort();
    if (this._completionUpdateFrame) cancelAnimationFrame(this._completionUpdateFrame);
    if (this._virtualScrollFrame) cancelAnimationFrame(this._virtualScrollFrame);
    this._completionUpdateFrame = 0;
    this._virtualScrollFrame = 0;
  }
  attributeChangedCallback(name, oldValue, newValue) {
    if (oldValue === newValue) return;
    if (name === "value") {
      const nextDefault = normalizeLineEndings(newValue ?? "");
      if (!this._hasConnected) { this._value = nextDefault; this._defaultValue = nextDefault; return; }
      const wasDirty = this._dirty;
      this._defaultValue = nextDefault;
      if (!wasDirty) this._setValueInternal(nextDefault, { source: "attribute", silent: true, recordUndo: false, preserveSelection: false });
      else {
        const oldDirty = this._dirty;
        this._dirty = this._value !== this._defaultValue;
        if (oldDirty !== this._dirty) this._dispatch("md-dirty-change", { dirty: this._dirty });
      }
      return;
    }
    if (!this._hasConnected) return;
    const restoreFocus = (name === "mode" || name === "readonly") && (this._focusWithin || Boolean(this._shadow.activeElement));
    if ((name === "disabled" || name === "readonly") && (this.disabled || this.readonly)) this._closeCompletion();
    this._syncAttributesToControls();
    if (name === "disabled" && this.disabled) this.blur();
    if (["mode", "disabled", "readonly"].includes(name)) this._renderAll({ restoreSelection: !this.disabled, force: true });
    if (["required", "disabled", "readonly", "maxlength", "minlength"].includes(name)) { this._updateFormValue(); this._updateValidity(); }
    if (restoreFocus && !this.disabled) this._focusEditable();
  }

  get value() { return this._value; }
  set value(next) { this._setValueInternal(next, { source: "api", silent: false, recordUndo: false }); }
  get defaultValue() { return this._defaultValue; }
  set defaultValue(next) { this._defaultValue = normalizeLineEndings(next ?? ""); this.setAttribute("value", this._defaultValue); }
  get name() { return this.getAttribute("name") ?? ""; }
  set name(v) { v == null ? this.removeAttribute("name") : this.setAttribute("name", String(v)); }
  get label() { return this.getAttribute("label") ?? ""; }
  set label(v) { v == null ? this.removeAttribute("label") : this.setAttribute("label", String(v)); }
  get placeholder() { return this.getAttribute("placeholder") ?? DEFAULTS.placeholder; }
  set placeholder(v) { v == null ? this.removeAttribute("placeholder") : this.setAttribute("placeholder", String(v)); }
  get mode() { const v = this.getAttribute("mode") ?? DEFAULTS.mode; return ["live", "source"].includes(v) ? v : DEFAULTS.mode; }
  set mode(v) { v == null ? this.removeAttribute("mode") : this.setAttribute("mode", String(v)); }
  get markdownFlavor() { const v = this.getAttribute("markdown-flavor") ?? DEFAULTS.markdownFlavor; return ["gfm", "commonmark"].includes(v) ? v : DEFAULTS.markdownFlavor; }
  set markdownFlavor(v) { v == null ? this.removeAttribute("markdown-flavor") : this.setAttribute("markdown-flavor", String(v)); }
  get tabBehavior() { const v = this.getAttribute("tab-behavior") ?? DEFAULTS.tabBehavior; return ["accessibility-first", "editor-first"].includes(v) ? v : DEFAULTS.tabBehavior; }
  set tabBehavior(v) { v == null ? this.removeAttribute("tab-behavior") : this.setAttribute("tab-behavior", String(v)); }
  get indentString() { return normalizeIndentAttribute(this.getAttribute("indent-string") ?? DEFAULTS.indentString); }
  set indentString(v) { this.setAttribute("indent-string", v === "\t" ? "tab" : String(v)); }
  get disabled() { return this.hasAttribute("disabled") || this._formDisabled; }
  set disabled(v) { this.toggleAttribute("disabled", Boolean(v)); }
  get readonly() { return this.hasAttribute("readonly"); }
  set readonly(v) { this.toggleAttribute("readonly", Boolean(v)); }
  get required() { return this.hasAttribute("required"); }
  set required(v) { this.toggleAttribute("required", Boolean(v)); }
  get dirty() { return this._dirty; }
  get selectionStart() { return this._getCurrentSelection().start; }
  set selectionStart(v) { this.setSelectionRange(v, this.selectionEnd); }
  get selectionEnd() { return this._getCurrentSelection().end; }
  set selectionEnd(v) { this.setSelectionRange(this.selectionStart, v); }
  get validationMessage() { return this._internals?.validationMessage || this._validationMessage || ""; }
  get validity() { return this._internals?.validity ?? this._fallbackValidity(); }
  get willValidate() { return this._internals?.willValidate ?? !this.disabled; }

  focus(options) { this._focusEditable(options); }
  blur() { this._focusWithin = false; this._shadow?.activeElement?.blur?.(); this._sourceTextarea?.blur(); this._liveEditor?.blur(); this._hideSelectionToolbar?.(); }
  select() { this.setSelectionRange(0, this._value.length); }
  setSelectionRange(start, end, direction = "none") {
    const s = clamp(Number(start) || 0, 0, this._value.length);
    const e = clamp(Number(end) || 0, 0, this._value.length);
    this._selection = { start: s, end: e, direction };
    this._structuredSelection = (!this._isSourceActive() && s !== e) ? { start: s, end: e, direction, label: "selection" } : null;
    if (this._sourceTextarea && this._isSourceActive()) this._sourceTextarea.setSelectionRange(s, e, direction);
    if (this._liveEditor && !this._isSourceActive()) { this._ignoreSelectionChangeCount = 2; this._restoreLiveSelection(this._selection); }
    this._emitSelectionChange();
    this._scheduleCompletionUpdate();
  }
  exec(actionId, args) { const result = this._runAction(actionId, args, { source: "api", apply: true }); return Boolean(result?.ok); }
  registerAction(action) {
    if (!action || typeof action.id !== "string" || typeof action.run !== "function") throw new TypeError("registerAction(action) requires an action with string id and run(ctx,args).");
    this._actions.set(action.id, { group: "Custom", visibleInSlash: false, aliases: [], keywords: [], ...action });
  }
  unregisterAction(actionId) { this._actions.delete(actionId); }
  registerCompletionProvider(provider) {
    if (!provider || typeof provider.id !== "string" || typeof provider.match !== "function" || typeof provider.getItems !== "function" || typeof provider.apply !== "function") throw new TypeError("Completion provider requires id, match, getItems, apply.");
    this._providers.set(provider.id, { priority: 0, triggers: [], ...provider });
  }
  unregisterCompletionProvider(providerId) { this._providers.delete(providerId); if (this._completion.providerId === providerId) this._closeCompletion(); }
  getHTML() { return renderMarkdown(this._value, this._rendererOptions()); }
  getText() { return textFromMarkdown(this._value, this._rendererOptions()); }
  getMarkdown() { return this._value; }
  setMarkdown(markdown) { this.value = markdown; }
  getPlainText() { return this.getText(); }
  getSelectionMarkdown() { const sel = this._getCurrentSelection(); return this._value.slice(Math.min(sel.start, sel.end), Math.max(sel.start, sel.end)); }
  insertMarkdown(markdown) { return this.exec("editor.insertText", { text: markdown }); }
  canExec(actionId, args) { const action = this._actions.get(actionId); if (!action) return false; const ctx = this._getContext(); if (ctx.mode === "disabled" && !action.viewSafe) return false; if (ctx.mode === "readonly" && !action.readonlySafe && !action.viewSafe) return false; return !action.when || action.when(ctx, args); }
  getCurrentBlock() { const sel = this._getCurrentSelection(); return this._findBlockAtOffset(sel.start) || null; }
  getSelectedBlocks() { const sel = this._getCurrentSelection(); const start = Math.min(sel.start, sel.end); const end = Math.max(sel.start, sel.end); return this._getBlocks().filter(block => block.to >= start && block.from <= end); }
  getActiveMarks() { return this._getActiveStateIds(this._getContext()); }
  find(query, options = {}) { return this._findText(query, options); }
  replace(query, replacement, options = {}) { return this._replaceText(query, replacement, { ...options, all: false }); }
  replaceAll(query, replacement, options = {}) { return this._replaceText(query, replacement, { ...options, all: true }); }
  commit() { const old = this._dirty; this._defaultValue = this._value; this._dirty = false; this._dispatch("md-change", { value: this._value }); if (old) this._dispatch("md-dirty-change", { dirty: false }); }
  reset() { this._setValueInternal(this._defaultValue, { source: "api", recordUndo: true }); this.setSelectionRange(0, 0); this._dirty = false; this._dispatch("md-dirty-change", { dirty: false }); }
  checkValidity() { this._updateValidity(); return this._internals ? this._internals.checkValidity() : this._fallbackValidity().valid; }
  reportValidity() { this._validationVisible = true; this._updateValidity(); return this._internals ? this._internals.reportValidity() : this.checkValidity(); }
  setCustomValidity(message) { this._customValidityMessage = String(message ?? ""); this._updateValidity(); }

  _upgradeProperties() {
    for (const prop of ["value", "defaultValue", "name", "label", "placeholder", "mode", "markdownFlavor", "tabBehavior", "indentString", "disabled", "readonly", "required"]) {
      if (Object.prototype.hasOwnProperty.call(this, prop)) { const value = this[prop]; delete this[prop]; this[prop] = value; }
    }
  }

  _renderShell() {
    this._shadow.innerHTML = `
      <style>
        :host {
          color-scheme: light dark;
          --md-editor-font: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
          --md-editor-mono-font: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
          --md-editor-font-size: 15px;
          --md-editor-line-height: 1.55;
          --md-editor-bg: Canvas;
          --md-editor-fg: CanvasText;
          --md-editor-muted: color-mix(in srgb, CanvasText 55%, Canvas 45%);
          --md-editor-token: color-mix(in srgb, CanvasText 42%, Canvas 58%);
          --md-editor-border: color-mix(in srgb, CanvasText 22%, Canvas 78%);
          --md-editor-border-focus: Highlight;
          --md-editor-radius: 10px;
          --md-editor-padding: 14px;
          --md-editor-min-height: 220px;
          --md-editor-max-height: none;
          --md-editor-focus-ring: 0 0 0 3px color-mix(in srgb, Highlight 32%, transparent);
          --md-editor-active-line-ring: none;
          --md-editor-active-line-bg: transparent;
          --md-editor-active-cell-ring: var(--md-editor-active-line-ring);
          --md-editor-active-cell-bg: var(--md-editor-active-line-bg);
          --md-editor-popup-bg: Canvas;
          --md-editor-popup-fg: CanvasText;
          --md-editor-popup-border: color-mix(in srgb, CanvasText 24%, Canvas 76%);
          --md-editor-popup-shadow: 0 12px 30px rgb(0 0 0 / 0.16);
          --md-editor-code-bg: color-mix(in srgb, CanvasText 8%, Canvas 92%);
          --md-editor-code-header-bg: color-mix(in srgb, CanvasText 5%, Canvas 95%);
          --md-editor-code-accent: color-mix(in srgb, CanvasText 45%, Canvas 55%);
          --md-editor-danger: #b00020;
          --md-editor-transition-duration: 140ms;
          --md-editor-transition-ease: cubic-bezier(.2,.8,.2,1);
          display: block;
          font-family: var(--md-editor-font);
          color: var(--md-editor-fg);
        }
        :host([hidden]) { display: none; }
        .container { display: grid; gap: 8px; font-size: var(--md-editor-font-size); }
        .label:empty { display: none; }
        .label { font-weight: 650; color: var(--md-editor-fg); }
        .workspace { display: grid; gap: 10px; }
        .editor-shell { position: relative; min-width: 0; }
        .live-editor, textarea {
          box-sizing: border-box; inline-size: 100%; min-block-size: var(--md-editor-min-height); max-block-size: var(--md-editor-max-height);
          border: 1px solid var(--md-editor-border); border-radius: var(--md-editor-radius); padding: var(--md-editor-padding);
          background: var(--md-editor-bg); color: var(--md-editor-fg); line-height: var(--md-editor-line-height); overflow: auto;
          transition: border-color var(--md-editor-transition-duration) var(--md-editor-transition-ease), box-shadow var(--md-editor-transition-duration) var(--md-editor-transition-ease), background-color var(--md-editor-transition-duration) var(--md-editor-transition-ease);
        }
        /* The live writing surface is a bare canvas: no frame, ever. */
        .live-editor, .live-editor:focus, .live-editor:focus-within, .live-editor:hover { border: 0; outline: none; box-shadow: none; }
        textarea:focus { outline: none; border-color: var(--md-editor-border-focus); box-shadow: var(--md-editor-focus-ring); }
        .live-editor[aria-disabled="true"], textarea:disabled { opacity: 0.62; cursor: not-allowed; }
        textarea { display: none; resize: vertical; font-family: var(--md-editor-mono-font); font-size: var(--md-editor-font-size); tab-size: 2; }
        :host([mode="source"]) .live-editor { display: none; }
        :host([mode="source"]) textarea { display: block; }
        .live-placeholder { color: var(--md-editor-placeholder, var(--md-editor-muted)); pointer-events: none; }
        .md-empty-placeholder::before { content: attr(data-placeholder); position: absolute; color: var(--md-editor-placeholder, var(--md-editor-muted)); pointer-events: none; }
        .md-virtual-spacer { display: block; pointer-events: none; user-select: none; inline-size: 1px; min-block-size: 0; }
        .md-line { position: relative; min-block-size: calc(var(--md-editor-line-height, 1.55) * 1em); line-height: var(--md-editor-line-height, 1.55); white-space: pre-wrap; overflow-wrap: anywhere; border-radius: 6px; padding: 0; outline: none; transition: background-color var(--md-editor-transition-duration) var(--md-editor-transition-ease), box-shadow var(--md-editor-transition-duration) var(--md-editor-transition-ease); }
        .md-line:focus, .md-task-source:focus, .md-code-line:focus { box-shadow: var(--md-editor-active-line-ring); background: var(--md-editor-active-line-bg); }
        .md-cell:focus { box-shadow: var(--md-editor-active-cell-ring); background: var(--md-editor-active-cell-bg); }
        /* Blank source lines stand in for the paragraph gap in the finished
           output, so give them the same block size as a rendered paragraph
           margin rather than a full text line. */
        .md-line[data-kind="blank"] { min-block-size: var(--md-editor-paragraph-gap, 1.1em); }
        .md-token { color: var(--md-editor-token); font-weight: 500; }
        .md-url { color: var(--md-editor-muted); text-decoration: underline; }
        .md-heading { font-family: var(--md-editor-font); font-weight: 760; line-height: 1.18; margin-block: 0.22em; }
        .md-h1 { font-size: 2.0em; }
        .md-h2 { font-size: 1.6em; }
        .md-h3 { font-size: 1.35em; }
        .md-h4 { font-size: 1.18em; }
        .md-list { padding-inline-start: calc(var(--md-list-depth, 0) * 1.4em + 2px); }
        .md-task-line { display: flex; align-items: baseline; gap: 0.35em; }
        .md-task-line input { transform: translateY(0.12em); }
        .md-task-source { flex: 1; min-width: 0; white-space: pre-wrap; outline: none; border-radius: 6px; }
        .md-quote {
          --md-quote-depth: 1;
          border-radius: 0;
          padding-inline-start: calc(var(--md-quote-depth) * 1em + 0.35em);
          color: color-mix(in srgb, CanvasText 80%, Canvas 20%);
          background-image: repeating-linear-gradient(to right, var(--md-editor-border) 0 4px, transparent 4px 1em);
          background-position: left top;
          background-repeat: no-repeat;
          background-size: calc(var(--md-quote-depth) * 1em) 100%;
        }
        .md-quote:dir(rtl) {
          background-image: repeating-linear-gradient(to left, var(--md-editor-border) 0 4px, transparent 4px 1em);
          background-position: right top;
        }
        .md-quote-depth-2 { --md-quote-depth: 2; }
        .md-quote-depth-3 { --md-quote-depth: 3; }
        .md-quote-depth-4 { --md-quote-depth: 4; }
        .md-quote-depth-5 { --md-quote-depth: 5; }
        .md-quote-depth-6 { --md-quote-depth: 6; }
        .md-quote-depth-7 { --md-quote-depth: 7; }
        .md-quote-depth-8 { --md-quote-depth: 8; }
        .md-quote-depth-9 { --md-quote-depth: 9; }
        .md-quote-depth-10 { --md-quote-depth: 10; }
        .md-quote-depth-11 { --md-quote-depth: 11; }
        .md-quote-depth-12 { --md-quote-depth: 12; }
        .md-quote-depth-13 { --md-quote-depth: 13; }
        .md-quote-depth-14 { --md-quote-depth: 14; }
        .md-quote-depth-15 { --md-quote-depth: 15; }
        .md-quote-depth-16 { --md-quote-depth: 16; }
        .md-quote + .md-quote { margin-block-start: 0; }
        .md-hr-line { display: block; min-block-size: 1.35em; color: var(--md-editor-token); cursor: text; }
        .md-hr-line::after { content: ""; display: block; border-block-start: 1px solid var(--md-editor-border); margin-block-start: 0.3em; }
        /* Show the raw divider markers (e.g. ***) so the line stays editable. */
        .md-hr-line .md-token { display: inline; }
        /* Code blocks mirror the published post output (post.html): flat panel
           with a header strip showing the language, monospace body at 14px/1.4.
           No syntax highlighting and no copy/collapse controls in the editor. */
        .md-code-block { margin-block: 20px; border: 1px solid var(--md-editor-border); border-radius: var(--md-editor-radius); background: var(--md-editor-code-bg); overflow: hidden; }
        .md-code-header { display: flex; align-items: center; justify-content: space-between; gap: 8px; min-block-size: 22px; padding: 1px 6px; border-block-end: 1px solid var(--md-editor-border); background: var(--md-editor-code-header-bg); color: var(--md-editor-code-accent); font-family: var(--md-editor-ui-font, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif); font-size: 12px; line-height: 1.4; user-select: none; }
        .md-code-label { position: relative; text-transform: uppercase; font-weight: 500; letter-spacing: 0.5px; outline: none; min-inline-size: 1ch; }
        .md-code-language { display: none; }
        .md-code-lines { padding: 6px 0 6px 12px; font-family: var(--md-editor-mono-font); font-size: 14px; line-height: 1.4; color: var(--md-editor-fg); white-space: pre; overflow-x: auto; }
        .md-code-line { min-height: 1.4em; line-height: 1.4; outline: none; white-space: pre; border-radius: 0; }
        .md-code-line:empty::before { content: "\\200b"; }
        .md-code-fence { display: none; }
        .md-table-block { overflow: auto; margin-block: 0.5em; }
        .md-table { border-collapse: collapse; inline-size: 100%; table-layout: fixed; }
        .md-table th, .md-table td { border: 1px solid var(--md-editor-border); padding: 6px 8px; vertical-align: top; }
        .md-table th { background: color-mix(in srgb, CanvasText 7%, Canvas 93%); font-weight: 700; text-align: left; }
        .md-cell { min-height: 1.35em; outline: none; white-space: pre-wrap; overflow-wrap: anywhere; }
        .completion-popup { position: absolute; z-index: 20; min-inline-size: 240px; max-inline-size: min(420px, 90vw); max-block-size: min(320px, 50vh); overflow: auto; border: 1px solid var(--md-editor-popup-border); border-radius: var(--md-editor-radius); background: var(--md-editor-popup-bg); color: var(--md-editor-popup-fg); box-shadow: var(--md-editor-popup-shadow); padding: 4px; }
        .completion-popup[hidden] { display: none; }
        /* Floating selection toolbar (bubble menu) shown above a text selection.
           Dark by default to match the site's Publish button, with white icons. */
        .selection-toolbar { position: absolute; z-index: 25; display: flex; gap: 1px; padding: 3px; border-radius: var(--md-editor-toolbar-radius, 6px); background: var(--md-editor-toolbar-bg, #333); color: var(--md-editor-toolbar-fg, #fff); white-space: nowrap; transition: left var(--md-editor-toolbar-transition, 120ms cubic-bezier(.2,.8,.2,1)), top var(--md-editor-toolbar-transition, 120ms cubic-bezier(.2,.8,.2,1)); }
        .selection-toolbar[hidden] { display: none; }
        /* Don't animate the very first placement (avoids sliding in from a stale spot). */
        .selection-toolbar[data-instant] { transition: none; }
        @media (prefers-reduced-motion: reduce) { .selection-toolbar { transition: none; } }
        .selection-toolbar button { display: inline-flex; align-items: center; justify-content: center; inline-size: 34px; block-size: 34px; padding: 0; border: 0; border-radius: 4px; background: transparent; color: inherit; cursor: pointer; }
        .selection-toolbar button:hover { background: var(--md-editor-toolbar-hover, rgba(255,255,255,0.16)); }
        .selection-toolbar button:active { background: var(--md-editor-toolbar-active, rgba(255,255,255,0.28)); }
        .selection-toolbar button svg { inline-size: 20px; block-size: 20px; display: block; fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; }
        .selection-toolbar button svg [fill] { fill: currentColor; }
        @keyframes md-editor-pop { from { opacity: 0; transform: translateY(-3px) scale(.985); } to { opacity: 1; transform: translateY(0) scale(1); } }
        .completion-item { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 4px 12px; padding: 8px 10px; border-radius: 6px; cursor: pointer; }
        .completion-item[aria-selected="true"] { background: color-mix(in srgb, Highlight 18%, transparent); }
        .completion-item[aria-disabled="true"] { cursor: not-allowed; opacity: 0.58; }
        .completion-label { font-weight: 650; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .completion-detail, .completion-description { color: var(--md-editor-muted); font-size: 0.9em; } .completion-description { grid-column: 1 / -1; }
        .validation { min-block-size: 1.2em; color: var(--md-editor-danger); font-size: 0.92em; } .validation:empty { display: none; }
        .sr-only { position: absolute; inline-size: 1px; block-size: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0; }

        @media (prefers-reduced-motion: reduce) { *, *::before, *::after { scroll-behavior: auto !important; transition-duration: 0.001ms !important; animation-duration: 0.001ms !important; } }

        /* Nonograph theme: make the live document and slash menu resemble the
           published post output. Values track the editor's CSS custom properties
           so the host can still retheme via --md-editor-*. */
        .live-editor { caret-color: var(--md-editor-fg); }
        .md-line strong, .md-task-source strong { font-weight: 600; }
        .md-line em, .md-task-source em { font-style: italic; }
        .md-line del, .md-task-source del { text-decoration: line-through; }
        .md-line a, .md-task-source a { color: var(--md-editor-fg); text-decoration: underline; }
        .md-line a:hover, .md-task-source a:hover { color: color-mix(in srgb, var(--md-editor-fg) 70%, #000 30%); }
        .md-line code, .md-task-source code { background: var(--md-editor-code-bg); padding: 2px 6px; font-family: var(--md-editor-mono-font); font-size: 0.9em; }
        .md-heading { font-weight: 600; color: var(--md-editor-fg); }
        .md-h1 { font-size: 1.8em; }
        .md-h2 { font-size: 1.5em; }
        .md-h3 { font-size: 1.3em; }
        .md-h4 { font-size: 1.15em; }
        .md-quote { color: var(--md-editor-muted); background-image: none; border-inline-start: 2px solid var(--md-editor-border); padding-inline-start: 20px; }
        .md-table th, .md-table td { border: 1px solid var(--md-editor-border); }
        .md-table th { background: var(--md-editor-code-bg); font-weight: 600; }
        /* Slash-command menu matches the site's earlier .editor-menu look. */
        .completion-popup { border-radius: 4px; padding: 4px; min-inline-size: 200px; }
        .completion-item { padding: 4px 12px; border-radius: 2px; font-family: var(--md-editor-ui-font, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif); color: var(--md-editor-popup-fg); }
        .completion-item[aria-selected="true"] { background: color-mix(in srgb, var(--md-editor-popup-fg) 12%, transparent); }
        .completion-item:hover { background: color-mix(in srgb, var(--md-editor-popup-fg) 7%, transparent); }
        .completion-label { font-weight: 500; font-size: 14px; }
        .completion-detail, .completion-description { color: var(--md-editor-muted); font-size: 12px; }
      </style>
      <div class="container" part="container">
        <label class="label" part="label" id="${this._ids.label}" for="${this._ids.source}"></label>
        <div class="workspace">
          <div class="editor-shell" part="editor">
            <div class="live-editor" part="live-editor" id="${this._ids.live}" role="textbox" aria-multiline="true" tabindex="0" aria-controls="${this._ids.completion}" aria-expanded="false" aria-autocomplete="list" aria-describedby="${this._ids.validation}"></div>
            <textarea part="textarea" id="${this._ids.source}" aria-controls="${this._ids.completion}" aria-expanded="false" aria-autocomplete="list" aria-describedby="${this._ids.validation}" rows="12"></textarea>
            <div class="completion-popup" part="completion-popup" id="${this._ids.completion}" role="listbox" hidden></div>
            <div class="selection-toolbar" part="selection-toolbar" id="${this._ids.toolbar}" role="toolbar" aria-label="Formatting" hidden></div>
          </div>
        </div>
        <div class="validation" part="error" id="${this._ids.validation}"></div>
        <div class="sr-only" part="status" id="${this._ids.status}" aria-live="polite" aria-atomic="true"></div>
      </div>`;
    this._label = this._shadow.querySelector(".label");
    this._liveEditor = this._shadow.querySelector(".live-editor");
    this._sourceTextarea = this._shadow.querySelector("textarea");
    this._completionPopup = this._shadow.querySelector(".completion-popup");
    this._selectionToolbar = this._shadow.querySelector(".selection-toolbar");
    this._validation = this._shadow.querySelector(".validation");
    this._status = this._shadow.querySelector(".sr-only");
    this._buildSelectionToolbar();
  }

  _buildSelectionToolbar() {
    if (!this._selectionToolbar) return;
    this._selectionToolbar.innerHTML = this._selectionToolbarActions
      .map(item => `<button type="button" part="selection-toolbar-button" data-action="${escapeAttribute(item.actionId)}" title="${escapeAttribute(item.label)}" aria-label="${escapeAttribute(item.label)}" tabindex="-1">${item.html}</button>`)
      .join("");
    // Keep focus/selection in the editor when interacting with the toolbar.
    this._selectionToolbar.addEventListener("mousedown", event => event.preventDefault());
    this._selectionToolbar.addEventListener("click", event => {
      const button = event.target.closest?.("button[data-action]");
      if (!button) return;
      event.preventDefault();
      this._runAction(button.dataset.action, undefined, { source: "toolbar", apply: true });
      this._updateSelectionToolbar();
    });
  }

  _bindEvents() {
    this._sourceTextarea.addEventListener("beforeinput", () => this._beforeInputSnapshot = this._snapshot());
    this._sourceTextarea.addEventListener("input", event => this._onSourceInput(event));
    this._sourceTextarea.addEventListener("change", () => this._dispatch("md-change", { value: this._value }));
    this._sourceTextarea.addEventListener("keydown", event => this._onKeyDown(event));
    this._sourceTextarea.addEventListener("keyup", event => this._onNavigationKey(event));
    this._sourceTextarea.addEventListener("click", () => this._onSelectionChanged());
    this._sourceTextarea.addEventListener("select", () => this._onSelectionChanged());
    this._sourceTextarea.addEventListener("paste", event => this._onPaste(event));
    this._sourceTextarea.addEventListener("drop", event => this._onDrop(event));
    this._sourceTextarea.addEventListener("compositionstart", () => { this._isComposing = true; this._closeCompletion(); });
    this._sourceTextarea.addEventListener("compositionend", () => { this._isComposing = false; this._scheduleCompletionUpdate(); });

    this._liveEditor.addEventListener("focus", () => { if (this._selection.start > this._value.length) this._selection = { start: 0, end: 0, direction: "none" }; }, true);
    this._liveEditor.addEventListener("keydown", event => this._onKeyDown(event));
    this._liveEditor.addEventListener("keyup", event => this._onNavigationKey(event));
    this._liveEditor.addEventListener("beforeinput", event => this._onLiveBeforeInput(event));
    this._liveEditor.addEventListener("input", event => this._onLiveInput(event));
    this._liveEditor.addEventListener("click", event => this._onLiveClick(event));
    this._liveEditor.addEventListener("mousedown", event => this._onLiveMouseDown(event));
    this._liveEditor.addEventListener("mouseup", () => this._onSelectionChanged());
    this._liveEditor.addEventListener("scroll", () => this._onLiveScroll());
    this._liveEditor.addEventListener("copy", event => this._onLiveCopy(event));
    this._liveEditor.addEventListener("cut", event => this._onLiveCut(event));
    this._liveEditor.addEventListener("paste", event => this._onPaste(event));
    this._liveEditor.addEventListener("drop", event => this._onDrop(event));
    this._liveEditor.addEventListener("compositionstart", () => { this._isComposing = true; this._closeCompletion(); });
    this._liveEditor.addEventListener("compositionend", () => { this._isComposing = false; this._onSelectionChanged(); });

    this._completionPopup.addEventListener("mousedown", e => e.preventDefault());
    this._completionPopup.addEventListener("click", e => { const item = e.target.closest("[data-index]"); if (!item) return; const index = Number(item.dataset.index); if (this._completion.items[index]?.disabled) return; this._completion.activeIndex = index; this._acceptCompletion("pointer"); });
    this._label.addEventListener("click", event => { event.preventDefault(); this._focusEditable(); });
    this._shadow.addEventListener("focusin", () => { this._focusWithin = true; });
    this._shadow.addEventListener("focusout", () => { queueMicrotask(() => { this._focusWithin = Boolean(this._shadow.activeElement); if (!this._focusWithin) this._hideSelectionToolbar?.(); }); });
    this._shadow.addEventListener("selectionchange", () => this._onSelectionChanged?.());
  }

  _syncAttributesToControls() {
    if (!this._sourceTextarea) return;
    this._label.textContent = this.label; this._label.hidden = !this.label;
    this._sourceTextarea.placeholder = this.placeholder;
    this._sourceTextarea.disabled = this.disabled;
    this._sourceTextarea.readOnly = this.readonly;
    this._sourceTextarea.required = this.required;
    this._sourceTextarea.name = this.name;
    this._liveEditor.setAttribute("aria-readonly", this.readonly ? "true" : "false");
    this._liveEditor.setAttribute("aria-disabled", this.disabled ? "true" : "false");
    this._liveEditor.contentEditable = this._liveSelectionAPI === false
      ? "false"
      : this._lineEditable();
    this._liveEditor.tabIndex = this.disabled ? -1 : 0;
    const maxLength = parseLengthConstraint(this.getAttribute("maxlength"));
    const minLength = parseLengthConstraint(this.getAttribute("minlength"));
    maxLength != null ? this._sourceTextarea.maxLength = maxLength : this._sourceTextarea.removeAttribute("maxlength");
    minLength != null ? this._sourceTextarea.minLength = minLength : this._sourceTextarea.removeAttribute("minlength");
    const rawSpellcheck = this.getAttribute("spellcheck");
    const spellcheck = rawSpellcheck == null || rawSpellcheck === "" || rawSpellcheck === "true";
    this._sourceTextarea.spellcheck = spellcheck;
    this._liveEditor.spellcheck = spellcheck;
    const ariaLabel = this.getAttribute("aria-label"); const ariaLabelledby = this.getAttribute("aria-labelledby");
    for (const el of [this._sourceTextarea, this._liveEditor]) {
      if (ariaLabel) el.setAttribute("aria-label", ariaLabel); else el.removeAttribute("aria-label");
      if (ariaLabelledby) el.setAttribute("aria-labelledby", ariaLabelledby); else if (this.label) el.setAttribute("aria-labelledby", this._ids.label); else el.removeAttribute("aria-labelledby");
      const dir = this.getAttribute("dir");
      if (dir) el.dir = dir; else el.removeAttribute("dir");
    }
  }

  _getActiveStateIds(ctx) {
    const ids = [];
    const block = ctx.block || {};
    if (block.kind === "heading" && block.heading) ids.push(`block.heading.${block.heading.level}`);
    if (block.kind === "bullet-list-item") ids.push("block.bulletList");
    if (block.kind === "ordered-list-item") ids.push("block.orderedList");
    if (block.kind === "task-list-item") ids.push("block.taskList");
    if (block.kind === "blockquote") ids.push("block.blockquote");
    if (block.kind === "fenced-code") ids.push("block.codeFence");
    if (block.kind === "table") ids.push("block.table");
    const line = ctx.currentLine?.text ?? "";
    const pos = clamp(ctx.selectionStart - (ctx.currentLine?.start ?? 0), 0, line.length);
    const before = line.slice(0, pos); const after = line.slice(pos);
    if ((before.match(/\*\*/g) || []).length % 2 === 1 && after.includes("**")) ids.push("inline.bold");
    if ((before.match(/(?<!\*)\*(?!\*)/g) || []).length % 2 === 1 && /(?<!\*)\*(?!\*)/.test(after)) ids.push("inline.italic");
    if ((before.match(/(?<!\\)`/g) || []).length % 2 === 1 && /(?<!\\)`/.test(after)) ids.push("inline.code");
    if ((before.match(/_/g) || []).length % 2 === 1 && after.includes("_")) ids.push("inline.underline");
    if ((before.match(/~/g) || []).length % 2 === 1 && after.includes("~")) ids.push("inline.strikethrough");
    if ((before.match(/\^/g) || []).length % 2 === 1 && after.includes("^")) ids.push("inline.superscript");
    if ((before.match(/==/g) || []).length % 2 === 1 && after.includes("==")) ids.push("inline.highlight");
    if ((before.match(/#/g) || []).length % 2 === 1 && after.includes("#")) ids.push("inline.secret");
    if (/\[[^\]]*$/.test(before) && /\]\([^)]+\)/.test(after)) ids.push("inline.link");
    return ids;
  }
  _findText(query, options = {}) {
    const q = String(query ?? ""); if (!q) return null;
    const hay = options.caseSensitive ? this._value : this._value.toLowerCase();
    const needle = options.caseSensitive ? q : q.toLowerCase();
    const from = clamp(Number(options.from ?? this.selectionEnd ?? 0), 0, this._value.length);
    let index = hay.indexOf(needle, from);
    if (index === -1 && options.wrap !== false) index = hay.indexOf(needle, 0);
    if (index === -1) return null;
    this.setSelectionRange(index, index + q.length, "forward");
    this._announce("Match found.");
    return { start: index, end: index + q.length, text: this._value.slice(index, index + q.length) };
  }
  _replaceText(query, replacement, options = {}) {
    const q = String(query ?? ""); if (!q) return 0;
    const repl = normalizeLineEndings(replacement ?? "");
    const source = options.caseSensitive ? this._value : this._value.toLowerCase();
    const needle = options.caseSensitive ? q : q.toLowerCase();
    const changes = [];
    if (options.all) {
      let i = 0;
      while ((i = source.indexOf(needle, i)) !== -1) { changes.push({ from: i, to: i + q.length, insert: repl }); i += q.length || 1; }
    } else {
      const sel = this._getCurrentSelection();
      const selected = this._value.slice(Math.min(sel.start, sel.end), Math.max(sel.start, sel.end));
      const matchSelected = (options.caseSensitive ? selected : selected.toLowerCase()) === needle;
      const found = matchSelected ? { start: Math.min(sel.start, sel.end), end: Math.max(sel.start, sel.end) } : this._findText(q, options);
      if (found) changes.push({ from: found.start, to: found.end, insert: repl });
    }
    if (!changes.length) return 0;
    const first = changes[0].from + repl.length;
    this._applyTransaction({ changes, selectionAfter: { start: first, end: first, direction: "none" }, actionId: options.all ? "editor.replaceAll" : "editor.replace", undoGroup: "replace", source: "api" }, { source: "api" });
    return changes.length;
  }

  _parseOptions() { return { markdownFlavor: this.markdownFlavor }; }
  _referenceDefinitions() {
    if (this._referenceCacheValue === this._value && this._referenceCache) return this._referenceCache;
    this._referenceCacheValue = this._value;
    this._referenceCache = extractReferenceDefinitions(this._value).references;
    return this._referenceCache;
  }
  _rendererOptions() {
    return {
      ...this._parseOptions(),
      allowRawHtml: false,
      sanitize: true,
      linkTarget: DEFAULTS.linkTarget,
      references: this._referenceDefinitions(),
    };
  }
  _setValueInternal(next, opts = {}) {
    const previousValue = this._value;
    const value = normalizeLineEndings(next ?? ""); const before = this._snapshot(); const changed = value !== previousValue;
    this._value = value; if (this._sourceTextarea && this._sourceTextarea.value !== value) this._sourceTextarea.value = value;
    if (!opts.preserveSelection) this._selection = { start: clamp(this._selection.start, 0, value.length), end: clamp(this._selection.end, 0, value.length), direction: "none" };
    if (opts.recordUndo && changed) this._recordUndo(before, this._snapshot(), opts.undoGroup || opts.source || "api", { coalesce: false });
    if (changed || opts.force || !this._hasRenderedOnce) this._afterValueChanged({ source: opts.source || "api", silent: opts.silent, restoreSelection: opts.preserveSelection !== false, previousValue, changes: changed ? diffTextChange(previousValue, value) : [] });
  }
  _afterValueChanged({ source = "api", inputType = null, silent = false, restoreSelection = true, previousValue = null, changes = null } = {}) {
    this._selectAllLevel = 0;
    this._structuredSelection = null;
    if (["user", "keyboard", "paste", "pointer"].includes(source)) this._validationVisible = true;
    this._updateFormValue(); this._updateValidity(); this._renderAll({ restoreSelection, previousValue, changes });
    const oldDirty = this._dirty; this._dirty = this._value !== this._defaultValue; if (oldDirty !== this._dirty) this._dispatch("md-dirty-change", { dirty: this._dirty });
    this._updateSelectionToolbar();
    if (!silent) this._dispatch("md-input", { value: this._value, source, inputType });
  }
  _renderAll({ restoreSelection = true, previousValue = null, changes = null, force = false } = {}) {
    if (!this._liveEditor) return;
    this._hasRenderedOnce = true;
    this._sourceTextarea.value = this._value;
    if (this._isLiveVisible()) {
      this._renderLive({ previousValue, changes, force });
      this._liveDirty = false;
      if (restoreSelection && !this._isSourceActive()) this._restoreLiveSelection(this._selection);
    } else {
      this._liveDirty = true;
    }
  }
  _isLiveVisible() {
    return this.mode === "live";
  }
  _getBlocks() {
    if (this._blockCacheValue === this._value && this._blockCache) return this._blockCache;
    const blocks = parseBlocks(this._value, this._parseOptions());
    this._setBlockCache(blocks, "full");
    return blocks;
  }
  _setBlockCache(blocks, mode = "full") {
    assignHeadingIds(blocks);
    this._blockCacheValue = this._value;
    this._blockCache = blocks;
    this._lastParseMode = mode;
  }
  _blocksForRender(previousValue, changes) {
    const incremental = this._tryIncrementalBlocks(previousValue, changes);
    if (incremental) {
      this._setBlockCache(incremental, "incremental");
      return incremental;
    }
    const blocks = parseBlocks(this._value, this._parseOptions());
    this._setBlockCache(blocks, "full");
    return blocks;
  }
  _renderLive({ previousValue = null, changes = null, force = false, virtualAnchorOffset = null } = {}) {
    const previousBlocks = this._liveBlocks || [];
    const blocks = this._blocksForRender(previousValue, changes);
    if (this._shouldVirtualize(blocks)) {
      this._renderLiveVirtual(blocks, { anchorOffset: virtualAnchorOffset ?? this._selection.start, force });
      this._liveBlocks = blocks;
      this._rebuildLiveIndex();
      return;
    }
    this._virtualState = { ...this._virtualState, active: false, start: 0, end: blocks.length, total: blocks.length };
    const patched = !force && this._tryPatchLiveBlocks(previousBlocks, blocks, changes, previousValue);
    if (!patched) this._renderLiveFull(blocks);
    this._liveBlocks = blocks;
    this._rebuildLiveIndex();
  }
  _renderLiveFull(blocks) {
    const html = blocks.map(block => this._renderLiveBlock(block)).join("");
    this._liveEditor.innerHTML = html || `<div class="live-placeholder">${escapeHtml(this.placeholder)}</div>`;
  }
  _tryPatchLiveBlocks(previousBlocks, blocks, changes, previousValue) {
    if (!previousValue || !changes?.length || !previousBlocks.length || !this._liveEditor?.children?.length || this._virtualState.active) return false;
    const children = Array.from(this._liveEditor.children);
    if (children.length !== previousBlocks.length) return false;
    const spans = changedSpans(changes);
    if (!spans) return false;
    const oldRange = this._expandedBlockRange(previousBlocks, spans.oldStart, spans.oldEnd, previousValue.length);
    const newRange = this._expandedBlockRange(blocks, spans.newStart, spans.newEnd, this._value.length);
    if (!oldRange || !newRange) return false;
    if ((oldRange.end - oldRange.start) > 250 || (newRange.end - newRange.start) > 250) return false;
    const fragment = this._liveFragmentForBlocks(blocks.slice(newRange.start, newRange.end));
    const reference = children[oldRange.end] || null;
    for (const node of children.slice(oldRange.start, oldRange.end)) node.remove();
    this._liveEditor.insertBefore(fragment, reference);
    if (!this._syncLiveMetadata(blocks)) {
      this._renderLiveFull(blocks);
    }
    return true;
  }
  _expandedBlockRange(blocks, start, end, valueLength = this._value.length) {
    if (!blocks.length) return { start: 0, end: 0 };
    const range = this._blockRangeForSourceRange(blocks, start, end, valueLength);
    if (!range) return null;
    return {
      start: clamp(range.start - 1, 0, blocks.length),
      end: clamp(range.end + 1, 0, blocks.length),
    };
  }
  _blockRangeForSourceRange(blocks, start, end, valueLength = this._value.length) {
    if (!blocks.length) return { start: 0, end: 0 };
    const rangeStart = clamp(Number(start) || 0, 0, valueLength);
    const rangeEnd = clamp(Number(end) || rangeStart, rangeStart, Math.max(rangeStart, valueLength));
    let first = -1;
    let last = -1;
    const pointRange = rangeStart === rangeEnd;
    for (let i = 0; i < blocks.length; i += 1) {
      const block = blocks[i];
      const blockEnd = Math.max(block.newlineEnd ?? block.to ?? block.from, block.to ?? block.from, block.from);
      if (first === -1 && (pointRange ? blockEnd >= rangeStart : blockEnd > rangeStart)) first = i;
      if (first !== -1 && block.from <= rangeEnd) last = i;
      if (first !== -1 && block.from > rangeEnd) break;
    }
    if (first === -1) first = Math.max(0, blocks.length - 1);
    if (last === -1 || last < first) last = first;
    return { start: first, end: Math.min(blocks.length, last + 1) };
  }
  _liveFragmentForBlocks(blocks) {
    const template = document.createElement("template");
    template.innerHTML = blocks.map(block => this._renderLiveBlock(block)).join("");
    return template.content;
  }
  _syncLiveMetadata(blocks) {
    const children = Array.from(this._liveEditor.children).filter(node => !node.classList?.contains("md-virtual-spacer"));
    if (children.length !== blocks.length) return false;
    for (let i = 0; i < blocks.length; i += 1) {
      if (!this._syncLiveBlockNodeMetadata(children[i], blocks[i])) return false;
    }
    return true;
  }
  _setEditableMetadata(el, from, to, editable = this._lineEditable(), spellcheck = this._sourceTextarea?.spellcheck ? "true" : "false") {
    if (!el) return false;
    el.dataset.from = String(from);
    el.dataset.to = String(to);
    if (el.hasAttribute("contenteditable")) el.contentEditable = editable;
    if (el.hasAttribute("spellcheck")) el.setAttribute("spellcheck", spellcheck);
    return true;
  }
  _syncLiveBlockNodeMetadata(node, block) {
    if (!node || !block) return false;
    const blockFrom = block.from ?? block.line?.start ?? 0;
    const blockTo = block.to ?? block.line?.end ?? blockFrom;
    if (node.dataset) {
      node.dataset.kind = block.type;
      node.dataset.from = String(blockFrom);
      node.dataset.to = String(blockTo);
    }
    if (block.type === "table") {
      const cells = Array.from(node.querySelectorAll('.md-cell[data-editable="cell"]'));
      const cols = Math.max(block.header.cells.length, ...block.rows.map(row => row.cells.length), 1);
      const bodyRows = block.rows.length ? block.rows : [{ cells: Array.from({ length: cols }, () => ({ text: "", from: block.delimiter.end, to: block.delimiter.end })) }];
      const expected = [
        ...Array.from({ length: cols }, (_, col) => ({ cell: block.header.cells[col] ?? { from: block.header.end, to: block.header.end }, row: -1, col })),
        ...bodyRows.flatMap((row, rowIndex) => Array.from({ length: cols }, (_, col) => ({ cell: row.cells[col] ?? { from: row.end, to: row.end }, row: rowIndex, col }))),
      ];
      if (cells.length !== expected.length) return false;
      cells.forEach((cell, index) => {
        const meta = expected[index];
        cell.dataset.row = String(meta.row);
        cell.dataset.col = String(meta.col);
        this._setEditableMetadata(cell, meta.cell.from, meta.cell.to);
      });
      return true;
    }
    if (block.type === "code-fence") {
      node.dataset.language = String(block.language || "");
      const editables = Array.from(node.querySelectorAll("[data-editable]"));
      const virtualOffset = block.codeLines[0]?.start ?? (block.closing ? block.opening.newlineEnd : block.opening.end);
      const lines = block.codeLines.length ? block.codeLines : [{ start: virtualOffset, end: virtualOffset }];
      if (editables.length !== lines.length) return false;
      editables.forEach((editable, index) => this._setEditableMetadata(editable, lines[index].start, lines[index].end, this._lineEditable(), "false"));
      return true;
    }
    if (block.type === "task-list-item") {
      const source = node.querySelector("[data-editable]");
      const checkbox = node.querySelector("[data-task-checkbox]");
      if (!source) return false;
      if (checkbox) checkbox.dataset.checkOffset = String(block.line.start + block.list.indent.length + `${block.list.marker} [`.length);
      return this._setEditableMetadata(source, block.line.start + block.list.contentStart, block.line.end);
    }
    if (block.type === "heading") {
      const heading = node.matches?.(".md-heading") ? node : node.querySelector?.(".md-heading");
      if (!heading) return false;
      heading.id = block.heading.id;
      return this._setEditableMetadata(heading, block.line.start, block.line.end);
    }
    const editable = node.matches?.("[data-editable]") ? node : node.querySelector?.("[data-editable]");
    if (editable && block.line) return this._setEditableMetadata(editable, block.line.start, block.line.end);
    return block.type === "horizontal-rule";
  }
  _lineAt(value, offset) {
    const line = getLineRange(value, offset);
    return { ...line, newlineEnd: line.end < value.length && value[line.end] === "\n" ? line.end + 1 : line.end };
  }
  _previousLineText(value, lineStart) {
    if (lineStart <= 0) return "";
    return getLineRange(value, lineStart - 1).text;
  }
  _nextLineText(value, lineEnd) {
    if (lineEnd >= value.length) return "";
    return getLineRange(value, lineEnd + 1).text;
  }
  _lineHasStructuralNeighbors(value, line) {
    const texts = [this._previousLineText(value, line.start), line.text, this._nextLineText(value, line.end)];
    return texts.some(text => isFenceLine(text) || isLikelyTableRow(text) || isTableDelimiter(text));
  }
  _parseSingleLineBlock(line) {
    const heading = parseHeading(line.text);
    if (heading) return { type: "heading", from: line.start, to: line.end, newlineEnd: line.newlineEnd, line, heading };
    const list = parseListItem(line.text, this._parseOptions());
    if (list) return { type: list.kind, from: line.start, to: line.end, newlineEnd: line.newlineEnd, line, list };
    const quote = parseBlockquote(line.text);
    if (quote) return { type: "blockquote", from: line.start, to: line.end, newlineEnd: line.newlineEnd, line, quote };
    if (isHorizontalRule(line.text)) return { type: "horizontal-rule", from: line.start, to: line.end, newlineEnd: line.newlineEnd, line };
    return { type: line.text.trim() ? "paragraph" : "blank", from: line.start, to: line.end, newlineEnd: line.newlineEnd, line };
  }
  _tryIncrementalBlocks(previousValue, changes) {
    const sorted = normalizeChanges(changes || []);
    if (!previousValue || sorted.length !== 1 || this._blockCacheValue !== previousValue || !this._blockCache) return null;
    const change = sorted[0];
    const removed = previousValue.slice(change.from, change.to);
    if (removed.includes("\n") || change.insert.includes("\n")) return null;
    const oldLine = this._lineAt(previousValue, change.from);
    const newLine = this._lineAt(this._value, change.from + change.insert.length);
    if (oldLine.start !== newLine.start || this._lineHasStructuralNeighbors(previousValue, oldLine) || this._lineHasStructuralNeighbors(this._value, newLine)) return null;
    const oldBlocks = this._blockCache;
    const index = oldBlocks.findIndex(block => block.from === oldLine.start && block.to === oldLine.end && block.line);
    if (index === -1) return null;
    const oldBlock = oldBlocks[index];
    if (oldBlock.type === "table" || oldBlock.type === "code-fence") return null;
    const newBlock = this._parseSingleLineBlock(newLine);
    const delta = this._value.length - previousValue.length;
    return oldBlocks.map((block, i) => {
      if (i < index) return block;
      if (i === index) return newBlock;
      return this._shiftBlockOffsets(block, delta);
    });
  }
  _shiftCell(cell, delta) {
    return cell ? { ...cell, from: cell.from + delta, to: cell.to + delta } : cell;
  }
  _shiftLineOffsets(line, delta) {
    if (!line) return line;
    const shifted = { ...line, start: line.start + delta, end: line.end + delta, newlineEnd: line.newlineEnd + delta };
    if (line.cells) shifted.cells = line.cells.map(cell => this._shiftCell(cell, delta));
    return shifted;
  }
  _shiftBlockOffsets(block, delta) {
    const shifted = { ...block, from: block.from + delta, to: block.to + delta, newlineEnd: block.newlineEnd + delta };
    if (block.line) shifted.line = this._shiftLineOffsets(block.line, delta);
    if (block.opening) shifted.opening = this._shiftLineOffsets(block.opening, delta);
    if (block.closing) shifted.closing = this._shiftLineOffsets(block.closing, delta);
    if (block.codeLines) shifted.codeLines = block.codeLines.map(line => this._shiftLineOffsets(line, delta));
    if (block.header) shifted.header = this._shiftLineOffsets(block.header, delta);
    if (block.delimiter) shifted.delimiter = this._shiftLineOffsets(block.delimiter, delta);
    if (block.rows) shifted.rows = block.rows.map(row => this._shiftLineOffsets(row, delta));
    return shifted;
  }
  _shouldVirtualize(blocks) {
    return blocks.length > 2500 || this._value.length > DEFAULTS.largeDocChars * 2;
  }
  _virtualLineHeight() {
    return Math.max(18, this._computedLineHeight(this._liveEditor || this));
  }
  _virtualWindowSize(lineHeight = this._virtualLineHeight()) {
    const viewportRows = Math.ceil((this._liveEditor.clientHeight || 600) / lineHeight);
    return clamp(viewportRows + 180, 220, 520);
  }
  _blockIndexForOffset(blocks, offset) {
    const safe = clamp(offset, 0, this._value.length);
    let low = 0;
    let high = blocks.length - 1;
    let best = 0;
    while (low <= high) {
      const mid = Math.floor((low + high) / 2);
      const block = blocks[mid];
      if (safe < block.from) high = mid - 1;
      else {
        best = mid;
        if (safe <= Math.max(block.to, block.from)) return mid;
        low = mid + 1;
      }
    }
    return clamp(best, 0, Math.max(0, blocks.length - 1));
  }
  _renderLiveVirtual(blocks, { anchorOffset = this._selection.start, force = false, fromScroll = false } = {}) {
    const total = blocks.length;
    const lineHeight = this._virtualLineHeight();
    const windowSize = this._virtualWindowSize(lineHeight);
    const scrollIndex = Math.floor((this._liveEditor.scrollTop || 0) / lineHeight);
    const anchorIndex = fromScroll ? scrollIndex : this._blockIndexForOffset(blocks, anchorOffset);
    let start = clamp(anchorIndex - Math.floor(windowSize / 2), 0, Math.max(0, total - windowSize));
    let end = clamp(start + windowSize, start, total);
    if (end - start < windowSize && start > 0) start = Math.max(0, end - windowSize);
    if (!force && this._virtualState.active && start === this._virtualState.start && end === this._virtualState.end && total === this._virtualState.total) return;
    const topHeight = Math.round(start * lineHeight);
    const bottomHeight = Math.round((total - end) * lineHeight);
    const top = `<div class="md-virtual-spacer" contenteditable="false" aria-hidden="true" style="block-size:${topHeight}px"></div>`;
    const bottom = `<div class="md-virtual-spacer" contenteditable="false" aria-hidden="true" style="block-size:${bottomHeight}px"></div>`;
    this._liveEditor.innerHTML = `${top}${blocks.slice(start, end).map(block => this._renderLiveBlock(block)).join("")}${bottom}`;
    this._virtualState = { active: true, start, end, total, lineHeight };
  }
  _isSourceOffsetRendered(offset) {
    if (!this._virtualState.active) return true;
    const blocks = this._liveBlocks?.length ? this._liveBlocks : this._getBlocks();
    const index = this._blockIndexForOffset(blocks, offset);
    return index >= this._virtualState.start && index < this._virtualState.end;
  }
  _ensureVirtualSelectionVisible(selection) {
    if (!this._virtualState.active) return true;
    const blocks = this._liveBlocks?.length ? this._liveBlocks : this._getBlocks();
    const startIndex = this._blockIndexForOffset(blocks, Math.min(selection.start, selection.end));
    const endIndex = this._blockIndexForOffset(blocks, Math.max(selection.start, selection.end));
    if (startIndex >= this._virtualState.start && endIndex < this._virtualState.end) return true;
    if ((endIndex - startIndex + 1) > this._virtualWindowSize()) return false;
    const focus = selection.direction === "backward" ? selection.start : selection.end;
    this._renderLiveVirtual(blocks, { anchorOffset: focus, force: true });
    this._rebuildLiveIndex();
    return startIndex >= this._virtualState.start && endIndex < this._virtualState.end;
  }
  _onLiveScroll() {
    if (this._selectionToolbar && !this._selectionToolbar.hidden) this._positionSelectionToolbar();
    if (!this._virtualState.active || this._virtualScrollFrame) return;
    this._virtualScrollFrame = requestAnimationFrame(() => {
      this._virtualScrollFrame = 0;
      if (!this._virtualState.active || !this._liveEditor) return;
      const blocks = this._liveBlocks?.length ? this._liveBlocks : this._getBlocks();
      const nextStart = Math.floor((this._liveEditor.scrollTop || 0) / Math.max(1, this._virtualState.lineHeight));
      if (Math.abs(nextStart - this._virtualState.start) < 60) return;
      this._renderLiveVirtual(blocks, { fromScroll: true, force: true });
      this._rebuildLiveIndex();
    });
  }
  _renderLiveBlock(block) {
    const options = this._rendererOptions();
    const lineAttrs = (line, kind, extra = "", attrs = "") => `class="md-line ${extra}" part="line" data-editable="line" data-kind="${kind}" data-from="${line.start}" data-to="${line.end}" contenteditable="${this._lineEditable()}" spellcheck="${this._sourceTextarea?.spellcheck ? "true" : "false"}"${attrs ? ` ${attrs}` : ""}`;
    if (block.type === "blank") {
      const placeholderAttrs = this._value.length === 0 ? `data-placeholder="${escapeAttribute(this.placeholder)}"` : "";
      const placeholderClass = this._value.length === 0 ? "md-empty-placeholder" : "";
      return `<div ${lineAttrs(block.line, "blank", placeholderClass, placeholderAttrs)}>${block.line.text ? decorateInline(block.line.text, options) : "<br>"}</div>`;
    }
    if (block.type === "heading") {
      const afterAnchor = block.setext && block.newlineEnd === block.to
        ? `<div class="md-line md-setext-after" part="line" data-editable="virtual-setext-after" data-kind="blank" data-from="${block.to}" data-to="${block.to}" contenteditable="${this._lineEditable()}" spellcheck="${this._sourceTextarea?.spellcheck ? "true" : "false"}" aria-label="After heading"><br></div>`
        : "";
      const content = block.setext ? decorateInline(block.line.text, options) : this._renderHeadingLine(block.line.text, block.heading);
      return `<div ${lineAttrs(block.line, "heading", `md-heading md-h${block.heading.level}`, `id="${escapeAttribute(block.heading.id)}"`)}>${content}</div>${afterAnchor}`;
    }
    if (block.type === "blockquote") {
      const depth = Math.max(1, Number(block.quote.depth) || 1);
      const visualDepth = Math.min(depth, 16);
      const markerEnd = block.quote.fullContentStart ?? block.quote.contentStart;
      const marker = `<span class="md-token">${escapeHtml(block.line.text.slice(0, markerEnd))}</span>`;
      const content = decorateInline(block.line.text.slice(markerEnd), options);
      return `<div ${lineAttrs(block.line, "blockquote", `md-quote md-quote-depth-${visualDepth}`, `data-quote-depth="${depth}"`)}>${marker}${content}</div>`;
    }
    if (block.type === "horizontal-rule") {
      // Keep the divider line editable and its source text visible so it can be
      // typed/backspaced through (e.g. editing ****/*** while toggling bold).
      return `<div ${lineAttrs(block.line, "horizontal-rule", "md-hr-line")} aria-label="Divider"><span class="md-token">${escapeHtml(block.line.text)}</span></div>`;
    }
    if (block.type === "task-list-item") {
      const list = block.list; const checkOffset = block.line.start + list.indent.length + `${list.marker} [`.length;
      const contentFrom = block.line.start + list.contentStart;
      const taskName = textFromMarkdown(list.content, options) || "task";
      const checkboxLabel = `${list.checked ? "Mark task incomplete" : "Mark task complete"}: ${taskName}`;
      return `<div class="md-line md-task-line md-list" part="line" data-kind="task-list-item" data-from="${block.line.start}" data-to="${block.line.end}" style="--md-list-depth:${Math.floor(list.indent.length / Math.max(1, this.indentString.length))}"><input type="checkbox" part="checkbox" data-task-checkbox="true" data-check-offset="${checkOffset}" aria-label="${escapeAttribute(checkboxLabel)}" ${list.checked ? "checked" : ""} ${this.disabled || this.readonly ? "disabled" : ""}><span class="md-task-source" data-editable="line" data-from="${contentFrom}" data-to="${block.line.end}" contenteditable="${this._lineEditable()}" spellcheck="${this._sourceTextarea?.spellcheck ? "true" : "false"}">${this._renderTaskLine(list)}</span></div>`;
    }
    if (block.type === "bullet-list-item" || block.type === "ordered-list-item") {
      const list = block.list; const depth = Math.floor(list.indent.length / Math.max(1, this.indentString.length));
      return `<div ${lineAttrs(block.line, block.type, "md-list")} style="--md-list-depth:${depth}">${decorateInline(block.line.text, options)}</div>`;
    }
    if (block.type === "code-fence") return this._renderCodeFence(block);
    if (block.type === "table") return this._renderTable(block);
    const content = parseReferenceDefinition(block.line.text)
      ? escapeHtml(block.line.text)
      : decorateInline(block.line.text, options);
    return `<div ${lineAttrs(block.line, "paragraph")}>${content}</div>`;
  }
  _lineEditable() { return (!this.disabled && !this.readonly) ? "true" : "false"; }
  _renderHeadingLine(text, heading) {
    const markerEnd = heading.indent.length + heading.markerText.length;
    return `<span class="md-token">${escapeHtml(text.slice(0, markerEnd))}</span>${decorateInline(text.slice(markerEnd), this._rendererOptions())}`;
  }
  _renderTaskLine(list) { return decorateInline(list.content, this._rendererOptions()); }
  _renderCodeFence(block) {
    const editable = this._lineEditable();
    const language = String(block.language || "").trim();
    // Map the header language label directly onto the language text in the
    // opening fence line (```LANGUAGE) so the user can type the language there.
    const openingText = block.opening.text;
    const infoMatch = /^(\s{0,3})(`{3,}|~{3,})([ \t]*)/.exec(openingText);
    const langStartInLine = infoMatch ? infoMatch[0].length : block.fence.sequence.length;
    const langFrom = block.opening.start + langStartInLine;
    const langTo = block.opening.end;
    const placeholderAttr = language ? "" : ` data-placeholder="code"`;
    const placeholderClass = language ? "" : " md-empty-placeholder";
    const label = `<span class="md-code-label${placeholderClass}" part="code-language" data-editable="line" data-kind="code-language" data-from="${langFrom}" data-to="${langTo}" contenteditable="${editable}" spellcheck="false"${placeholderAttr}>${escapeHtml(language) || "<br>"}</span>`;
    const header = `<div class="md-code-header" part="code-header" contenteditable="false">${label}</div>`;
    const codeLines = block.codeLines.map(line => `<div class="md-code-line" part="code-line" data-editable="line" data-kind="code-line" data-from="${line.start}" data-to="${line.end}" contenteditable="${editable}" spellcheck="false">${escapeHtml(line.text) || "<br>"}</div>`).join("");
    const virtualOffset = block.codeLines[0]?.start ?? (block.closing ? block.opening.newlineEnd : block.opening.end);
    const virtualLine = `<div class="md-code-line" part="code-line" data-editable="virtual-code" data-kind="code-line" data-from="${virtualOffset}" data-to="${virtualOffset}" contenteditable="${editable}" spellcheck="false"><br></div>`;
    const afterAnchor = block.closing && block.newlineEnd === block.to
      ? `<div class="md-line md-code-after" part="line" data-editable="virtual-code-after" data-kind="blank" data-from="${block.to}" data-to="${block.to}" contenteditable="${editable}" spellcheck="${this._sourceTextarea?.spellcheck ? "true" : "false"}" aria-label="After code block"><br></div>`
      : "";
    return `<div class="md-code-block" part="code-block" data-kind="code-fence" data-from="${block.from}" data-to="${block.to}" data-language="${escapeAttribute(language)}">${header}<div class="md-code-lines" part="code-lines">${codeLines || virtualLine}</div></div>${afterAnchor}`;
  }
  _renderTable(block) {
    const cols = Math.max(block.header.cells.length, ...block.rows.map(r => r.cells.length), 1);
    const alignments = Array.from({ length: cols }, (_, i) => tableAlignmentFromDelimiter(block.delimiter.cells[i]?.text));
    const renderCell = (cell, tag, row, col) => `<${tag}${tableAlignmentStyle(alignments[col])}><div class="md-cell" part="table-cell" data-editable="cell" data-row="${row}" data-col="${col}" data-from="${cell?.from ?? block.to}" data-to="${cell?.to ?? block.to}" contenteditable="${this._lineEditable()}" spellcheck="${this._sourceTextarea?.spellcheck ? "true" : "false"}">${decorateInline(unescapeTableCellText(cell?.text ?? ""), this._rendererOptions())}</div></${tag}>`;
    const header = `<thead><tr>${Array.from({ length: cols }, (_, i) => renderCell(block.header.cells[i] ?? { text: "", from: block.header.end, to: block.header.end }, "th", -1, i)).join("")}</tr></thead>`;
    const bodyRows = block.rows.length ? block.rows : [{ cells: Array.from({ length: cols }, () => ({ text: "", from: block.delimiter.end, to: block.delimiter.end })) }];
    const body = `<tbody>${bodyRows.map((row, r) => `<tr>${Array.from({ length: cols }, (_, i) => renderCell(row.cells[i] ?? { text: "", from: row.end, to: row.end }, "td", r, i)).join("")}</tr>`).join("")}</tbody>`;
    const afterAnchor = block.newlineEnd === block.to
      ? `<div class="md-line md-table-after" part="line" data-editable="virtual-table-after" data-kind="blank" data-from="${block.to}" data-to="${block.to}" contenteditable="${this._lineEditable()}" spellcheck="${this._sourceTextarea?.spellcheck ? "true" : "false"}" aria-label="After table"><br></div>`
      : "";
    return `<div class="md-table-block" part="table" data-kind="table" data-from="${block.from}" data-to="${block.to}"><table class="md-table">${header}${body}</table></div>${afterAnchor}`;
  }

  _onSourceInput(event) {
    const before = this._beforeInputSnapshot;
    const previousValue = this._value;
    this._value = normalizeLineEndings(this._sourceTextarea.value);
    if (this._sourceTextarea.value !== this._value) this._sourceTextarea.value = this._value;
    this._selection = { start: this._sourceTextarea.selectionStart, end: this._sourceTextarea.selectionEnd, direction: this._sourceTextarea.selectionDirection || "none" };
    const after = this._snapshot();
    if (before) this._recordUndo(before, after, this._undoGroupForInput(event?.inputType), { coalesce: event?.inputType === "insertText" || event?.inputType === "deleteContentBackward" });
    this._beforeInputSnapshot = null;
    this._afterValueChanged({ source: "user", inputType: event?.inputType, restoreSelection: false, previousValue, changes: diffTextChange(previousValue, this._value) });
    if (!this._isComposing) this._scheduleCompletionUpdate();
  }
  _onLiveBeforeInput(event) {
    if (this._isComposing) return;
    this._ignoreSelectionChangeCount = 0;
    this._structuredSelection = null;
    const inputType = event?.inputType || "";
    const inputTarget = this._inputTargetFromBeforeInput(event);
    this._beforeInputTarget = inputTarget;
    if (this._liveSelectionAPI === false && inputTarget) {
      const targetHasSelection = inputTarget.selection.start !== inputTarget.selection.end;
      const modelHasSelection = this._selection.start !== this._selection.end;
      if (!this._fallbackSelectionPending || (targetHasSelection && !modelHasSelection)) {
        this._selection = { ...inputTarget.selection };
      }
      this._fallbackEditable = inputTarget.editable;
    }
    if (this._value.length === 0 && inputType.startsWith("delete")) {
      event.preventDefault();
      this._beforeInputTarget = null;
      this._ensureEmptyLiveEditable();
      return;
    }
    if (!this._isSourceActive() && !this.disabled && !this.readonly) {
      const ctx = this._getContext();
      const hasSelection = ctx.selectionStart !== ctx.selectionEnd;
      const editable = this._closestEditable(event.target) || this._activeEditableFromSelection();
      const editableRange = this._editableSourceRange(editable);
      const selectionInsideTableCell = editable?.dataset.editable === "cell"
        && editableRange
        && ctx.selectionStart >= editableRange.from
        && ctx.selectionEnd <= editableRange.to;
      if (hasSelection && inputType.startsWith("delete") && !selectionInsideTableCell) {
        event.preventDefault();
        this._beforeInputTarget = null;
        this._applyActionResult("editor.deleteSelection", this._deleteSelectionResult(ctx, "editor.deleteSelection"), { source: "user" });
        return;
      }
      if (hasSelection && inputType === "insertText" && event.data != null && !selectionInsideTableCell) {
        event.preventDefault();
        this._beforeInputTarget = null;
        const text = normalizeLineEndings(event.data);
        this._applyActionResult("editor.replaceSelection", insertionTransaction(ctx, "editor.replaceSelection", text, text.length, "typing"), { source: "user" });
        return;
      }
    }
    if (this._liveSelectionAPI === false && this._applyFallbackBeforeInput(event, inputTarget)) {
      this._beforeInputTarget = null;
      return;
    }
    this._beforeInputSnapshot = this._snapshot();
  }
  _onLiveInput(event) {
    if (this.disabled || this.readonly) return;
    this._ignoreSelectionChangeCount = 0;
    this._structuredSelection = null;
    const inputTarget = this._beforeInputTarget;
    const editable = this._closestEditable(event.target) || inputTarget?.editable || this._activeEditableFromSelection();
    this._beforeInputTarget = null;
    if (!editable) return;
    const before = this._beforeInputSnapshot;
    const from = Number(editable.dataset.from); const to = Number(editable.dataset.to);
    const raw = this._plainText(editable).replace(/\n/g, "");
    const liveSelection = this._getLiveSelection(editable);
    const changedDisplay = inputTarget
      ? diffTextChange(inputTarget.text, raw)[0]
      : null;
    const inferredDisplayCursor = changedDisplay
      ? changedDisplay.from + changedDisplay.insert.length
      : null;
    const tableDisplayCursor = editable.dataset.editable === "cell"
      ? (this._displayOffsetFromSelection(editable) ?? inferredDisplayCursor)
      : null;
    const tableEdit = editable.dataset.editable === "cell" ? this._tableCellInputEdit(editable, raw, tableDisplayCursor) : null;
    if (tableEdit) {
      const previousValue = this._value;
      this._value = tableEdit.nextValue;
      this._selection = { start: tableEdit.cursor, end: tableEdit.cursor, direction: "none" };
      const after = makeSnapshot(this._value, this._selection.start, this._selection.end, this._selection.direction);
      if (before) this._recordUndo(before, after, this._undoGroupForInput(event?.inputType), { coalesce: event?.inputType === "insertText" || event?.inputType === "deleteContentBackward" });
      this._beforeInputSnapshot = null;
      this._afterValueChanged({ source: "user", inputType: event?.inputType, restoreSelection: true, previousValue, changes: diffTextChange(previousValue, this._value) });
      if (!this._isComposing) this._scheduleCompletionUpdate();
      return;
    }
    let insert = raw;
    let virtualPrefixLength = 0;
    if (editable.dataset.editable === "virtual-code" || editable.dataset.editable === "virtual-code-after" || editable.dataset.editable === "virtual-hr-after" || editable.dataset.editable === "virtual-setext-after" || editable.dataset.editable === "virtual-table-after") {
      const beforeSource = this._value.slice(0, from);
      if (!beforeSource.endsWith("\n")) {
        insert = `\n${raw}`;
        virtualPrefixLength = 1;
      }
    }
    const previousValue = this._value;
    const nextValue = previousValue.slice(0, from) + insert + previousValue.slice(to);
    const liveCursor = liveSelection?.end != null
      ? liveSelection.end - from
      : (inferredDisplayCursor ?? insert.length - virtualPrefixLength);
    const cursor = clamp(from + virtualPrefixLength + liveCursor, from, from + insert.length);
    this._value = nextValue;
    this._selection = { start: cursor, end: cursor, direction: "none" };
    const after = makeSnapshot(this._value, this._selection.start, this._selection.end, this._selection.direction);
    if (before) this._recordUndo(before, after, this._undoGroupForInput(event?.inputType), { coalesce: event?.inputType === "insertText" || event?.inputType === "deleteContentBackward" });
    this._beforeInputSnapshot = null;
    this._afterValueChanged({ source: "user", inputType: event?.inputType, restoreSelection: true, previousValue, changes: [{ from, to, insert }] });
    if (!this._isComposing) this._scheduleCompletionUpdate();
  }
  _inputTargetFromBeforeInput(event) {
    const range = event?.getTargetRanges?.()[0];
    if (!range) return null;
    const startElement = range.startContainer?.nodeType === Node.ELEMENT_NODE
      ? range.startContainer
      : range.startContainer?.parentElement;
    const endElement = range.endContainer?.nodeType === Node.ELEMENT_NODE
      ? range.endContainer
      : range.endContainer?.parentElement;
    const editable = this._closestEditable(startElement);
    if (!editable || this._closestEditable(endElement) !== editable) return null;
    const start = this._sourceOffsetFromDom(editable, range.startContainer, range.startOffset);
    const end = this._sourceOffsetFromDom(editable, range.endContainer, range.endOffset);
    if (start == null || end == null) return null;
    return {
      editable,
      selection: {
        start: Math.min(start, end),
        end: Math.max(start, end),
        direction: start <= end ? "forward" : "backward"
      },
      text: this._plainText(editable)
    };
  }
  _applyFallbackBeforeInput(event, inputTarget) {
    const inputType = event?.inputType || "";
    const selection = this._selection;
    const editable = inputTarget?.editable || this._fallbackEditable;
    let start = Math.min(selection.start, selection.end);
    let end = Math.max(selection.start, selection.end);
    let insert = "";
    if (inputType.startsWith("insert") && event.data != null) {
      insert = normalizeLineEndings(event.data);
    } else if (inputType.startsWith("delete")) {
      if (start === end && inputTarget?.selection.start !== inputTarget?.selection.end) {
        start = inputTarget.selection.start;
        end = inputTarget.selection.end;
      } else if (start === end && inputType.includes("Backward") && start > 0) {
        start -= Array.from(this._value.slice(0, start)).at(-1)?.length || 1;
      } else if (start === end && inputType.includes("Forward") && end < this._value.length) {
        end += Array.from(this._value.slice(end))[0]?.length || 1;
      }
      if (start === end) return false;
    } else {
      return false;
    }
    if (editable?.dataset.editable === "cell") {
      const raw = this._plainText(editable);
      const displayStart = this._displayOffsetFromSourceOffset(editable, start);
      const displayEnd = this._displayOffsetFromSourceOffset(editable, end);
      const nextRaw = raw.slice(0, displayStart) + insert + raw.slice(displayEnd);
      const tableEdit = this._tableCellInputEdit(
        editable,
        nextRaw,
        displayStart + insert.length
      );
      if (!tableEdit) return false;
      event.preventDefault();
      this._applyFallbackInput(
        inputType,
        diffTextChange(this._value, tableEdit.nextValue),
        { start: tableEdit.cursor, end: tableEdit.cursor, direction: "none" }
      );
      return true;
    }
    if (inputType.startsWith("insert")
      && ["virtual-code", "virtual-code-after", "virtual-hr-after", "virtual-setext-after", "virtual-table-after"].includes(editable?.dataset.editable)
      && !this._value.slice(0, start).endsWith("\n")) {
      insert = `\n${insert}`;
    }
    event.preventDefault();
    this._applyFallbackInput(inputType, [{ from: start, to: end, insert }], {
      start: start + insert.length,
      end: start + insert.length,
      direction: "none"
    });
    return true;
  }
  _applyFallbackInput(inputType, changes, selectionAfter) {
    const before = this._snapshot();
    const previousValue = this._value;
    this._value = applyTextChanges(previousValue, changes);
    this._selection = {
      start: clamp(selectionAfter.start, 0, this._value.length),
      end: clamp(selectionAfter.end, 0, this._value.length),
      direction: selectionAfter.direction || "none"
    };
    const after = makeSnapshot(
      this._value,
      this._selection.start,
      this._selection.end,
      this._selection.direction
    );
    this._recordUndo(before, after, this._undoGroupForInput(inputType), {
      coalesce: inputType === "insertText"
        || inputType === "deleteContentBackward"
        || inputType === "deleteContentForward"
    });
    this._redoStack.length = 0;
    this._afterValueChanged({
      source: "user",
      inputType,
      restoreSelection: true,
      previousValue,
      changes
    });
    if (!this._isComposing) this._scheduleCompletionUpdate();
  }
  _plainText(el) { return (el.textContent ?? "").replace(/\u00a0/g, " ").replace(/\n+$/g, ""); }
  _editableSourceRange(editable) {
    const from = Number(editable?.dataset?.from);
    const to = Number(editable?.dataset?.to);
    if (!Number.isFinite(from) || !Number.isFinite(to)) return null;
    return { from, to: Math.max(from, to) };
  }
  _cellRawSource(editable) {
    const range = this._editableSourceRange(editable);
    return range && editable?.dataset?.editable === "cell" ? this._value.slice(range.from, range.to) : null;
  }
  _displayOffsetFromSourceOffset(editable, sourceOffset) {
    const range = this._editableSourceRange(editable);
    if (!range) return 0;
    const raw = this._cellRawSource(editable);
    const rel = clamp(Number(sourceOffset) - range.from, 0, range.to - range.from);
    if (raw == null) return clamp(rel, 0, this._plainText(editable).length);
    let display = 0;
    for (let i = 0; i < raw.length && i < rel;) {
      if (raw[i] === "\\" && (raw[i + 1] === "|" || raw[i + 1] === "\\")) {
        if (rel <= i + 1) return display;
        display += 1;
        i += 2;
      } else {
        display += 1;
        i += 1;
      }
    }
    return clamp(display, 0, this._plainText(editable).length);
  }
  _sourceOffsetFromDisplayOffset(editable, displayOffset) {
    const range = this._editableSourceRange(editable);
    if (!range) return null;
    const raw = this._cellRawSource(editable);
    const target = clamp(Number(displayOffset) || 0, 0, this._plainText(editable).length);
    if (raw == null) return clamp(range.from + target, range.from, Math.max(range.to, range.from + this._plainText(editable).length));
    let display = 0;
    for (let i = 0; i < raw.length;) {
      if (display >= target) return range.from + i;
      if (raw[i] === "\\" && (raw[i + 1] === "|" || raw[i + 1] === "\\")) {
        if (display + 1 >= target) return range.from + i + 2;
        display += 1;
        i += 2;
      } else {
        if (display + 1 >= target) return range.from + i + 1;
        display += 1;
        i += 1;
      }
    }
    return range.to;
  }
  _closestEditable(target) { return target?.closest?.("[data-editable]") ?? null; }
  _fragmentIdForLink(link) {
    const href = link?.getAttribute?.("href")?.trim() ?? "";
    if (!href.startsWith("#") || href.length === 1) return "";
    try { return decodeURIComponent(href.slice(1)); } catch { return href.slice(1); }
  }
  _headingElementForId(surface, id) {
    return [...surface?.querySelectorAll?.(".md-heading[id], h1[id], h2[id], h3[id], h4[id]") || []]
      .find(heading => heading.id === id) || null;
  }
  _navigateFragmentLink(event, surface) {
    const link = event.target?.closest?.("a[href]");
    const id = this._fragmentIdForLink(link);
    if (!link || !surface?.contains(link) || !id) return false;
    let target = this._headingElementForId(surface, id);
    let label = target?.textContent?.trim() || "";
    if (surface === this._liveEditor) {
      const block = this._getBlocks().find(candidate => candidate.type === "heading" && candidate.heading?.id === id);
      if (!block) return false;
      label = block.heading.content;
      event.preventDefault();
      this.setSelectionRange(block.from, block.from, "none");
      target = this._headingElementForId(surface, id);
    } else {
      if (!target) return false;
      event.preventDefault();
    }
    target?.scrollIntoView?.({ block: "nearest", inline: "nearest" });
    this._announce(`Navigated to ${label || id}.`);
    return true;
  }

  _onLiveClick(event) {
    if (this._suppressLiveClick) {
      this._suppressLiveClick = false;
      event.preventDefault();
      event.stopPropagation();
      return;
    }
    if (this._navigateFragmentLink(event, this._liveEditor)) return;
    this._structuredSelection = null;
    const checkbox = event.target.closest?.("[data-task-checkbox]");
    if (checkbox) {
      event.preventDefault();
      const offset = Number(checkbox.dataset.checkOffset);
      const current = this._value[offset] || " ";
      const next = current.toLowerCase() === "x" ? " " : "x";
      const source = event.detail === 0 ? "keyboard" : "pointer";
      const ctx = this._getContext();
      const result = ok(
        tx(
          ctx,
          "block.taskDone",
          [{ from: offset, to: offset + 1, insert: next }],
          this._selection,
          "block"
        ),
        next === "x" ? "Task checked." : "Task unchecked."
      );
      this._applyActionResult("block.taskDone", result, { source });
      if (source === "keyboard") this._liveEditor.querySelector(`[data-task-checkbox][data-check-offset="${offset}"]`)?.focus();
      return;
    }
    this._onSelectionChanged();
  }

  _onLiveMouseDown(event) {
    if (this.disabled || this.readonly || this.mode === "source" || event.button !== 0 || event.detail > 1) return;
    if (event.target.closest?.("[data-task-checkbox]")) return;
    const anchor = this._sourceOffsetForClientPoint(event.clientX, event.clientY);
    if (anchor == null) return;
    const editable = this._liveEditableFromPoint(event.clientX, event.clientY);
    this._closeCompletion();
    if (this._liveSelectionAPI === false) {
      this._fallbackEditable = editable;
      this._fallbackSelectionPending = false;
      this._selection = { start: anchor, end: anchor, direction: "none" };
      this._pointerSelection = {
        anchor,
        focus: anchor,
        startX: event.clientX,
        startY: event.clientY,
        moved: false,
        native: true
      };
      this._bindPointerSelectionListeners();
      return;
    }
    event.preventDefault();
    this._pointerSelection = { anchor, focus: anchor, startX: event.clientX, startY: event.clientY, moved: false, native: false };
    this.setSelectionRange(anchor, anchor, "none");
    this._bindPointerSelectionListeners();
  }

  _bindPointerSelectionListeners() {
    this._boundLiveMouseMove ??= mouseEvent => this._onLiveMouseMove(mouseEvent);
    this._boundLiveMouseEnd ??= mouseEvent => this._onLiveMouseEnd(mouseEvent);
    const doc = this.ownerDocument || document;
    doc.addEventListener("mousemove", this._boundLiveMouseMove, true);
    doc.addEventListener("mouseup", this._boundLiveMouseEnd, true);
  }

  _onLiveMouseMove(event) {
    const state = this._pointerSelection;
    if (!state) return;
    const focus = this._sourceOffsetForClientPoint(event.clientX, event.clientY);
    if (focus == null) return;
    if (state.native) {
      state.focus = focus;
      state.moved = state.moved || Math.hypot(event.clientX - state.startX, event.clientY - state.startY) > 2;
      this._setFallbackPointerSelection(state.anchor, focus);
      this._updateSelectionToolbar();
      return;
    }
    event.preventDefault();
    state.focus = focus;
    state.moved = state.moved || Math.hypot(event.clientX - state.startX, event.clientY - state.startY) > 2;
    this._setLivePointerSelection(state.anchor, focus);
    this._updateSelectionToolbar();
  }

  _onLiveMouseEnd(event) {
    const state = this._pointerSelection;
    if (!state) return;
    const focus = this._sourceOffsetForClientPoint(event.clientX, event.clientY);
    if (focus != null) {
      if (state.native) this._setFallbackPointerSelection(state.anchor, focus);
      else this._setLivePointerSelection(state.anchor, focus);
    }
    this._suppressLiveClick = state.moved;
    if (this._suppressLiveClick) globalThis.setTimeout?.(() => { this._suppressLiveClick = false; }, 0);
    this._pointerSelection = null;
    const doc = this.ownerDocument || document;
    doc.removeEventListener("mousemove", this._boundLiveMouseMove, true);
    doc.removeEventListener("mouseup", this._boundLiveMouseEnd, true);
    if (!state.native) event.preventDefault();
    else {
      this._emitSelectionChange();
      if (!this._isComposing) this._scheduleCompletionUpdate();
    }
  }

  _setLivePointerSelection(anchor, focus) {
    const start = Math.min(anchor, focus);
    const end = Math.max(anchor, focus);
    const direction = anchor <= focus ? "forward" : "backward";
    this.setSelectionRange(start, end, direction);
  }

  _setFallbackPointerSelection(anchor, focus) {
    const start = Math.min(anchor, focus);
    const end = Math.max(anchor, focus);
    this._selection = {
      start,
      end,
      direction: anchor <= focus ? "forward" : "backward"
    };
  }

  _onNavigationKey(event) { if (NAVIGATION_KEYS.has(event.key)) this._onSelectionChanged(); }

  _maybeHandleLineBoundaryKey(event, activeCell = null, activeEditable = null) {
    if (event.defaultPrevented || event.altKey || event.ctrlKey || activeCell) return false;
    const isMac = /Mac|iPhone|iPad|iPod/.test(globalThis.navigator?.platform ?? "");
    let boundary = null;
    if (!event.metaKey && (event.key === "Home" || event.key === "End")) boundary = event.key === "Home" ? "start" : "end";
    else if (isMac && event.metaKey && (event.key === "ArrowLeft" || event.key === "ArrowRight")) boundary = event.key === "ArrowLeft" ? "start" : "end";
    if (!boundary) return false;
    const selection = this._getCurrentSelection();
    const focus = selection.direction === "backward" ? selection.start : selection.end;
    const anchor = selection.direction === "backward" ? selection.end : selection.start;
    const line = getLineRange(this._value, focus);
    const editableRange = !this._isSourceActive() ? this._editableSourceRange(activeEditable) : null;
    const target = boundary === "start" ? (editableRange?.from ?? line.start) : (editableRange?.to ?? line.end);
    event.preventDefault();
    if (event.shiftKey) this._setSourceBackedSelection(anchor, target);
    else this.setSelectionRange(target, target, "none");
    return true;
  }

  _maybeHandleLiveArrowKey(event, activeEditable = null) {
    if (this._isSourceActive() || event.defaultPrevented || event.altKey || event.metaKey || event.ctrlKey) return false;
    if (!LIVE_ARROW_KEYS.has(event.key)) return false;
    const editable = activeEditable || this._activeEditableFromEvent(event);
    if (!editable || editable.dataset.editable === "cell") return false;
    const selection = this._getLiveSelection() || this._getCurrentSelection();
    if (!selection) return false;
    if (selection.start !== selection.end && !event.shiftKey) {
      const target = (event.key === "ArrowLeft" || event.key === "ArrowUp") ? selection.start : selection.end;
      event.preventDefault();
      this.setSelectionRange(target, target, "none");
      return true;
    }
    if (event.shiftKey) return this._maybeExtendLiveArrowSelection(event, selection);
    if (selection.start !== selection.end) return false;
    const direction = (event.key === "ArrowLeft" || event.key === "ArrowUp") ? -1 : 1;
    let target = (event.key === "ArrowLeft" || event.key === "ArrowRight")
      ? this._horizontalArrowTarget(editable, selection.start, direction)
      : this._verticalArrowTarget(editable, selection.start, direction);
    if (target == null && this._liveSelectionAPI === false) {
      target = (event.key === "ArrowLeft" || event.key === "ArrowRight")
        ? this._fallbackHorizontalArrowTarget(editable, selection.start, direction)
        : this._fallbackVerticalArrowTarget(editable, selection.start, direction);
    }
    if (target == null || target === selection.start) return false;
    event.preventDefault();
    this.setSelectionRange(target, target, "none");
    return true;
  }

  _maybeExtendLiveArrowSelection(event, selection = this._getCurrentSelection()) {
    const focus = selection.direction === "backward" ? selection.start : selection.end;
    const anchor = selection.direction === "backward" ? selection.end : selection.start;
    const pos = this._domPositionFromSource(focus);
    const editable = pos?.editable;
    if (!editable || editable.dataset.editable === "cell") return false;
    const direction = (event.key === "ArrowLeft" || event.key === "ArrowUp") ? -1 : 1;
    let target = (event.key === "ArrowLeft" || event.key === "ArrowRight")
      ? this._horizontalArrowTarget(editable, focus, direction)
      : this._verticalArrowTarget(editable, focus, direction);
    if (target == null && this._liveSelectionAPI === false) {
      target = (event.key === "ArrowLeft" || event.key === "ArrowRight")
        ? this._fallbackHorizontalArrowTarget(editable, focus, direction)
        : this._fallbackVerticalArrowTarget(editable, focus, direction);
    }
    if (target == null || target === focus) return false;
    event.preventDefault();
    this._setSourceBackedSelection(anchor, target);
    return true;
  }

  _setSourceBackedSelection(anchor, focus) {
    const start = Math.min(anchor, focus);
    const end = Math.max(anchor, focus);
    const direction = anchor <= focus ? "forward" : "backward";
    this.setSelectionRange(start, end, direction);
  }

  _horizontalArrowTarget(editable, offset, direction) {
    const from = Number(editable.dataset.from);
    const to = Number(editable.dataset.to);
    if (!Number.isFinite(from) || !Number.isFinite(to)) return null;
    if (direction < 0 && offset <= from) {
      const previous = this._adjacentLiveEditable(editable, -1);
      return previous ? Number(previous.dataset.to) : null;
    }
    if (direction > 0 && offset >= to) {
      const next = this._adjacentLiveEditable(editable, 1);
      return next ? Number(next.dataset.from) : null;
    }
    return null;
  }

  _fallbackHorizontalArrowTarget(editable, offset, direction) {
    const from = Number(editable.dataset.from);
    const to = Number(editable.dataset.to);
    if (!Number.isFinite(from) || !Number.isFinite(to)) return null;
    if (direction < 0 && offset > from) {
      return offset - (Array.from(this._value.slice(from, offset)).at(-1)?.length || 1);
    }
    if (direction > 0 && offset < to) {
      return offset + (Array.from(this._value.slice(offset, to))[0]?.length || 1);
    }
    return null;
  }

  _fallbackVerticalArrowTarget(editable, offset, direction) {
    const caret = this._caretRectForSourceOffset(offset, editable);
    const box = editable.getBoundingClientRect();
    if (!caret || !box || box.height === 0) return null;
    const lineHeight = this._computedLineHeight(editable);
    const clientX = caret.left;
    const clientY = ((caret.top + caret.bottom) / 2) + direction * lineHeight;
    if (clientY <= box.top || clientY >= box.bottom) return null;
    const fromPoint = this._sourceOffsetFromPoint(editable, clientX, clientY);
    if (fromPoint != null) return fromPoint;
    return this._nearestSourceOffsetInEditable(editable, clientX, clientY);
  }

  _verticalArrowTarget(editable, offset, direction) {
    if (!this._isCaretOnVisualBoundary(editable, offset, direction)) return null;
    const targetEditable = this._adjacentLiveEditable(editable, direction);
    if (!targetEditable) return null;
    if (this._isSingleVisualRow(editable) && this._isSingleVisualRow(targetEditable)) {
      const sourceColumn = clamp(offset - Number(editable.dataset.from), 0, this._plainText(editable).length);
      return Number(targetEditable.dataset.from) + Math.min(sourceColumn, this._plainText(targetEditable).length);
    }
    const caretRect = this._caretRectForSourceOffset(offset, editable);
    const fallbackRect = editable.getBoundingClientRect();
    const clientX = caretRect?.left ?? fallbackRect.left;
    return this._sourceOffsetInEditableAtX(targetEditable, clientX, direction);
  }

  _isSingleVisualRow(editable) {
    const box = editable.getBoundingClientRect();
    if (!box || box.height === 0) return true;
    return box.height <= this._computedLineHeight(editable) * 1.65;
  }

  _rebuildLiveIndex() {
    const editables = [...this._liveEditor.querySelectorAll("[data-editable]")]
      .filter(el => Number.isFinite(Number(el.dataset.from)) && Number.isFinite(Number(el.dataset.to)))
      .sort((a, b) => Number(a.dataset.from) - Number(b.dataset.from));
    this._liveEditablesCache = editables;
    this._liveNavigationCache = editables.filter(el => el.dataset.editable !== "cell");
    this._liveIndexDirty = false;
  }
  _liveEditables() {
    if (this._liveIndexDirty) this._rebuildLiveIndex();
    return this._liveEditablesCache || [];
  }
  _liveNavigationEditables() {
    if (this._liveIndexDirty) this._rebuildLiveIndex();
    return this._liveNavigationCache || [];
  }

  _adjacentLiveEditable(editable, direction) {
    const editables = this._liveNavigationEditables();
    const index = editables.indexOf(editable);
    return index === -1 ? null : editables[index + direction] || null;
  }

  _computedLineHeight(el) {
    const style = globalThis.getComputedStyle?.(el);
    const parsed = Number.parseFloat(style?.lineHeight || "");
    if (Number.isFinite(parsed)) return parsed;
    const fontSize = Number.parseFloat(style?.fontSize || "");
    return Number.isFinite(fontSize) ? fontSize * 1.2 : 18;
  }

  _isCaretOnVisualBoundary(editable, offset, direction) {
    const from = Number(editable.dataset.from);
    const to = Number(editable.dataset.to);
    const rect = this._caretRectForSourceOffset(offset, editable);
    const box = editable.getBoundingClientRect();
    if (!rect || !box || box.height === 0) return direction < 0 ? offset <= from : offset >= to;
    const tolerance = this._computedLineHeight(editable) * 0.65;
    return direction < 0
      ? rect.top <= box.top + tolerance
      : rect.bottom >= box.bottom - tolerance;
  }

  _caretRectForSourceOffset(offset, preferredEditable = null) {
    const pos = preferredEditable
      ? this._textPositionInElement(preferredEditable, this._displayOffsetFromSourceOffset(preferredEditable, offset))
      : this._domPositionFromSource(offset);
    if (!pos) return null;
    return this._caretRectFromDomPosition(pos.node, pos.offset);
  }

  _caretRectFromDomPosition(node, offset) {
    const range = document.createRange();
    try { range.setStart(node, offset); } catch { return null; }
    range.collapse(true);
    const rect = range.getClientRects()[0] || range.getBoundingClientRect();
    return rect && Number.isFinite(rect.left) && (rect.height > 0 || rect.width > 0) ? rect : null;
  }

  _sourceOffsetInEditableAtX(editable, clientX, direction) {
    const from = Number(editable.dataset.from);
    const to = Number(editable.dataset.to);
    if (!Number.isFinite(from) || !Number.isFinite(to)) return null;
    const box = editable.getBoundingClientRect();
    if (!box || box.width === 0 || box.height === 0) return direction < 0 ? to : from;
    const lineHeight = this._computedLineHeight(editable);
    const x = clamp(clientX, box.left + 1, box.right - 1);
    const rowOffset = Math.min(Math.max(lineHeight / 2, 1), Math.max(box.height / 2, 1));
    const y = direction < 0 ? box.bottom - rowOffset : box.top + rowOffset;
    const fromPoint = this._sourceOffsetFromPoint(editable, x, y);
    if (fromPoint != null) return clamp(fromPoint, from, to);
    return this._nearestSourceOffsetInEditable(editable, x, y);
  }

  _sourceOffsetFromPoint(editable, clientX, clientY) {
    const doc = editable.ownerDocument || document;
    let node = null;
    let offset = 0;
    if (doc.caretPositionFromPoint) {
      try {
        const pos = doc.caretPositionFromPoint(clientX, clientY, { shadowRoots: [this._shadow] });
        if (pos) { node = pos.offsetNode; offset = pos.offset; }
      } catch {
        const pos = doc.caretPositionFromPoint(clientX, clientY);
        if (pos) { node = pos.offsetNode; offset = pos.offset; }
      }
    }
    if (!node && doc.caretRangeFromPoint) {
      const range = doc.caretRangeFromPoint(clientX, clientY);
      if (range) { node = range.startContainer; offset = range.startOffset; }
    }
    if (!node || (node !== editable && !editable.contains(node))) return null;
    return this._sourceOffsetFromDom(editable, node, offset);
  }

  _liveEditableFromPoint(clientX, clientY) {
    const direct = this._shadow.elementFromPoint?.(clientX, clientY);
    const directEditable = this._closestEditable(direct);
    if (directEditable) return directEditable;
    const editables = this._liveEditables();
    let best = null;
    let bestScore = Infinity;
    for (const el of editables) {
      const rect = el.getBoundingClientRect();
      if (!rect || rect.width === 0 || rect.height === 0) continue;
      const dx = clientX < rect.left ? rect.left - clientX : clientX > rect.right ? clientX - rect.right : 0;
      const dy = clientY < rect.top ? rect.top - clientY : clientY > rect.bottom ? clientY - rect.bottom : 0;
      const score = dy * 10000 + dx;
      if (score < bestScore) { bestScore = score; best = el; }
    }
    return best;
  }

  _sourceOffsetForClientPoint(clientX, clientY) {
    const editable = this._liveEditableFromPoint(clientX, clientY);
    if (!editable) return null;
    const rect = editable.getBoundingClientRect();
    if (!rect || rect.width === 0 || rect.height === 0) return Number(editable.dataset.from);
    const x = clamp(clientX, rect.left + 1, rect.right - 1);
    const y = clamp(clientY, rect.top + 1, rect.bottom - 1);
    const fromPoint = this._sourceOffsetFromPoint(editable, x, y);
    if (fromPoint != null) return clamp(fromPoint, Number(editable.dataset.from), Number(editable.dataset.to));
    return this._nearestSourceOffsetInEditable(editable, x, y);
  }

  _nearestSourceOffsetInEditable(editable, clientX, clientY) {
    const from = Number(editable.dataset.from);
    const length = this._plainText(editable).length;
    if (!length) return from;
    let bestOffset = 0;
    let bestScore = Infinity;
    for (let offset = 0; offset <= length; offset += 1) {
      const pos = this._textPositionInElement(editable, offset);
      const rect = this._caretRectFromDomPosition(pos.node, pos.offset);
      if (!rect) continue;
      const rowDistance = Math.abs(((rect.top + rect.bottom) / 2) - clientY);
      const columnDistance = Math.abs(rect.left - clientX);
      const score = (rowDistance * 1000) + columnDistance;
      if (score < bestScore) { bestScore = score; bestOffset = offset; }
    }
    return this._sourceOffsetFromDisplayOffset(editable, bestOffset) ?? from;
  }

  _onSelectionChanged() {
    if (this._ignoreSelectionChangeCount > 0) {
      this._ignoreSelectionChangeCount -= 1;
      this._emitSelectionChange();
      if (!this._isComposing) this._scheduleCompletionUpdate();
      this._updateSelectionToolbar();
      return;
    }
    this._structuredSelection = null;
    this._selection = this._getCurrentSelection();
    this._emitSelectionChange();
    if (!this._isComposing) this._scheduleCompletionUpdate();
    this._updateSelectionToolbar();
  }

  _hideSelectionToolbar() {
    if (this._selectionToolbar && !this._selectionToolbar.hidden) this._selectionToolbar.hidden = true;
  }

  // Show the floating formatting toolbar above a non-empty selection in the live
  // editor. Hidden in source mode, while composing, when the slash
  // completion popup is open, when the editor isn't focused, or with no selection.
  _updateSelectionToolbar() {
    if (!this._selectionToolbar) return;
    const sel = this._selection;
    const hasRange = sel && sel.start !== sel.end;
    const canShow = hasRange && !this.disabled && !this._isComposing
      && this.mode !== "source"
      && !this._completion.open && this._focusWithin;
    if (!canShow) { this._hideSelectionToolbar(); return; }
    const wasHidden = this._selectionToolbar.hidden;
    this._selectionToolbar.hidden = false;
    // First appearance jumps to place; later moves glide via CSS transition.
    if (wasHidden) {
      this._selectionToolbar.setAttribute("data-instant", "");
      this._positionSelectionToolbar();
      requestAnimationFrame(() => this._selectionToolbar?.removeAttribute("data-instant"));
      return;
    }
    this._positionSelectionToolbar();
  }

  _positionSelectionToolbar() {
    const shell = this._shadow.querySelector(".editor-shell");
    if (!shell || !this._selectionToolbar || this._selectionToolbar.hidden) return;
    const shellRect = shell.getBoundingClientRect();
    let rect = null;
    try { const sel = this._shadow.getSelection?.() || globalThis.getSelection?.(); if (sel?.rangeCount) rect = sel.getRangeAt(0).getBoundingClientRect(); } catch {}
    if (!rect || (!rect.width && !rect.height)) {
      const startRect = this._caretRectForSourceOffset(Math.min(this._selection.start, this._selection.end));
      const endRect = this._caretRectForSourceOffset(Math.max(this._selection.start, this._selection.end));
      if (startRect && endRect) rect = { left: Math.min(startRect.left, endRect.left), right: Math.max(startRect.right, endRect.right), top: Math.min(startRect.top, endRect.top), bottom: Math.max(startRect.bottom, endRect.bottom), width: 1, height: startRect.height };
    }
    if (!rect) { this._hideSelectionToolbar(); return; }
    const toolbarRect = this._selectionToolbar.getBoundingClientRect();
    const centerX = (rect.left + rect.right) / 2 - shellRect.left;
    let left = clamp(centerX - toolbarRect.width / 2, 4, Math.max(4, shellRect.width - toolbarRect.width - 4));
    // Prefer above the selection; drop below if there isn't room.
    let top = rect.top - shellRect.top - toolbarRect.height - 8;
    if (top < 4) top = rect.bottom - shellRect.top + 8;
    top = clamp(top, 4, Math.max(4, shellRect.height - toolbarRect.height - 4));
    this._selectionToolbar.style.left = `${left}px`;
    this._selectionToolbar.style.top = `${top}px`;
  }

  _isSourceActive() { return this._shadow.activeElement === this._sourceTextarea || this.mode === "source"; }
  _focusEditable(options) {
    if (this.disabled) return;
    if (this.mode === "source") { this._sourceTextarea?.focus(options); this._sourceTextarea?.setSelectionRange(this._selection.start, this._selection.end, this._selection.direction); return; }
    this._liveEditor?.focus(options); this._restoreLiveSelection(this._selection);
  }
  _getCurrentSelection() {
    if (this._isSourceActive() && this._sourceTextarea) return { start: this._sourceTextarea.selectionStart, end: this._sourceTextarea.selectionEnd, direction: this._sourceTextarea.selectionDirection || "none" };
    if (this._structuredSelection) return { ...this._structuredSelection };
    const live = this._getLiveSelection();
    return live || this._selection || { start: 0, end: 0, direction: "none" };
  }
  _getLiveSelection(preferredEditable = null) {
    const exposed = this._readLiveSelection(preferredEditable);
    if (exposed) return exposed;
    if (this._liveSelectionAPI !== false || !this._selection) return null;
    if (preferredEditable) {
      const range = this._editableSourceRange(preferredEditable);
      if (!range
        || this._selection.start < range.from
        || this._selection.end > range.to) return null;
    }
    return { ...this._selection };
  }
  _readLiveSelection(preferredEditable = null) {
    const sel = this._exposedLiveSelection();
    if (!sel) return null;
    const anchorEditable = preferredEditable || this._closestEditable(sel.anchorNode?.nodeType === Node.ELEMENT_NODE ? sel.anchorNode : sel.anchorNode?.parentElement);
    const focusEditable = preferredEditable || this._closestEditable(sel.focusNode?.nodeType === Node.ELEMENT_NODE ? sel.focusNode : sel.focusNode?.parentElement);
    if (!anchorEditable || !focusEditable) return null;
    if (preferredEditable
      && ((!preferredEditable.contains(sel.anchorNode) && preferredEditable !== sel.anchorNode)
        || (!preferredEditable.contains(sel.focusNode) && preferredEditable !== sel.focusNode))) return null;
    const start = this._sourceOffsetFromDom(anchorEditable, sel.anchorNode, sel.anchorOffset);
    const end = this._sourceOffsetFromDom(focusEditable, sel.focusNode, sel.focusOffset);
    if (start == null || end == null) return null;
    return { start: Math.min(start, end), end: Math.max(start, end), direction: start <= end ? "forward" : "backward" };
  }
  _exposedLiveSelection() {
    if (!this._liveEditor) return null;
    const selections = [];
    try {
      if (typeof this._shadow?.getSelection === "function") {
        selections.push(this._shadow.getSelection());
      }
    } catch {}
    try {
      const selection = globalThis.getSelection?.();
      if (selection && !selections.includes(selection)) selections.push(selection);
    } catch {}
    return selections.find(selection => {
      if (!selection || selection.rangeCount === 0) return false;
      const anchor = selection.anchorNode;
      const focus = selection.focusNode;
      return Boolean(
        anchor
        && focus
        && (anchor === this._liveEditor || this._liveEditor.contains(anchor))
        && (focus === this._liveEditor || this._liveEditor.contains(focus))
      );
    }) || null;
  }
  _displayOffsetFromSelection(editable) {
    const sel = this._exposedLiveSelection();
    const node = sel?.focusNode;
    if (!sel || sel.rangeCount === 0 || !node || (node !== editable && !editable.contains(node))) return null;
    const range = document.createRange();
    range.selectNodeContents(editable);
    try { range.setEnd(node, sel.focusOffset); } catch { return null; }
    return range.toString().replace(/\u00a0/g, " ").replace(/\n/g, "").length;
  }
  _sourceOffsetFromDom(editable, node, offset) {
    const from = Number(editable.dataset.from); if (!Number.isFinite(from)) return null;
    const range = document.createRange(); range.selectNodeContents(editable);
    try { range.setEnd(node, offset); } catch { return from; }
    const text = range.toString().replace(/\u00a0/g, " ").replace(/\n/g, "");
    return this._sourceOffsetFromDisplayOffset(editable, text.length) ?? from;
  }
  _restoreLiveSelection(selection = this._selection) {
    if (!this._liveEditor || this.mode === "source") return;
    if (this._virtualState.active && !this._ensureVirtualSelectionVisible(selection)) { this._liveEditor.focus(); return; }
    const startPos = this._domPositionFromSource(selection.start);
    const endPos = this._domPositionFromSource(selection.end);
    if (!startPos || !endPos) { this._liveEditor.focus(); return; }
    if (!this._isLiveDomPositionConnected(startPos) || !this._isLiveDomPositionConnected(endPos)) { this._liveEditor.focus(); return; }
    const focusEditable = selection.direction === "backward" ? startPos.editable : endPos.editable;
    const range = document.createRange();
    try {
      range.setStart(startPos.node, startPos.offset);
      range.setEnd(endPos.node, endPos.offset);
    } catch {
      this._liveEditor.focus();
      return;
    }
    let sel = null;
    try {
      sel = typeof this._shadow?.getSelection === "function"
        ? this._shadow.getSelection()
        : globalThis.getSelection?.();
    } catch {}
    if (!sel) {
      this._useFallbackLiveSelection(focusEditable || startPos.editable);
      return;
    }
    sel.removeAllRanges();
    try {
      if (selection.start !== selection.end && typeof sel.setBaseAndExtent === "function") {
        if (selection.direction === "backward") sel.setBaseAndExtent(endPos.node, endPos.offset, startPos.node, startPos.offset);
        else sel.setBaseAndExtent(startPos.node, startPos.offset, endPos.node, endPos.offset);
      } else {
        sel.addRange(range);
      }
    } catch {
      sel.removeAllRanges();
      try {
        if (this._isLiveDomPositionConnected(startPos) && this._isLiveDomPositionConnected(endPos)) sel.addRange(range);
      } catch {}
    }
    if (!this._readLiveSelection()) {
      this._useFallbackLiveSelection(focusEditable || startPos.editable);
      return;
    }
    this._liveSelectionAPI = true;
    this._fallbackEditable = null;
    this._fallbackSelectionPending = false;
    (focusEditable || startPos.editable)?.focus?.();
  }
  _useFallbackLiveSelection(editable) {
    this._liveSelectionAPI = false;
    this._fallbackEditable = editable || this._domPositionFromSource(this._selection.end)?.editable || null;
    this._fallbackSelectionPending = true;
    if (this._liveEditor) this._liveEditor.contentEditable = "false";
    const focusTarget = this._fallbackEditable || this._liveEditor;
    focusTarget?.blur();
    focusTarget?.focus();
  }
  _isLiveDomPositionConnected(pos) {
    return Boolean(pos?.node?.isConnected && pos?.editable?.isConnected && this._liveEditor?.contains(pos.editable));
  }
  _domPositionFromSource(offset) {
    const safe = clamp(offset, 0, this._value.length);
    if (this._virtualState.active && !this._isSourceOffsetRendered(safe)) {
      const blocks = this._liveBlocks?.length ? this._liveBlocks : this._getBlocks();
      this._renderLiveVirtual(blocks, { anchorOffset: safe, force: true });
      this._rebuildLiveIndex();
    }
    const editables = this._liveEditables();
    if (!editables.length) return null;
    let previous = null;
    for (const el of editables) {
      const from = Number(el.dataset.from);
      const to = Number(el.dataset.to);
      if (safe < from) {
        if (!previous) return this._textPositionInElement(el, 0);
        const prevTo = Number(previous.dataset.to);
        return (safe - prevTo <= from - safe)
          ? this._textPositionInElement(previous, this._plainText(previous).length)
          : this._textPositionInElement(el, 0);
      }
      if (safe >= from && safe <= to) return this._textPositionInElement(el, this._displayOffsetFromSourceOffset(el, safe));
      previous = el;
    }
    return this._textPositionInElement(previous, this._plainText(previous).length);
  }
  _textPositionInElement(el, offset) {
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
    let remaining = clamp(offset, 0, this._plainText(el).length);
    let lastText = null;
    while (walker.nextNode()) {
      const node = walker.currentNode; const len = node.nodeValue.length; lastText = node;
      if (remaining <= len) return { node, offset: remaining, editable: el };
      remaining -= len;
    }
    if (lastText) return { node: lastText, offset: lastText.nodeValue.length, editable: el };
    if (el.firstChild && el.firstChild.nodeName === "BR") return { node: el, offset: 0, editable: el };
    const text = document.createTextNode(""); el.appendChild(text); return { node: text, offset: 0, editable: el };
  }
  _ensureEmptyLiveEditable() {
    if (this._value.length !== 0 || !this._liveEditor) return;
    if (!this._liveEditor.querySelector("[data-editable]")) {
      const blocks = this._getBlocks();
      this._renderLiveFull(blocks);
      this._liveBlocks = blocks;
      this._rebuildLiveIndex();
    }
    this._selection = { start: 0, end: 0, direction: "none" };
    this._restoreLiveSelection(this._selection);
  }

  _snapshot() { const sel = this._getCurrentSelection(); this._selection = sel; return makeSnapshot(this._value, sel.start, sel.end, sel.direction || "none"); }
  _recordUndo(before, after, group, { coalesce = false } = {}) {
    if (!before || !after) return; if (before.value === after.value && sameSelection(before.selection, after.selection)) return;
    const latest = this._undoStack[this._undoStack.length - 1]; const timestamp = now();
    if (coalesce && latest && latest.group === group && timestamp - latest.timestamp < 900) { latest.after = after; latest.timestamp = timestamp; return; }
    this._undoStack.push({ before, after, group, timestamp }); if (this._undoStack.length > this._maxUndo) this._undoStack.shift();
  }
  _undoGroupForInput(inputType) { if (inputType?.startsWith("insert")) return "typing"; if (inputType?.startsWith("delete")) return "delete"; return inputType || "input"; }
  _undo() { const entry = this._undoStack.pop(); if (!entry) return false; const current = this._snapshot(); this._redoStack.push({ before: entry.before, after: current, group: entry.group, timestamp: now() }); this._restoreSnapshot(entry.before, "undo"); this._dispatch("md-action", { actionId: "history.undo", source: "keyboard", before: current, after: this._snapshot() }); return true; }
  _redo() { const entry = this._redoStack.pop(); if (!entry) return false; const current = this._snapshot(); this._undoStack.push({ before: current, after: entry.after, group: entry.group, timestamp: now() }); this._restoreSnapshot(entry.after, "redo"); this._dispatch("md-action", { actionId: "history.redo", source: "keyboard", before: current, after: this._snapshot() }); return true; }
  _restoreSnapshot(snapshot, source) {
    const previousValue = this._value;
    this._value = snapshot.value;
    this._selection = { ...snapshot.selection };
    this._redoStack = this._redoStack;
    this._afterValueChanged({ source, restoreSelection: true, previousValue, changes: diffTextChange(previousValue, this._value) });
  }

  _onKeyDown(event) {
    const isMac = /Mac|iPhone|iPad|iPod/.test(globalThis.navigator?.platform ?? "");
    const mod = isMac ? event.metaKey : event.ctrlKey;
    const taskCheckbox = event.target.closest?.("[data-task-checkbox]");
    if (!this._isSourceActive() && taskCheckbox && (event.key === " " || event.key === "Spacebar")) {
      event.preventDefault();
      taskCheckbox.click();
      return;
    }
    const activeEditable = this._activeEditableFromEvent(event);
    const activeCell = activeEditable?.dataset.editable === "cell" ? activeEditable : null;
    if (!this._isSourceActive() && NAVIGATION_KEYS.has(event.key)) this._ignoreSelectionChangeCount = 0;
    if (!this._isSourceActive() && this._value.length === 0 && (event.key === "Backspace" || event.key === "Delete")) {
      event.preventDefault();
      this._ensureEmptyLiveEditable();
      return;
    }

    if (mod && !event.altKey && event.key.toLowerCase() === "z") {
      if (this.readonly || this.disabled) return;
      event.preventDefault();
      event.shiftKey ? this._redo() : this._undo();
      return;
    }

    if (mod && !event.altKey && event.key.toLowerCase() === "a" && !this._isSourceActive()) {
      event.preventDefault();
      this._expandSelection();
      return;
    }

    if (this._isComposing) return;

    if (this._completion.open) {
      const map = { ArrowDown: "completion.next", ArrowUp: "completion.previous", Home: "completion.first", End: "completion.last", Escape: "completion.close" };
      if (map[event.key]) { event.preventDefault(); this._runAction(map[event.key], undefined, { source: "keyboard", apply: true }); return; }
      if (event.key === "PageDown") { event.preventDefault(); this._moveCompletion(5); return; }
      if (event.key === "PageUp") { event.preventDefault(); this._moveCompletion(-5); return; }
      if (event.key === "Enter" || (event.key === "Tab" && !event.shiftKey)) { event.preventDefault(); this._runAction("completion.accept", undefined, { source: "keyboard", apply: true }); return; }
    }

    if (this._maybeHandleTableLineBoundaryKey(event, activeCell)) return;
    if (this._maybeHandleLineBoundaryKey(event, activeCell, activeEditable)) return;
    if (activeCell && this._maybeHandleTableArrowKey(event, activeCell)) return;
    if (this._maybeHandleLiveArrowKey(event, activeEditable)) return;

    if ((event.key === "Backspace" || event.key === "Delete") && !this._isSourceActive()) {
      if (this.readonly || this.disabled) return;
      const selection = this._getCurrentSelection();
      if (selection.start !== selection.end) {
        event.preventDefault();
        const result = this._deleteSelectionResult(this._getContext(), event.key === "Delete" ? "editor.smartDelete" : "editor.smartBackspace");
        this._applyActionResult(event.key === "Delete" ? "editor.smartDelete" : "editor.smartBackspace", result, { source: "keyboard" });
        return;
      }
    }

    if (activeCell && (event.key === "Backspace" || event.key === "Delete")) {
      if (this.readonly || this.disabled) return;
      const result = this._deleteEmptyTableRowFromCellResult(activeCell);
      if (result?.ok && result.transaction) {
        event.preventDefault();
        this._applyActionResult("table.deleteRow", result, { source: "keyboard" });
        return;
      }
    }

    if (event.key === "Escape") {
      if (activeCell && !this._completion.open) {
        event.preventDefault();
        this._exitTable(activeCell, "after");
        return;
      }
      this._closeCompletion();
      return;
    }

    if (activeCell && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
      const direction = event.key === "ArrowDown" ? 1 : -1;
      if (this._maybeExitTableWithArrow(activeCell, direction)) {
        event.preventDefault();
        return;
      }
    }

    if (mod && !event.shiftKey && !event.altKey) {
      const key = event.key.toLowerCase();
      const map = { b: "inline.bold", i: "inline.italic", k: "inline.link", e: "inline.code" };
      if (map[key]) { event.preventDefault(); this._runAction(map[key], undefined, { source: "keyboard", apply: true }); return; }
    }
    if (mod && event.shiftKey && !event.altKey && event.key.toLowerCase() === "x") { event.preventDefault(); this._runAction("inline.strikethrough", undefined, { source: "keyboard", apply: true }); return; }
    if (mod && event.altKey && /^[1-6]$/.test(event.key)) { event.preventDefault(); this._runAction(`block.heading.${event.key}`, undefined, { source: "keyboard", apply: true }); return; }

    if (event.key === " " && !event.shiftKey && !event.altKey && !mod) {
      if (this.readonly || this.disabled) return;
      const result = this._runAction("editor.markdownShortcut", undefined, { source: "keyboard", apply: false });
      if (result?.ok && result.transaction) { event.preventDefault(); this._applyActionResult("editor.markdownShortcut", result, { source: "keyboard" }); return; }
    }

    if (event.key === "Delete") {
      if (this.readonly || this.disabled) return;
      const result = this._runAction("editor.smartDelete", undefined, { source: "keyboard", apply: false });
      if (result?.ok && result.transaction) { event.preventDefault(); this._applyActionResult("editor.smartDelete", result, { source: "keyboard" }); return; }
    }

    if (event.key === "Enter") {
      if (this.readonly || this.disabled) return;
      event.preventDefault();
      if (activeCell) {
        if (event.shiftKey || mod || event.altKey) this._exitTable(activeCell, "after");
        else this._insertTableRowAfterCell(activeCell);
        return;
      }
      const activeEditableType = activeEditable?.dataset.editable;
      if (activeEditableType === "virtual-code-after" || activeEditableType === "virtual-table-after") {
        this._runAction("editor.insertParagraph", undefined, { source: "keyboard", apply: true });
        return;
      }
      this._runAction(event.shiftKey ? "editor.insertSoftBreak" : "editor.smartEnter", undefined, { source: "keyboard", apply: true });
      return;
    }

    if (event.key === "Tab") {
      if (this.readonly || this.disabled) return;
      if (activeCell) { event.preventDefault(); this._handleTableTab(activeCell, event.shiftKey ? -1 : 1); return; }
      const id = event.shiftKey ? "editor.smartOutdent" : "editor.smartTab";
      const result = this._runAction(id, undefined, { source: "keyboard", apply: false });
      if (result?.ok && (result.transaction || result.preventDefault)) { event.preventDefault(); this._applyActionResult(id, result, { source: "keyboard" }); }
      return;
    }

    if (event.key === "Backspace") {
      if (this.readonly || this.disabled) return;
      const result = this._runAction("editor.smartBackspace", undefined, { source: "keyboard", apply: false });
      if (result?.ok && result.transaction) { event.preventDefault(); this._applyActionResult("editor.smartBackspace", result, { source: "keyboard" }); }
    }
  }

  _activeEditableFromEvent(event) {
    return this._closestEditable(event?.target) || this._activeEditableFromSelection();
  }

  _activeEditableFromSelection() {
    const sel = this._exposedLiveSelection();
    if (sel) {
      const node = sel.focusNode || sel.anchorNode;
      const editable = this._closestEditable(node?.nodeType === Node.ELEMENT_NODE ? node : node?.parentElement);
      if (editable) return editable;
    }
    if (this._liveSelectionAPI !== false) return null;
    const focus = this._selection.direction === "backward"
      ? this._selection.start
      : this._selection.end;
    return this._domPositionFromSource(focus)?.editable || this._fallbackEditable;
  }

  _findBlockAtOffset(offset, type = null) {
    const safe = clamp(Number(offset) || 0, 0, this._value.length);
    return this._getBlocks().find(block => (!type || block.type === type) && safe >= block.from && safe <= Math.max(block.to, block.from)) || null;
  }

  _findTableBlockForCell(cell) {
    const tableEl = cell?.closest?.(".md-table-block");
    if (!tableEl) return null;
    const from = Number(tableEl.dataset.from);
    const to = Number(tableEl.dataset.to);
    return this._getBlocks().find(block => block.type === "table" && block.from === from && block.to === to) || null;
  }

  _tableInfoForCell(cell) {
    const block = this._findTableBlockForCell(cell);
    if (!block) return null;
    const row = Number(cell.dataset.row);
    const col = Number(cell.dataset.col);
    const line = row < 0 ? block.header : block.rows[row];
    const cols = Math.max(block.header.cells.length, ...block.rows.map(r => r.cells.length), 1);
    return { block, row, col, line, cols };
  }

  _moveTableCell(cell, delta) {
    const cells = [...cell.closest(".md-table-block")?.querySelectorAll('[data-editable="cell"]') || []];
    const i = cells.indexOf(cell);
    if (i === -1) return false;
    const next = cells[i + delta];
    if (!next) return false;
    const from = Number(next.dataset.from);
    const text = this._plainText(next);
    this._selection = { start: from, end: from + text.length, direction: "none" };
    this._restoreLiveSelection(this._selection);
    this._announce("Table cell.");
    return true;
  }

  _handleTableTab(cell, delta) {
    if (this._moveTableCell(cell, delta)) return;
    if (delta < 0) {
      this._exitTable(cell, "before");
      return;
    }
    const info = this._tableInfoForCell(cell);
    if (!info) return;
    if (info.row >= 0 && this._isTableRowEmpty(info.line)) this._exitTable(cell, "after");
    else this._insertTableRowAfterCell(cell);
  }

  _maybeHandleTableLineBoundaryKey(event, activeCell = null) {
    if (this._isSourceActive() || event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) return false;
    if (event.key !== "Home" && event.key !== "End") return false;
    const selectedEditable = activeCell || this._activeEditableFromSelection();
    const cell = selectedEditable?.dataset?.editable === "cell" ? selectedEditable : null;
    if (!cell) return false;
    const range = this._editableSourceRange(cell);
    if (!range) return false;
    const selection = this._getLiveSelection() || this._getCurrentSelection();
    const focus = selection.direction === "backward" ? selection.start : selection.end;
    const anchor = selection.direction === "backward" ? selection.end : selection.start;
    const target = event.key === "Home" ? range.from : range.to;
    if (target === focus && !event.shiftKey) {
      event.preventDefault();
      return true;
    }
    event.preventDefault();
    if (event.shiftKey) this._setSourceBackedSelection(anchor, target);
    else this.setSelectionRange(target, target, "none");
    return true;
  }

  _maybeHandleTableArrowKey(event, cell) {
    if (this._isSourceActive() || event.defaultPrevented || event.altKey || event.metaKey || event.ctrlKey) return false;
    if (!LIVE_ARROW_KEYS.has(event.key)) return false;
    const selection = this._getLiveSelection() || this._getCurrentSelection();
    if (!selection) return false;
    if (selection.start !== selection.end && !event.shiftKey) {
      const target = (event.key === "ArrowLeft" || event.key === "ArrowUp") ? selection.start : selection.end;
      event.preventDefault();
      this.setSelectionRange(target, target, "none");
      return true;
    }
    const focus = selection.direction === "backward" ? selection.start : selection.end;
    const anchor = selection.direction === "backward" ? selection.end : selection.start;
    const direction = (event.key === "ArrowLeft" || event.key === "ArrowUp") ? -1 : 1;
    const target = (event.key === "ArrowLeft" || event.key === "ArrowRight")
      ? this._horizontalTableArrowTarget(cell, focus, direction)
      : this._verticalTableArrowTarget(cell, focus, direction);
    if (!target) return false;
    event.preventDefault();
    if (target.exit) {
      if (event.shiftKey) this._setSourceBackedSelection(anchor, target.offset);
      else this._exitTable(cell, target.exit);
      return true;
    }
    if (event.shiftKey) this._setSourceBackedSelection(anchor, target.offset);
    else this.setSelectionRange(target.offset, target.offset, "none");
    return true;
  }

  _tableCellsForCell(cell) {
    return [...cell.closest(".md-table-block")?.querySelectorAll('[data-editable="cell"]') || []];
  }

  _tableCellElement(block, row, col) {
    return this._liveEditor?.querySelector(`.md-table-block[data-from="${block.from}"][data-to="${block.to}"] .md-cell[data-row="${row}"][data-col="${col}"]`) || null;
  }

  _horizontalTableArrowTarget(cell, offset, direction) {
    const range = this._editableSourceRange(cell);
    if (!range) return null;
    if (direction < 0 && offset <= range.from) {
      const cells = this._tableCellsForCell(cell);
      const previous = cells[cells.indexOf(cell) - 1];
      if (previous) return { offset: Number(previous.dataset.to) };
      const info = this._tableInfoForCell(cell);
      return info?.block ? { exit: "before", offset: info.block.from } : null;
    }
    if (direction > 0 && offset >= range.to) {
      const cells = this._tableCellsForCell(cell);
      const next = cells[cells.indexOf(cell) + 1];
      if (next) return { offset: Number(next.dataset.from) };
      const info = this._tableInfoForCell(cell);
      return info?.block ? { exit: "after", offset: info.block.to } : null;
    }
    return null;
  }

  _verticalTableArrowTarget(cell, offset, direction) {
    const info = this._tableInfoForCell(cell);
    if (!info) return null;
    const nextRow = info.row + direction;
    if (nextRow < -1) return { exit: "before", offset: info.block.from };
    if (nextRow >= info.block.rows.length) return { exit: "after", offset: info.block.to };
    const targetCell = this._tableCellElement(info.block, nextRow, info.col);
    if (!targetCell) return null;
    const displayOffset = this._displayOffsetFromSourceOffset(cell, offset);
    const targetDisplayOffset = Math.min(displayOffset, this._plainText(targetCell).length);
    return { offset: this._sourceOffsetFromDisplayOffset(targetCell, targetDisplayOffset) ?? Number(targetCell.dataset.from) };
  }

  _maybeExitTableWithArrow(cell, direction) {
    const info = this._tableInfoForCell(cell);
    if (!info) return false;
    const atStart = this._isCellSelectionAtBoundary(cell, "start");
    const atEnd = this._isCellSelectionAtBoundary(cell, "end");
    if (direction < 0 && info.row < 0 && atStart) { this._exitTable(cell, "before"); return true; }
    if (direction > 0 && info.row === info.block.rows.length - 1 && atEnd) { this._exitTable(cell, "after"); return true; }
    return false;
  }

  _isCellSelectionAtBoundary(cell, boundary) {
    const sel = this._getLiveSelection(cell);
    if (!sel || sel.start !== sel.end) return false;
    const from = Number(cell.dataset.from);
    const to = Number(cell.dataset.to);
    return boundary === "start" ? sel.start <= from : sel.start >= to;
  }

  _isTableRowEmpty(line) {
    if (!line) return true;
    return splitTableRow(line.text).every(cell => cell.trim() === "");
  }

  _escapeTableCellText(cell) {
    return String(cell ?? "").replace(/\|/g, "\\|");
  }

  _tableRowSourceParts(cells) {
    const escaped = cells.map(cell => this._escapeTableCellText(cell));
    const offsets = [];
    let text = "| ";
    escaped.forEach((cell, index) => {
      offsets[index] = { from: text.length, to: text.length + cell.length };
      text += cell;
      text += index === escaped.length - 1 ? " |" : " | ";
    });
    return { text, offsets };
  }

  _tableBlockSourceWithOffsets(headerCells, delimiterCells, rows) {
    const parts = [
      this._tableRowSourceParts(headerCells),
      this._tableRowSourceParts(delimiterCells),
      ...rows.map(row => this._tableRowSourceParts(row)),
    ];
    const lines = [];
    const lineStarts = [];
    let cursor = 0;
    for (const part of parts) {
      lineStarts.push(cursor);
      lines.push(part.text);
      cursor += part.text.length + 1;
    }
    return { source: lines.join("\n"), parts, lineStarts };
  }

  _tableCellInputEdit(cell, raw, displayCursor = null) {
    const info = this._tableInfoForCell(cell);
    if (!info) return null;
    const cols = info.cols;
    const header = this._tableCellTexts(info.block.header, cols);
    const delimiter = this._tableCellTexts(info.block.delimiter, cols);
    const rows = info.block.rows.map(row => this._tableCellTexts(row, cols));
    if (info.row < 0) {
      header[info.col] = raw;
    } else {
      while (rows.length <= info.row) rows.push(Array.from({ length: cols }, () => ""));
      rows[info.row][info.col] = raw;
    }
    const serialized = this._tableBlockSourceWithOffsets(header, delimiter, rows);
    const lineIndex = info.row < 0 ? 0 : info.row + 2;
    const cellOffsets = serialized.parts[lineIndex]?.offsets?.[info.col];
    const rawCursor = clamp(displayCursor ?? raw.length, 0, raw.length);
    const escapedCursor = this._escapeTableCellText(raw.slice(0, rawCursor)).length;
    const cursor = info.block.from + (serialized.lineStarts[lineIndex] ?? 0) + (cellOffsets?.from ?? 0) + escapedCursor;
    return {
      nextValue: this._value.slice(0, info.block.from) + serialized.source + this._value.slice(info.block.to),
      cursor,
    };
  }

  _deleteEmptyTableRowFromCellResult(cell) {
    const info = this._tableInfoForCell(cell);
    if (!info || info.row < 0 || !info.line) return fail("not-applicable");
    const selection = this._getLiveSelection(cell);
    if (!selection || selection.start !== selection.end) return fail("not-applicable");
    if (this._plainText(cell).trim() || !this._isTableRowEmpty(info.line)) return fail("not-applicable");
    return this._tableDeleteRowResult(this._getContext(), info.block, info.row);
  }

  _insertTableRowAfterCell(cell) {
    const info = this._tableInfoForCell(cell);
    if (!info) return;
    const ctx = this._getContext();
    const result = this._tableRowInsertionResult(ctx, info.block, info.line, info.row < 0 ? "after-delimiter" : "after-row");
    this._applyActionResult("table.insertRowAfter", result, { source: "keyboard" });
  }

  _tableRowInsertionResult(ctx, block, line, placement = "after-row") {
    if (!block) return fail("not-applicable");
    const cols = Math.max(block.header.cells.length, ...block.rows.map(r => r.cells.length), 1);
    const insert = `\n| ${Array.from({ length: cols }, () => "").join(" | ")} |`;
    const insertionLine = placement === "after-delimiter" ? block.delimiter : (line || block.rows.at(-1) || block.delimiter);
    const from = insertionLine.end;
    const cursor = from + 3;
    return ok(tx(ctx, "table.insertRowAfter", [{ from, to: from, insert }], { start: cursor, end: cursor, direction: "none" }, "table"), "Table row inserted.");
  }

  _exitTable(cell, direction = "after") {
    const info = this._tableInfoForCell(cell);
    if (!info) return;
    const before = this._snapshot();
    const block = info.block;
    let changes = [];
    let cursor;
    if (direction === "before") {
      if (block.from === 0 || this._value[block.from - 1] !== "\n") {
        changes = [{ from: block.from, to: block.from, insert: "\n" }];
        cursor = block.from;
      } else {
        cursor = block.from;
      }
    } else {
      cursor = block.to < this._value.length ? block.to + 1 : block.to;
    }
    if (changes.length) {
      this._applyTransaction({ changes, selectionAfter: { start: cursor, end: cursor, direction: "none" }, actionId: "table.exit", undoGroup: "table", source: "keyboard" }, { source: "keyboard" });
    } else {
      this._selection = { start: cursor, end: cursor, direction: "none" };
      this._restoreLiveSelection(this._selection);
      this._emitSelectionChange();
    }
    const after = this._snapshot();
    this._dispatch("md-action", { actionId: "table.exit", source: "keyboard", before, after });
    this._announce(direction === "before" ? "Before table." : "After table.");
  }

  _expandSelection() {
    const current = this._getCurrentSelection();
    const candidates = this._selectionExpansionCandidates(current);
    const normalized = { start: Math.min(current.start, current.end), end: Math.max(current.start, current.end) };
    let next = candidates.find(range => (range.end > range.start || this._value.length === 0) && range.start <= normalized.start && range.end >= normalized.end && !this._sameRange(range, normalized));
    if (!next) next = { start: 0, end: this._value.length, label: "document" };
    this.setSelectionRange(next.start, next.end, "forward");
    this._announce(`Selected ${next.label || "content"}.`);
  }

  _selectionExpansionCandidates(selection) {
    const point = clamp(selection.start, 0, this._value.length);
    const out = [];
    const push = (start, end, label) => {
      const range = { start: clamp(start, 0, this._value.length), end: clamp(end, 0, this._value.length), label };
      if (range.end < range.start) [range.start, range.end] = [range.end, range.start];
      if (!out.some(existing => this._sameRange(existing, range))) out.push(range);
    };
    const active = this._activeEditableFromSelection();
    if (active?.dataset.editable === "cell") {
      const info = this._tableInfoForCell(active);
      const cellFrom = Number(active.dataset.from);
      const cellTo = Number(active.dataset.to);
      push(cellFrom, cellTo, "cell");
      if (info?.line) push(info.line.start, info.line.end, "row");
      if (info?.block) push(info.block.from, info.block.to, "table");
    }
    const block = this._findBlockAtOffset(point) || this._getBlocks()[0];
    if (block) push(block.from, block.to, block.type === "table" ? "table" : "block");
    const section = this._sectionRangeForOffset(point);
    if (section) push(section.start, section.end, "section");
    push(0, this._value.length, "document");
    return out.sort((a, b) => (a.end - a.start) - (b.end - b.start));
  }

  _sectionRangeForOffset(offset) {
    const blocks = this._getBlocks();
    let headingIndex = -1;
    let headingLevel = Infinity;
    for (let i = 0; i < blocks.length; i += 1) {
      const block = blocks[i];
      if (block.type === "heading" && block.from <= offset) {
        headingIndex = i;
        headingLevel = block.heading.level;
      }
      if (block.from > offset) break;
    }
    if (headingIndex === -1) return null;
    let end = this._value.length;
    for (let i = headingIndex + 1; i < blocks.length; i += 1) {
      const block = blocks[i];
      if (block.type === "heading" && block.heading.level <= headingLevel) { end = block.from; break; }
    }
    return { start: blocks[headingIndex].from, end, label: "section" };
  }

  _sameRange(a, b) {
    return a && b && a.start === b.start && a.end === b.end;
  }

  _clipboardRangeFromSelection(selection = this._getCurrentSelection()) {
    if (!selection || selection.start === selection.end) return null;
    const start = Math.min(selection.start, selection.end);
    const end = Math.max(selection.start, selection.end);
    return expandMarkdownFormattingRange(this._value, start, end);
  }
  _writeMarkdownClipboard(event, markdown) {
    event.clipboardData?.setData("text/plain", markdown);
    event.clipboardData?.setData("text/markdown", markdown);
    event.clipboardData?.setData("text/x-markdown", markdown);
    event.clipboardData?.setData("text/html", renderMarkdown(markdown, this._rendererOptions()));
  }
  _onLiveCopy(event) {
    if (this._isSourceActive()) return;
    const range = this._clipboardRangeFromSelection();
    if (!range) return;
    const markdown = this._value.slice(range.start, range.end);
    event.preventDefault();
    this._writeMarkdownClipboard(event, markdown);
    this._dispatch("md-copy", { markdown, start: range.start, end: range.end });
  }
  _onLiveCut(event) {
    if (this._isSourceActive() || this.disabled || this.readonly) return;
    const range = this._clipboardRangeFromSelection();
    if (!range) return;
    const markdown = this._value.slice(range.start, range.end);
    event.preventDefault();
    this._writeMarkdownClipboard(event, markdown);
    const ctx = this._getContext();
    const result = ok(tx(ctx, "editor.deleteSelection", [{ from: range.start, to: range.end, insert: "" }], { start: range.start, end: range.start, direction: "none" }, "cut"), "Cut.");
    this._applyActionResult("editor.deleteSelection", result, { source: "keyboard" });
    this._dispatch("md-cut", { markdown, start: range.start, end: range.end });
  }

  _serializeTableBlock(headerCells, delimiterCells, rows) {
    const serialize = cells => this._tableRowSourceParts(cells).text;
    return [serialize(headerCells), serialize(delimiterCells), ...rows.map(serialize)].join("\n");
  }
  _tableCellTexts(line, cols) {
    const cells = splitTableRow(line?.text ?? "");
    return Array.from({ length: cols }, (_, i) => unescapeTableCellText(cells[i] ?? ""));
  }
  _tablePositionForOffset(block, offset) {
    if (!block) return null;
    const safe = clamp(Number(offset) || 0, block.from, block.to);
    const lines = [
      { line: block.header, row: -1 },
      { line: block.delimiter, row: -2 },
      ...block.rows.map((line, row) => ({ line, row })),
    ];
    const match = lines.find(entry => safe >= entry.line.start && safe <= entry.line.end)
      || lines.find(entry => safe >= entry.line.start && safe <= entry.line.newlineEnd)
      || lines.at(-1);
    if (!match) return null;
    const cells = match.line.cells || [];
    let col = cells.findIndex(cell => safe >= cell.from && safe <= cell.to);
    if (col === -1 && cells.length) {
      let bestDistance = Infinity;
      cells.forEach((cell, index) => {
        const distance = safe < cell.from ? cell.from - safe : safe > cell.to ? safe - cell.to : 0;
        if (distance < bestDistance) { bestDistance = distance; col = index; }
      });
    }
    return { row: match.row, col: Math.max(0, col), line: match.line };
  }
  _tableColumnResult(ctx, block, col, mode) {
    const cols = Math.max(block.header.cells.length, ...block.rows.map(r => r.cells.length), 1);
    const target = clamp(Number(col) || 0, 0, cols - 1);
    const header = this._tableCellTexts(block.header, cols);
    const delimiter = this._tableCellTexts(block.delimiter, cols);
    const rows = block.rows.map(row => this._tableCellTexts(row, cols));
    if (mode === "delete") {
      if (cols <= 1) return fail("not-applicable", "Cannot delete the only column.");
      for (const list of [header, delimiter, ...rows]) list.splice(target, 1);
    } else {
      const at = target + 1;
      header.splice(at, 0, `Column ${cols + 1}`);
      delimiter.splice(at, 0, "---");
      for (const row of rows) row.splice(at, 0, "");
    }
    const insert = this._serializeTableBlock(header, delimiter, rows.length ? rows : [Array.from({ length: header.length }, () => "")]);
    const cursor = block.from + insert.split("\n")[0].length;
    return ok(tx(ctx, mode === "delete" ? "table.deleteColumn" : "table.insertColumnAfter", [{ from: block.from, to: block.to, insert }], { start: cursor, end: cursor, direction: "none" }, "table"), mode === "delete" ? "Column deleted." : "Column inserted.");
  }
  _tableDeleteRowResult(ctx, block, row) {
    if (!block.rows.length) return fail("not-applicable");
    const index = clamp(Number(row) || 0, 0, block.rows.length - 1);
    const line = block.rows[index];
    let from = line.start; let to = line.newlineEnd;
    if (to <= line.end && line.start > block.delimiter.end && ctx.value[line.start - 1] === "\n") {
      from = line.start - 1;
      to = line.end;
    } else if (to <= from) {
      to = line.end;
    }
    const cursor = from;
    return ok(tx(ctx, "table.deleteRow", [{ from, to, insert: "" }], { start: cursor, end: cursor, direction: "none" }, "table"), "Row deleted.");
  }

  _preparePastedMarkdown(markdown, kind = "text") {
    let insert = normalizeLineEndings(markdown ?? "");
    if (!insert) return insert;
    const sel = this._getCurrentSelection();
    if (sel.start !== sel.end) return insert;
    const shouldSeparate = kind === "markdown" || kind === "html" || kind === "table" || insert.includes("\n") || looksLikeBlockMarkdown(insert);
    if (!shouldSeparate) return insert;
    const line = getLineRange(this._value, sel.start);
    const beforeLine = this._value.slice(line.start, sel.start);
    const afterLine = this._value.slice(sel.start, line.end);
    if (beforeLine.trim() && !insert.startsWith("\n")) insert = "\n" + insert;
    if (afterLine.trim() && !insert.endsWith("\n")) insert = insert + "\n";
    return insert;
  }
  _insertPastedMarkdown(markdown, kind = "text") {
    const prepared = this._preparePastedMarkdown(markdown, kind);
    if (!prepared) return false;
    this._runAction("editor.insertText", { text: prepared }, { source: "paste", apply: true });
    this._dispatch("md-paste", { markdown: prepared, kind });
    return true;
  }
  _onPaste(event) {
    if (this.disabled || this.readonly) return;
    const clipboard = event.clipboardData || event.dataTransfer; if (!clipboard) return;
    const files = Array.from(clipboard.files || []);
    if (files.length > 0) { event.preventDefault(); const insertionPoint = this.selectionStart; this._dispatch("md-file-paste", { files, insertionPoint, insertMarkdown: markdown => this._insertPastedMarkdown(markdown, "file") }); return; }
    const text = safeClipboardGet(clipboard, "text/plain");
    if (text && this.selectionStart !== this.selectionEnd && isProbablyUrl(text) && !safeClipboardGet(clipboard, "text/markdown") && !safeClipboardGet(clipboard, "text/html")) { event.preventDefault(); this._runAction("inline.link", { url: text.trim() }, { source: "paste", apply: true }); return; }
    const { markdown, kind } = markdownFromClipboardData(clipboard);
    if (!markdown) return;
    event.preventDefault();
    this._insertPastedMarkdown(markdown, kind);
  }
  _onDrop(event) { if (this.disabled || this.readonly) return; const files = Array.from(event.dataTransfer?.files || []); if (!files.length) return; event.preventDefault(); const insertionPoint = this.selectionStart; this._dispatch("md-file-drop", { files, insertionPoint, insertMarkdown: markdown => this._insertPastedMarkdown(markdown, "file") }); }

  _getContext() {
    const sel = this._getCurrentSelection(); this._selection = sel;
    const parseOptions = this._parseOptions();
    const value = this._value; const line = getLineRange(value, sel.start); const currentLine = makeLineInfo(line.start, line.end, line.text, parseOptions); const selectedLines = getSelectedLineRanges(value, sel.start, sel.end, parseOptions); const block = classifyLine(value, sel.start, currentLine, parseOptions); const lineBeforeCursor = currentLine.text.slice(0, sel.start - currentLine.start);
    return { value, selectionStart: sel.start, selectionEnd: sel.end, selectionDirection: sel.direction || "none", mode: this.disabled ? "disabled" : this.readonly ? "readonly" : this._isComposing ? "composing-ime" : this._completion.open ? (this._completion.providerId === "slash" ? "slash-open" : "completion-open") : "idle", currentLine, selectedLines, block, inline: { insideInlineCode: isInsideInlineCode(lineBeforeCursor) }, completion: { ...this._completion }, config: { mode: this.mode, markdownFlavor: this.markdownFlavor, tabBehavior: this.tabBehavior, indentString: this.indentString, disabled: this.disabled, readonly: this.readonly }, host: this };
  }
  _runAction(actionId, args, options = {}) {
    const action = this._actions.get(actionId); if (!action) return fail("not-applicable", `Unknown action: ${actionId}`);
    const ctx = this._getContext(); if (ctx.mode === "disabled" && !action.viewSafe) return fail("disabled"); if (ctx.mode === "readonly" && !action.readonlySafe && !action.viewSafe) return fail("readonly"); if (ctx.mode === "composing-ime" && action.structural !== false) return fail("composition-active"); if (action.when && !action.when(ctx, args)) return fail("not-applicable");
    try { const result = action.run(ctx, args); if (options.apply === false) return result; return this._applyActionResult(actionId, result, options); } catch (error) { this._emitError("action", error, true, { actionId }); return fail("provider-error", String(error?.message || error)); }
  }
  _applyActionResult(actionId, result, options = {}) {
    if (!result?.ok) return result; const before = this._snapshot();
    if (result.transaction) { const t = { ...result.transaction, source: options.source || result.transaction.source || "api", actionId, timestamp: now() }; this._applyTransaction(t, { source: t.source }); const after = this._snapshot(); this._dispatch("md-action", { actionId, args: t.args, source: t.source, before, after }); }
    else this._dispatch("md-action", { actionId, source: options.source || "api", before, after: this._snapshot() });
    if (result.announcement) this._announce(result.announcement); return result;
  }
  _applyTransaction(transaction, options = {}) {
    const before = this._snapshot();
    const nextValue = applyTextChanges(this._value, transaction.changes);
    const proposedSelection = transaction.selectionAfter || { start: nextValue.length, end: nextValue.length, direction: "none" };
    const beforeEvent = this._dispatch("md-before-change", { transaction, before, nextValue, selectionAfter: proposedSelection, source: options.source || transaction.source || "api" }, { cancelable: true });
    if (beforeEvent.defaultPrevented) { this._announce("Change blocked."); return false; }
    this._value = nextValue;
    const sel = proposedSelection;
    this._selection = { start: clamp(sel.start, 0, nextValue.length), end: clamp(sel.end, 0, nextValue.length), direction: sel.direction || "none" }; this._structuredSelection = null;
    const after = makeSnapshot(this._value, this._selection.start, this._selection.end, this._selection.direction); this._recordUndo(before, after, transaction.undoGroup || transaction.actionId, { coalesce: false }); this._redoStack.length = 0; this._afterValueChanged({ source: options.source || transaction.source || "api", restoreSelection: true, previousValue: before.value, changes: transaction.changes }); this._scheduleCompletionUpdate();
    return true;
  }

  _installBuiltInActions() {
    const r = a => this.registerAction(a);
    r({ id: "editor.insertText", label: "Insert text", group: "Editor", structural: false, run: (ctx, args = {}) => insertionTransaction(ctx, "editor.insertText", normalizeLineEndings(args.text ?? ""), normalizeLineEndings(args.text ?? "").length, "insertText") });
    r({ id: "editor.replaceSelection", label: "Replace selection", group: "Editor", structural: false, run: (ctx, args = {}) => insertionTransaction(ctx, "editor.replaceSelection", normalizeLineEndings(args.text ?? ""), normalizeLineEndings(args.text ?? "").length, "replaceSelection") });
    r({ id: "editor.insertParagraph", label: "Insert paragraph", group: "Editor", defaultShortcut: "Enter", run: ctx => insertionTransaction(ctx, "editor.insertParagraph", "\n", 1, "insertParagraph") });
    r({ id: "editor.insertSoftBreak", label: "Insert soft break", group: "Editor", defaultShortcut: "Shift+Enter", run: ctx => insertionTransaction(ctx, "editor.insertSoftBreak", "  \n", 3, "insertSoftBreak") });
    r({ id: "editor.smartEnter", label: "Smart enter", group: "Editor", defaultShortcut: "Enter", run: ctx => this._smartEnter(ctx) });
    r({ id: "editor.smartTab", label: "Indent", group: "Editor", defaultShortcut: "Tab", run: ctx => this._smartTab(ctx) });
    r({ id: "editor.smartOutdent", label: "Outdent", group: "Editor", defaultShortcut: "Shift+Tab", run: ctx => this._smartOutdent(ctx) });
    r({ id: "editor.smartBackspace", label: "Smart backspace", group: "Editor", defaultShortcut: "Backspace", run: ctx => this._smartBackspace(ctx) });
    r({ id: "editor.smartDelete", label: "Smart delete", group: "Editor", defaultShortcut: "Delete", run: ctx => this._smartDelete(ctx) });
    r({ id: "editor.markdownShortcut", label: "Markdown shortcut", group: "Editor", defaultShortcut: "Space", run: ctx => this._markdownShortcut(ctx) });
    r({ id: "editor.deleteSelection", label: "Delete selection", group: "Editor", run: ctx => this._deleteSelectionResult(ctx, "editor.deleteSelection") });
    r({ id: "editor.selectAllExpand", label: "Expand selection", group: "Editor", defaultShortcut: "Mod+A", viewSafe: true, readonlySafe: true, run: () => { this._expandSelection(); return okNoop("Selection expanded."); } });
    r({ id: "history.undo", label: "Undo", group: "History", defaultShortcut: "Mod+Z", run: () => this._undo() ? okNoop("Undo.") : fail("not-applicable") });
    r({ id: "history.redo", label: "Redo", group: "History", defaultShortcut: "Mod+Shift+Z", run: () => this._redo() ? okNoop("Redo.") : fail("not-applicable") });
    // Slash-menu actions, ordered to match nonogra.ph's menu priority:
    // headings, inline formatting, link/image, block containers, secret, divider.
    for (let level = 1; level <= 4; level += 1) r({ id: `block.heading.${level}`, label: `Heading ${level}`, group: "Blocks", aliases: [`h${level}`, `heading${level}`], keywords: ["title", "section"], defaultShortcut: level <= 3 ? `Mod+Alt+${level}` : undefined, visibleInSlash: true, run: ctx => this._toggleHeading(ctx, level) });
    r({ id: "inline.bold", label: "Bold", group: "Inline", aliases: ["bold", "strong"], defaultShortcut: "Mod+B", visibleInSlash: true, run: ctx => this._wrapInline(ctx, "**", "**", "Bold") });
    r({ id: "inline.italic", label: "Italic", group: "Inline", aliases: ["italic", "em"], defaultShortcut: "Mod+I", visibleInSlash: true, run: ctx => this._wrapInline(ctx, "*", "*", "Italic") });
    r({ id: "inline.underline", label: "Underline", group: "Inline", aliases: ["underline", "u"], defaultShortcut: "Mod+U", visibleInSlash: true, run: ctx => this._wrapInline(ctx, "_", "_", "Underline") });
    r({ id: "inline.strikethrough", label: "Strikethrough", group: "Inline", aliases: ["strike", "s"], defaultShortcut: "Mod+Shift+X", visibleInSlash: true, run: ctx => this._wrapInline(ctx, "~", "~", "Strikethrough") });
    r({ id: "inline.superscript", label: "Superscript", group: "Inline", aliases: ["superscript", "sup"], visibleInSlash: true, run: ctx => this._wrapInline(ctx, "^", "^", "Superscript") });
    r({ id: "inline.highlight", label: "Highlight", group: "Inline", aliases: ["highlight", "mark"], defaultShortcut: "Mod+Shift+H", visibleInSlash: true, run: ctx => this._wrapInline(ctx, "==", "==", "Highlight") });
    r({ id: "inline.code", label: "Inline code", group: "Inline", aliases: ["inline-code", "codespan"], defaultShortcut: "Mod+E", visibleInSlash: true, run: ctx => this._wrapInline(ctx, "`", "`", "Inline code") });
    r({ id: "block.codeFence", label: "Code block", group: "Insert", aliases: ["code", "pre", "fence"], visibleInSlash: true, run: (ctx, args) => this._toggleCodeFence(ctx, args) });
    r({ id: "inline.link", label: "Link", group: "Inline", aliases: ["link", "url"], defaultShortcut: "Mod+K", visibleInSlash: true, run: (ctx, args = {}) => this._insertLink(ctx, args) });
    r({ id: "inline.image", label: "Image", group: "Inline", aliases: ["image", "img", "picture"], visibleInSlash: true, run: (ctx, args = {}) => this._insertImage(ctx, args) });
    r({ id: "block.blockquote", label: "Blockquote", group: "Blocks", aliases: ["quote", "blockquote"], visibleInSlash: true, run: ctx => this._toggleBlockquote(ctx) });
    r({ id: "block.bulletList", label: "Bullet list", group: "Blocks", aliases: ["bullet", "ul", "list"], visibleInSlash: true, run: ctx => this._toggleList(ctx, "bullet") });
    r({ id: "block.orderedList", label: "Numbered list", group: "Blocks", aliases: ["number", "numbered", "ol"], visibleInSlash: true, run: ctx => this._toggleList(ctx, "ordered") });
    r({ id: "block.table", label: "Table", group: "Insert", aliases: ["table", "grid"], visibleInSlash: true, run: (ctx, args = {}) => this._insertTable(ctx, args) });
    r({ id: "inline.secret", label: "Hidden text", group: "Inline", aliases: ["secret", "spoiler", "hidden"], visibleInSlash: true, run: ctx => this._wrapInline(ctx, "#", "#", "Hidden text") });
    r({ id: "block.horizontalRule", label: "Divider", group: "Insert", aliases: ["hr", "divider", "rule"], visibleInSlash: true, run: ctx => this._insertHorizontalRule(ctx) });
    // Available as actions/shortcuts but not surfaced in the slash menu (not part of nonograph's set).
    r({ id: "block.paragraph", label: "Paragraph", group: "Blocks", aliases: ["p", "text", "clear"], run: ctx => this._toggleParagraph(ctx) });
    r({ id: "block.taskList", label: "Task list", group: "Blocks", aliases: ["todo", "task", "checkbox"], run: ctx => this._toggleList(ctx, "task") });
    r({ id: "block.taskDone", label: "Toggle task done", group: "Blocks", aliases: ["done", "check"], run: ctx => this._toggleTaskDone(ctx) });    r({ id: "view.live", label: "Live mode", group: "View", viewSafe: true, readonlySafe: true, run: () => { this.mode = "live"; return okNoop("Live mode."); } });
    r({ id: "view.source", label: "Source mode", group: "View", viewSafe: true, readonlySafe: true, run: () => { this.mode = "source"; return okNoop("Source mode."); } });
    r({ id: "completion.close", label: "Close completion", group: "Completion", viewSafe: true, run: () => { this._closeCompletion(); return okNoop("Completion closed."); } });
    r({ id: "completion.next", label: "Next completion", group: "Completion", viewSafe: true, run: () => { this._moveCompletion(1); return okNoop(); } });
    r({ id: "completion.previous", label: "Previous completion", group: "Completion", viewSafe: true, run: () => { this._moveCompletion(-1); return okNoop(); } });
    r({ id: "completion.first", label: "First completion", group: "Completion", viewSafe: true, run: () => { this._setCompletionIndex(0, 1); return okNoop(); } });
    r({ id: "completion.last", label: "Last completion", group: "Completion", viewSafe: true, run: () => { this._setCompletionIndex(this._completion.items.length - 1, -1); return okNoop(); } });
    r({ id: "completion.accept", label: "Accept completion", group: "Completion", viewSafe: true, run: () => this._acceptCompletion("action") });
  }

  _installBuiltInProviders() {
    this.registerCompletionProvider({ id: "slash", priority: 100, triggers: ["/"], match: ctx => this._matchSlash(ctx), getItems: match => this._getSlashItems(match), apply: (item, match, ctx) => this._applySlashItem(item, match, ctx) });
    this.registerCompletionProvider({ id: "code-language", priority: 60, triggers: ["```", "~~~"], match: ctx => this._matchCodeLanguage(ctx), getItems: match => this._getLanguageItems(match), apply: (item, match, ctx) => { const insert = `${match.sequence || "```"}${item.label}`; const cursor = match.from + insert.length; return ok(tx(ctx, "completion.accept", [{ from: match.from, to: match.to, insert }], { start: cursor, end: cursor, direction: "none" }, "completion"), `Language ${item.label}.`); } });
  }
  _getLanguageItems(match) {
    const q = match.query.toLowerCase(); const alias = ALIASES.get(q);
    return LANGUAGES.map(lang => ({ lang, score: !q ? 0 : lang === q || lang === alias ? -100 : lang.startsWith(q) ? -50 : lang.includes(q) ? -10 : 0 })).filter(x => !q || x.score < 0).sort((a, b) => a.score - b.score || a.lang.localeCompare(b.lang)).slice(0, 16).map(x => ({ id: x.lang, label: x.lang, detail: "code language", kind: "code-language" }));
  }

  _smartEnter(ctx) {
    if (ctx.selectionStart !== ctx.selectionEnd) return insertionTransaction(ctx, "editor.smartEnter", "\n", 1, "smartEnter");
    const currentTextBeforeCursor = ctx.currentLine.text.slice(0, ctx.selectionStart - ctx.currentLine.start);
    const fenceInfo = getFenceInfo(ctx.currentLine.text);
    if (fenceInfo && currentTextBeforeCursor.trim().startsWith(fenceInfo.sequence) && !isInsideFence(ctx.value, ctx.selectionStart) && !hasClosingFenceAfter(ctx.value, ctx.currentLine.end, fenceInfo)) {
      return insertionTransaction(ctx, "editor.smartEnter", `\n\n${fenceInfo.sequence}`, 1, "smartEnter");
    }
    if (ctx.block.kind === "fenced-code") return insertionTransaction(ctx, "editor.smartEnter", "\n", 1, "smartEnter");
    const list = ctx.block.list;
    if (list) {
      if (list.content.trim() === "") return removePrefixFromLine(ctx, "editor.smartEnter", list.contentStart, "Exited list.");
      if (list.kind === "task-list-item") { const insert = `\n${list.indent}${list.marker} [ ] `; return insertionTransaction(ctx, "editor.smartEnter", insert, insert.length, "smartEnter"); }
      if (list.kind === "ordered-list-item") { const next = Number.isFinite(list.number) ? list.number + 1 : 1; const insert = `\n${list.indent}${next}${list.delimiter || "."} `; return insertionTransaction(ctx, "editor.smartEnter", insert, insert.length, "smartEnter"); }
      const insert = `\n${list.indent}${list.marker} `; return insertionTransaction(ctx, "editor.smartEnter", insert, insert.length, "smartEnter");
    }
    const quote = ctx.block.blockquote;
    if (quote) { if (quote.content.trim() === "") return removePrefixFromLine(ctx, "editor.smartEnter", quote.contentStart, "Exited blockquote."); const insert = `\n${quote.markerText}`; return insertionTransaction(ctx, "editor.smartEnter", insert, insert.length, "smartEnter"); }
    if (ctx.block.kind === "heading") return insertionTransaction(ctx, "editor.smartEnter", "\n", 1, "smartEnter");
    if (ctx.block.kind === "table") {
      const table = this._findBlockAtOffset(ctx.selectionStart, "table");
      if (table) {
        const placement = ctx.currentLine.start === table.header.start ? "after-delimiter" : "after-row";
        return this._tableRowInsertionResult(ctx, table, ctx.currentLine, placement);
      }
    }
    return insertionTransaction(ctx, "editor.smartEnter", "\n", 1, "smartEnter");
  }
  _smartTab(ctx) {
    if (ctx.completion?.open) return this._acceptCompletion("tab");
    const anyList = ctx.selectedLines.some(line => parseListItem(line.text, ctx.config)); if (anyList) return this._indentLines(ctx, ctx.config.indentString);
    if (ctx.block.kind === "fenced-code") return insertionTransaction(ctx, "editor.smartTab", ctx.config.indentString, ctx.config.indentString.length, "indent");
    if (ctx.config.tabBehavior === "editor-first") return insertionTransaction(ctx, "editor.smartTab", ctx.config.indentString, ctx.config.indentString.length, "indent");
    return fail("not-applicable", "Tab should move focus in accessibility-first mode.");
  }
  _smartOutdent(ctx) { const any = ctx.selectedLines.some(line => this._lineOutdentAmount(line.text) > 0); if (any) return this._outdentLines(ctx); return fail("not-applicable"); }
  _deleteSelectionResult(ctx, actionId = "editor.deleteSelection") {
    const start = Math.min(ctx.selectionStart, ctx.selectionEnd);
    const end = Math.max(ctx.selectionStart, ctx.selectionEnd);
    if (start === end) return fail("not-applicable");
    return ok(tx(ctx, actionId, [{ from: start, to: end, insert: "" }], { start, end: start, direction: "none" }, "delete"), "Deleted selection.");
  }
  _markdownShortcut(ctx) {
    if (ctx.selectionStart !== ctx.selectionEnd || ctx.inline.insideInlineCode || ctx.block.kind === "fenced-code") return fail("not-applicable");
    const before = ctx.currentLine.text.slice(0, ctx.selectionStart - ctx.currentLine.start);
    const after = ctx.currentLine.text.slice(ctx.selectionStart - ctx.currentLine.start);
    if (after.trim()) return fail("not-applicable");
    const task = /^(\s*)(\[\]|\[ \]|\[x\]|\[X\])$/.exec(before);
    if (task) {
      const checked = /x/i.test(task[2]) ? "x" : " ";
      const insert = `${task[1]}- [${checked}] `;
      const cursor = ctx.currentLine.start + insert.length;
      return ok(tx(ctx, "editor.markdownShortcut", [{ from: ctx.currentLine.start, to: ctx.selectionStart, insert }], { start: cursor, end: cursor, direction: "none" }, "markdownShortcut"), "Task list.");
    }
    const heading = /^(\s*)(#{1,4})$/.exec(before);
    if (heading) return insertionTransaction(ctx, "editor.markdownShortcut", " ", 1, "markdownShortcut");
    const bullet = /^(\s*)[-+*]$/.exec(before);
    if (bullet) return insertionTransaction(ctx, "editor.markdownShortcut", " ", 1, "markdownShortcut");
    const ordered = /^(\s*)\d+[.)]$/.exec(before);
    if (ordered) return insertionTransaction(ctx, "editor.markdownShortcut", " ", 1, "markdownShortcut");
    const quote = /^(\s*)>$/.exec(before);
    if (quote) return insertionTransaction(ctx, "editor.markdownShortcut", " ", 1, "markdownShortcut");
    return fail("not-applicable");
  }
  _smartDelete(ctx) {
    if (ctx.selectionStart !== ctx.selectionEnd) return this._deleteSelectionResult(ctx, "editor.smartDelete");
    const lineOffset = ctx.selectionStart - ctx.currentLine.start;
    if (lineOffset === ctx.currentLine.text.length && ctx.currentLine.end < ctx.value.length && ctx.value[ctx.currentLine.end] === "\n") {
      return ok(tx(ctx, "editor.smartDelete", [{ from: ctx.currentLine.end, to: ctx.currentLine.end + 1, insert: "" }], { start: ctx.currentLine.end, end: ctx.currentLine.end, direction: "none" }, "smartDelete"), "Joined line.");
    }
    return fail("not-applicable");
  }
  _smartBackspace(ctx) {
    if (ctx.selectionStart !== ctx.selectionEnd) return this._deleteSelectionResult(ctx, "editor.smartBackspace"); const lineOffset = ctx.selectionStart - ctx.currentLine.start; const list = ctx.block.list;
    if (list) { if (list.content.trim() === "" && lineOffset >= list.contentStart) return removePrefixFromLine(ctx, "editor.smartBackspace", list.contentStart, "Exited list."); if (lineOffset === list.contentStart) { const from = ctx.currentLine.start + list.fullMarkerStart; const to = ctx.currentLine.start + list.fullMarkerEnd; return ok(tx(ctx, "editor.smartBackspace", [{ from, to, insert: "" }], { start: from, end: from, direction: "none" }, "smartBackspace"), "Removed list marker."); } }
    const heading = ctx.block.heading; if (heading && lineOffset === heading.contentStart) return removePrefixFromLine(ctx, "editor.smartBackspace", heading.contentStart, "Converted to paragraph.");
    const quote = ctx.block.blockquote; if (quote && lineOffset === quote.contentStart) return removePrefixFromLine(ctx, "editor.smartBackspace", quote.contentStart, "Exited blockquote.");
    if (lineOffset === 0 && ctx.currentLine.start > 0 && ctx.value[ctx.currentLine.start - 1] === "\n") { const joinAt = ctx.currentLine.start - 1; return ok(tx(ctx, "editor.smartBackspace", [{ from: joinAt, to: ctx.currentLine.start, insert: "" }], { start: joinAt, end: joinAt, direction: "none" }, "smartBackspace"), "Joined line."); }
    if (lineOffset > 0 && /^\s+$/.test(ctx.currentLine.text.slice(0, lineOffset))) { const amount = this._lineOutdentAmount(ctx.currentLine.text.slice(0, lineOffset)); if (amount > 0) { const from = ctx.selectionStart - amount; return ok(tx(ctx, "editor.smartBackspace", [{ from, to: ctx.selectionStart, insert: "" }], { start: from, end: from, direction: "none" }, "smartBackspace")); } }
    return fail("not-applicable");
  }
  _lineOutdentAmount(text) { if (text.startsWith("\t")) return 1; const indent = (text.match(/^ +/) || [""])[0].length; if (indent >= this.indentString.length && this.indentString !== "\t") return this.indentString.length; if (indent >= 4) return 4; if (indent >= 2) return 2; if (indent >= 1) return 1; return 0; }
  _indentLines(ctx, indent) { const changes = []; let ds = 0; let de = 0; for (const line of ctx.selectedLines) { if (!parseListItem(line.text, ctx.config) && ctx.block.kind !== "fenced-code") continue; changes.push({ from: line.start, to: line.start, insert: indent }); if (line.start < ctx.selectionStart) ds += indent.length; if (line.start < ctx.selectionEnd || ctx.selectionStart === ctx.selectionEnd) de += indent.length; } if (!changes.length) return fail("not-applicable"); return ok(tx(ctx, "editor.smartTab", changes, { start: ctx.selectionStart + ds, end: ctx.selectionEnd + de, direction: ctx.selectionDirection || "none" }, "indent"), "Indented."); }
  _outdentLines(ctx) { const changes = []; let ds = 0; let de = 0; for (const line of ctx.selectedLines) { const amount = this._lineOutdentAmount(line.text); if (amount <= 0) continue; changes.push({ from: line.start, to: line.start + amount, insert: "" }); if (line.start < ctx.selectionStart) ds -= amount; if (line.start < ctx.selectionEnd || ctx.selectionStart === ctx.selectionEnd) de -= amount; } if (!changes.length) return fail("not-applicable"); const base = ctx.selectedLines[0]?.start ?? 0; return ok(tx(ctx, "editor.smartOutdent", changes, { start: Math.max(base, ctx.selectionStart + ds), end: Math.max(base, ctx.selectionEnd + de), direction: ctx.selectionDirection || "none" }, "outdent"), "Outdented."); }

  _toggleParagraph(ctx) { const changes = []; for (const line of ctx.selectedLines) { const list = parseListItem(line.text, ctx.config); const heading = parseHeading(line.text); const quote = parseBlockquote(line.text); if (list) changes.push({ from: line.start + list.fullMarkerStart, to: line.start + list.fullMarkerEnd, insert: "" }); else if (heading) changes.push({ from: line.start, to: line.start + heading.contentStart, insert: heading.indent }); else if (quote) changes.push({ from: line.start, to: line.start + quote.contentStart, insert: "" }); } if (!changes.length) return fail("not-applicable"); const d = changes.reduce((sum, c) => c.from < ctx.selectionStart ? sum + c.insert.length - (c.to - c.from) : sum, 0); return ok(tx(ctx, "block.paragraph", changes, { start: Math.max(0, ctx.selectionStart + d), end: Math.max(0, ctx.selectionEnd + d), direction: ctx.selectionDirection || "none" }, "block"), "Converted to paragraph."); }
  _toggleHeading(ctx, level) { const marker = `${"#".repeat(level)} `; const lines = ctx.selectedLines; const allSame = lines.every(line => { const h = parseHeading(line.text); return h && h.level === level; }); const changes = []; for (const line of lines) { const h = parseHeading(line.text); const list = parseListItem(line.text, ctx.config); const quote = parseBlockquote(line.text); if (allSame && h) changes.push({ from: line.start, to: line.start + h.contentStart, insert: h.indent }); else if (h) changes.push({ from: line.start, to: line.start + h.contentStart, insert: h.indent + marker }); else if (list) changes.push({ from: line.start + list.fullMarkerStart, to: line.start + list.fullMarkerEnd, insert: marker }); else if (quote) changes.push({ from: line.start, to: line.start + quote.contentStart, insert: marker }); else changes.push({ from: line.start, to: line.start, insert: marker }); } let ds = 0; let de = 0; for (const c of changes) { const diff = c.insert.length - (c.to - c.from); if (c.from < ctx.selectionStart) ds += diff; if (c.from < ctx.selectionEnd || ctx.selectionStart === ctx.selectionEnd) de += diff; } return ok(tx(ctx, `block.heading.${level}`, changes, { start: Math.max(0, ctx.selectionStart + ds), end: Math.max(0, ctx.selectionEnd + de), direction: ctx.selectionDirection || "none" }, "block"), allSame ? "Converted to paragraph." : `Heading level ${level}.`); }
  _toggleList(ctx, type) { const markerFor = i => type === "ordered" ? `${i + 1}. ` : type === "task" ? "- [ ] " : "- "; const lines = ctx.selectedLines; const allList = lines.every(line => parseListItem(line.text, ctx.config)); const changes = []; lines.forEach((line, i) => { const list = parseListItem(line.text, ctx.config); const h = parseHeading(line.text); const quote = parseBlockquote(line.text); let c; if (allList && list) c = { from: line.start + list.fullMarkerStart, to: line.start + list.fullMarkerEnd, insert: "" }; else if (list) c = { from: line.start + list.fullMarkerStart, to: line.start + list.fullMarkerEnd, insert: markerFor(i) }; else if (h) c = { from: line.start, to: line.start + h.contentStart, insert: h.indent + markerFor(i) }; else if (quote) c = { from: line.start, to: line.start + quote.contentStart, insert: markerFor(i) }; else { const indent = (line.text.match(/^\s*/) || [""])[0]; c = { from: line.start + indent.length, to: line.start + indent.length, insert: markerFor(i) }; } changes.push(c); }); let ds = 0; let de = 0; for (const c of changes) { const diff = c.insert.length - (c.to - c.from); if (c.from < ctx.selectionStart) ds += diff; if (c.from < ctx.selectionEnd || ctx.selectionStart === ctx.selectionEnd) de += diff; } const id = `block.${type === "bullet" ? "bulletList" : type === "ordered" ? "orderedList" : "taskList"}`; return ok(tx(ctx, id, changes, { start: Math.max(0, ctx.selectionStart + ds), end: Math.max(0, ctx.selectionEnd + de), direction: ctx.selectionDirection || "none" }, "block"), allList ? "Removed list." : type === "ordered" ? "Numbered list." : type === "task" ? "Task list." : "Bullet list."); }
  _toggleTaskDone(ctx) { const list = ctx.block.list; if (!list || list.kind !== "task-list-item") return fail("not-applicable"); const checkboxStart = ctx.currentLine.start + list.indent.length + `${list.marker} [`.length; const next = list.checked ? " " : "x"; return ok(tx(ctx, "block.taskDone", [{ from: checkboxStart, to: checkboxStart + 1, insert: next }], { start: ctx.selectionStart, end: ctx.selectionEnd, direction: ctx.selectionDirection || "none" }, "block"), next === "x" ? "Task checked." : "Task unchecked."); }
  _toggleBlockquote(ctx) { const lines = ctx.selectedLines; const allQuote = lines.every(line => parseBlockquote(line.text)); const changes = lines.map(line => { const quote = parseBlockquote(line.text); return allQuote && quote ? { from: line.start, to: line.start + quote.contentStart, insert: "" } : { from: line.start, to: line.start, insert: "> " }; }); let ds = 0; let de = 0; for (const c of changes) { const diff = c.insert.length - (c.to - c.from); if (c.from < ctx.selectionStart) ds += diff; if (c.from < ctx.selectionEnd || ctx.selectionStart === ctx.selectionEnd) de += diff; } return ok(tx(ctx, "block.blockquote", changes, { start: Math.max(0, ctx.selectionStart + ds), end: Math.max(0, ctx.selectionEnd + de), direction: ctx.selectionDirection || "none" }, "block"), allQuote ? "Removed blockquote." : "Blockquote."); }
  _setCodeLanguageResult(ctx, block, language) {
    const opening = block?.opening;
    if (!opening) return fail("not-applicable");
    const clean = String(language ?? "").replace(/[`~\\\r\n]/g, "").trim().replace(/\s+/g, "-");
    const match = /^(\s*(?:`{3,}|~{3,}))\s*([^`~]*)$/.exec(opening.text);
    const prefix = match?.[1] ?? "```";
    const insert = `${prefix}${clean}`;
    const cursor = opening.start + insert.length;
    return ok(tx(ctx, "code.setLanguage", [{ from: opening.start, to: opening.end, insert }], { start: cursor, end: cursor, direction: "none" }, "code"), clean ? `Language ${clean}.` : "Language cleared.");
  }
  _toggleCodeFence(ctx, args = {}) { const language = String(args.language ?? "").trim(); const langPart = language ? language : ""; if (ctx.selectionStart !== ctx.selectionEnd) { const selected = ctx.value.slice(ctx.selectionStart, ctx.selectionEnd); const insert = `\`\`\`${langPart}\n${selected}\n\`\`\``; const cursor = ctx.selectionStart + 4 + langPart.length + selected.length; return ok(tx(ctx, "block.codeFence", [{ from: ctx.selectionStart, to: ctx.selectionEnd, insert }], { start: cursor, end: cursor, direction: "none" }, "block"), "Code block."); } const insert = `\`\`\`${langPart}\n\n\`\`\``; const cursor = ctx.selectionStart + 4 + langPart.length; return ok(tx(ctx, "block.codeFence", [{ from: ctx.selectionStart, to: ctx.selectionEnd, insert }], { start: cursor, end: cursor, direction: "none" }, "block"), "Code block."); }
  _insertHorizontalRule(ctx) { const lead = ctx.selectionStart > 0 && ctx.value[ctx.selectionStart - 1] !== "\n" ? "\n" : ""; const trail = ctx.selectionStart < ctx.value.length && ctx.value[ctx.selectionStart] !== "\n" ? "\n" : "\n"; const insert = `${lead}---${trail}`; return insertionTransaction(ctx, "block.horizontalRule", insert, insert.length, "block"); }
  _insertTable(ctx, args = {}) { const rows = clamp(Number(args.rows) || 2, 1, 20); const cols = clamp(Number(args.cols) || 3, 2, 12); const header = `| ${Array.from({ length: cols }, (_, i) => `Column ${i + 1}`).join(" | ")} |`; const delimiter = `| ${Array.from({ length: cols }, () => "---").join(" | ")} |`; const body = Array.from({ length: rows }, (_, r) => `| ${Array.from({ length: cols }, (_, c) => `Cell ${r * cols + c + 1}`).join(" | ")} |`); const insert = [header, delimiter, ...body].join("\n"); const cursor = ctx.selectionStart + header.indexOf("Column 1"); return ok(tx(ctx, "block.table", [{ from: ctx.selectionStart, to: ctx.selectionEnd, insert }], { start: cursor, end: cursor + "Column 1".length, direction: "none" }, "block"), "Table inserted."); }
  _wrapInline(ctx, prefix, suffix, label) { const selected = ctx.value.slice(ctx.selectionStart, ctx.selectionEnd); if (selected) { const insert = `${prefix}${selected}${suffix}`; const cursor = ctx.selectionStart + insert.length; return ok(tx(ctx, `inline.${label.toLowerCase().replace(/\s+/g, "")}`, [{ from: ctx.selectionStart, to: ctx.selectionEnd, insert }], { start: cursor, end: cursor, direction: "none" }, "inline"), `${label}.`); } const insert = `${prefix}${suffix}`; const cursor = ctx.selectionStart + prefix.length; return ok(tx(ctx, `inline.${label.toLowerCase().replace(/\s+/g, "")}`, [{ from: ctx.selectionStart, to: ctx.selectionEnd, insert }], { start: cursor, end: cursor, direction: "none" }, "inline"), `${label}.`); }
  _insertLink(ctx, args = {}) { const selected = ctx.value.slice(ctx.selectionStart, ctx.selectionEnd); const url = args.url ?? ""; if (selected) { const insert = `[${selected}](${url})`; const cursor = url ? ctx.selectionStart + insert.length : ctx.selectionStart + selected.length + 3; return ok(tx(ctx, "inline.link", [{ from: ctx.selectionStart, to: ctx.selectionEnd, insert }], { start: cursor, end: cursor, direction: "none" }, "inline"), "Link."); } const insert = url ? `[](${url})` : `[]()`; return ok(tx(ctx, "inline.link", [{ from: ctx.selectionStart, to: ctx.selectionEnd, insert }], { start: ctx.selectionStart + 1, end: ctx.selectionStart + 1, direction: "none" }, "inline"), "Link."); }
  _insertImage(ctx, args = {}) { const alt = args.alt ?? ""; const src = args.src ?? ""; const insert = `![${alt}](${src})`; const cursor = alt ? ctx.selectionStart + insert.length : ctx.selectionStart + 2; return ok(tx(ctx, "inline.image", [{ from: ctx.selectionStart, to: ctx.selectionEnd, insert }], { start: cursor, end: cursor, direction: "none" }, "inline"), "Image."); }

  _matchSlash(ctx) { if (ctx.block.kind === "fenced-code" || ctx.inline.insideInlineCode) return null; const before = ctx.currentLine.text.slice(0, ctx.selectionStart - ctx.currentLine.start); const m = /^(\s*)\/([\w-]*)$/.exec(before); if (!m) return null; return { from: ctx.currentLine.start + m[1].length, to: ctx.selectionStart, trigger: "/", query: m[2], providerId: "slash" }; }
  _getSlashItems(match) { const q = match.query.toLowerCase(); const items = []; for (const action of this._actions.values()) { if (!action.visibleInSlash) continue; const hay = [action.label, action.description, ...(action.aliases || []), ...(action.keywords || [])].filter(Boolean).join(" ").toLowerCase(); if (q && !hay.includes(q)) continue; items.push({ id: action.id, label: action.label, detail: displayShortcut(action.defaultShortcut), description: "", kind: "slash-command", actionId: action.id }); } return items.slice(0, 24); }
  _applySlashItem(item, match, ctx) { const repl = this._slashReplacementForAction(item.actionId); if (repl) { const insert = typeof repl.insert === "function" ? repl.insert(ctx) : repl.insert; const off = typeof repl.selectionOffset === "number" ? repl.selectionOffset : insert.length; return ok(tx(ctx, "completion.accept", [{ from: match.from, to: match.to, insert }], { start: match.from + off, end: match.from + off + (repl.selectionLength || 0), direction: "none" }, "slash"), item.label); } return ok(tx(ctx, "completion.accept", [{ from: match.from, to: match.to, insert: "" }], { start: match.from, end: match.from, direction: "none" }, "slash"), item.label); }
  _slashReplacementForAction(actionId) { return { "block.paragraph": { insert: "", selectionOffset: 0 }, "block.heading.1": { insert: "# ", selectionOffset: 2 }, "block.heading.2": { insert: "## ", selectionOffset: 3 }, "block.heading.3": { insert: "### ", selectionOffset: 4 }, "block.heading.4": { insert: "#### ", selectionOffset: 5 }, "block.bulletList": { insert: "- ", selectionOffset: 2 }, "block.orderedList": { insert: "1. ", selectionOffset: 3 }, "block.taskList": { insert: "- [ ] ", selectionOffset: 6 }, "block.blockquote": { insert: "> ", selectionOffset: 2 }, "block.codeFence": { insert: "```\n\n```", selectionOffset: 4 }, "block.horizontalRule": { insert: "---\n", selectionOffset: 4 }, "block.table": { insert: "| Column 1 | Column 2 | Column 3 |\n| --- | --- | --- |\n| Cell 1 | Cell 2 | Cell 3 |", selectionOffset: 2, selectionLength: "Column 1".length }, "inline.link": { insert: "[]()", selectionOffset: 3 }, "inline.image": { insert: "![]()", selectionOffset: 4 }, "inline.bold": { insert: "****", selectionOffset: 2 }, "inline.italic": { insert: "**", selectionOffset: 1 }, "inline.code": { insert: "``", selectionOffset: 1 }, "inline.underline": { insert: "__", selectionOffset: 1 }, "inline.strikethrough": { insert: "~~", selectionOffset: 1 }, "inline.superscript": { insert: "^^", selectionOffset: 1 }, "inline.highlight": { insert: "====", selectionOffset: 2 }, "inline.secret": { insert: "##", selectionOffset: 1 } }[actionId] || null; }
  _matchCodeLanguage(ctx) { if (ctx.block.kind === "fenced-code") return null; const before = ctx.currentLine.text.slice(0, ctx.selectionStart - ctx.currentLine.start); const m = /^(\s*)(`{3,}|~{3,})([\w+-]*)$/.exec(before); if (!m || LANGUAGES.includes(m[3].toLowerCase())) return null; return { from: ctx.currentLine.start + m[1].length, to: ctx.selectionStart, trigger: m[2], sequence: m[2], query: m[3], providerId: "code-language" }; }

  _scheduleCompletionUpdate({ immediate = false } = {}) {
    if (this.disabled || this.readonly || this._isComposing) return;
    if (immediate) {
      if (this._completionUpdateFrame) cancelAnimationFrame(this._completionUpdateFrame);
      this._completionUpdateFrame = 0;
      this._maybeUpdateCompletions();
      return;
    }
    if (this._completionUpdateFrame) return;
    this._completionUpdateFrame = requestAnimationFrame(() => {
      this._completionUpdateFrame = 0;
      this._maybeUpdateCompletions();
    });
  }
  _maybeUpdateCompletions() {
    if (this.disabled || this.readonly || this._isComposing) return; const ctx = this._getContext(); if (ctx.selectionStart !== ctx.selectionEnd) { this._closeCompletion(); return; } const providers = [...this._providers.values()].sort((a, b) => b.priority - a.priority); let selectedProvider = null; let selectedMatch = null;
    for (const provider of providers) { try { const match = provider.match(ctx); if (match) { selectedProvider = provider; selectedMatch = match; break; } } catch (error) { this._emitError("completion", error, true, { providerId: provider.id }); } }
    if (!selectedProvider || !selectedMatch) { this._closeCompletion(); return; }
    const requestId = this._completion.requestId + 1; this._completion.requestId = requestId; this._completion.abort?.abort(); const abort = new AbortController(); this._completion.abort = abort;
    try { Promise.resolve(selectedProvider.getItems(selectedMatch, ctx, abort.signal)).then(items => { if (abort.signal.aborted || this._completion.requestId !== requestId) return; const normalized = this._normalizeCompletionItems(items); if (!normalized.length) { this._closeCompletion(); return; } this._openCompletion(selectedProvider.id, selectedMatch, normalized); }).catch(error => { if (!abort.signal.aborted) { this._emitError("completion", error, true, { providerId: selectedProvider.id }); this._closeCompletion(); } }); } catch (error) { this._emitError("completion", error, true, { providerId: selectedProvider.id }); this._closeCompletion(); }
  }
  _normalizeCompletionItems(items) { const seen = new Set(); const out = []; for (const item of items || []) { if (!item?.id || !item?.label) continue; const key = `${item.kind}:${item.id}`; if (seen.has(key)) continue; seen.add(key); out.push(item); } return out; }
  _openCompletion(providerId, match, items) { const was = this._completion.open; this._completion.open = true; this._completion.providerId = providerId; this._completion.match = match; this._completion.items = items; const preferred = clamp(this._completion.activeIndex, 0, items.length - 1); this._completion.activeIndex = this._enabledCompletionIndex(preferred, 1); this._hideSelectionToolbar?.(); this._renderCompletion(); if (!was) this._dispatch("md-completion-open", { providerId, match, items }); }
  _closeCompletion() {
    const wasOpen = this._completion.open;
    const detail = { providerId: this._completion.providerId, match: this._completion.match };
    this._completion.abort?.abort();
    this._completion = { ...this._completion, open: false, providerId: null, match: null, items: [], activeIndex: 0, requestId: this._completion.requestId + 1, abort: null };
    this._renderCompletion();
    if (wasOpen) this._dispatch("md-completion-close", detail);
  }
  _renderCompletion() {
    if (!this._completionPopup) return; const open = this._completion.open && this._completion.items.length > 0; this._completionPopup.hidden = !open; const controller = this._isSourceActive() ? this._sourceTextarea : this._liveEditor; controller?.setAttribute("aria-expanded", open ? "true" : "false");
    if (!open) { this._completionPopup.innerHTML = ""; this._completionPopup.scrollTop = 0; this._sourceTextarea?.removeAttribute("aria-activedescendant"); this._liveEditor?.removeAttribute("aria-activedescendant"); return; }
    const previousScrollTop = this._completionPopup.scrollTop;
    const activeId = this._completion.activeIndex >= 0 ? `${this._ids.completion}-item-${this._completion.activeIndex}` : null;
    if (activeId) controller?.setAttribute("aria-activedescendant", activeId); else controller?.removeAttribute("aria-activedescendant");
    this._completionPopup.innerHTML = this._completion.items.map((item, index) => `<div id="${this._ids.completion}-item-${index}" class="completion-item" part="${index === this._completion.activeIndex ? "completion-item completion-item-active" : "completion-item"}" role="option" aria-selected="${index === this._completion.activeIndex ? "true" : "false"}" aria-disabled="${item.disabled ? "true" : "false"}" data-index="${index}"><div class="completion-label">${escapeHtml(item.label)}</div><div class="completion-detail">${escapeHtml(item.detail || "")}</div>${item.description ? `<div class="completion-description">${escapeHtml(item.description)}</div>` : ""}</div>`).join("");
    this._completionPopup.scrollTop = previousScrollTop;
    this._positionCompletionPopup();
    this._scrollActiveCompletionIntoView();
  }
  _scrollActiveCompletionIntoView() {
    const popup = this._completionPopup;
    const option = popup?.querySelector('[role="option"][aria-selected="true"]');
    if (!popup || !option) return;
    const optionTop = option.offsetTop;
    const optionBottom = optionTop + option.offsetHeight;
    const visibleTop = popup.scrollTop;
    const visibleBottom = visibleTop + popup.clientHeight;
    if (optionTop < visibleTop) popup.scrollTop = optionTop;
    else if (optionBottom > visibleBottom) popup.scrollTop = optionBottom - popup.clientHeight;
  }
  _positionCompletionPopup() {
    const shell = this._shadow.querySelector(".editor-shell"); if (!shell || !this._completionPopup) return; const shellRect = shell.getBoundingClientRect(); let rect = null;
    try { const sel = this._shadow.getSelection?.() || globalThis.getSelection?.(); if (sel?.rangeCount) rect = sel.getRangeAt(0).getBoundingClientRect(); } catch {}
    if (!rect || (!rect.width && !rect.height)) { const target = this._domPositionFromSource(this._selection.start)?.editable || this._sourceTextarea; rect = target?.getBoundingClientRect?.(); }
    const left = clamp((rect?.left ?? shellRect.left) - shellRect.left, 4, Math.max(4, shellRect.width - 260)); const top = clamp((rect?.bottom ?? shellRect.top) - shellRect.top + 6, 4, Math.max(4, shellRect.height - 16));
    this._completionPopup.style.left = `${left}px`; this._completionPopup.style.top = `${top}px`;
  }
  _enabledCompletionIndex(start, direction = 1) {
    const n = this._completion.items.length;
    if (!n) return -1;
    const step = direction < 0 ? -1 : 1;
    for (let distance = 0; distance < n; distance += 1) {
      const index = (start + (distance * step) + n) % n;
      if (!this._completion.items[index]?.disabled) return index;
    }
    return -1;
  }
  _moveCompletion(delta) {
    const n = this._completion.items.length;
    if (!n) return;
    const start = this._completion.activeIndex < 0 ? (delta < 0 ? n - 1 : 0) : this._completion.activeIndex + delta;
    const index = this._enabledCompletionIndex((start + n) % n, delta);
    if (index >= 0) this._setCompletionIndex(index, delta);
  }
  _setCompletionIndex(index, direction = 1) { const n = this._completion.items.length; if (!n) return; this._completion.activeIndex = this._enabledCompletionIndex(clamp(index, 0, n - 1), direction); this._renderCompletion(); }
  _acceptCompletion(source = "action") { if (!this._completion.open || !this._completion.items.length) return fail("not-applicable"); const provider = this._providers.get(this._completion.providerId); const item = this._completion.items[this._completion.activeIndex]; if (!provider || !item || item.disabled) return fail("not-applicable"); const ctx = this._getContext(); let result; try { const currentMatch = provider.match(ctx); if (!currentMatch) { this._closeCompletion(); return fail("not-applicable"); } result = provider.apply(item, currentMatch, ctx); } catch (error) { this._emitError("completion", error, true, { providerId: provider.id }); this._closeCompletion(); return fail("provider-error", String(error?.message || error)); } this._closeCompletion(); if (result?.ok && result.transaction) { const before = this._snapshot(); this._applyTransaction({ ...result.transaction, source: source === "pointer" ? "pointer" : "keyboard", actionId: "completion.accept" }, { source: source === "pointer" ? "pointer" : "keyboard" }); const after = this._snapshot(); this._dispatch("md-completion-accept", { providerId: provider.id, item, before, after }); if (result.announcement) this._announce(result.announcement); return okNoop(result.announcement); } return result || fail("not-applicable"); }

  _updateFormValue() { if (!this._internals) return; this.disabled ? this._internals.setFormValue(null) : this._internals.setFormValue(this._value); }
  _fallbackValidity() { const flags = this._computeValidityFlags(); return { valid: Object.keys(flags).length === 0, valueMissing: Boolean(flags.valueMissing), tooShort: Boolean(flags.tooShort), tooLong: Boolean(flags.tooLong), customError: Boolean(flags.customError) }; }
  _computeValidityFlags() { const flags = {}; const value = this._value; if (this._customValidityMessage) flags.customError = true; if (this.required) { const empty = DEFAULTS.emptyRequiredTrim ? value.trim().length === 0 : value.length === 0; if (empty) flags.valueMissing = true; } const min = parseLengthConstraint(this.getAttribute("minlength")); if (min != null && value.length > 0 && value.length < min) flags.tooShort = true; const max = parseLengthConstraint(this.getAttribute("maxlength")); if (max != null && value.length > max) flags.tooLong = true; return flags; }
  _updateValidity() { if (!this._sourceTextarea) return; const flags = this._computeValidityFlags(); const valid = Object.keys(flags).length === 0; if (valid) this._validationVisible = false; for (const el of [this._sourceTextarea, this._liveEditor]) el?.setAttribute("aria-invalid", valid ? "false" : "true"); this._validation.textContent = ""; this._validationMessage = this._customValidityMessage || ""; const anchor = this.mode === "source" ? this._sourceTextarea : this._liveEditor; const setValidityMessage = valid ? "" : (this._customValidityMessage || " "); this._internals?.setValidity(flags, setValidityMessage, anchor); }
  _emitSelectionChange() { this._dispatch("md-selection-change", { selectionStart: this._selection.start, selectionEnd: this._selection.end, selectionDirection: this._selection.direction || "none" }); }
  _announce(message) { if (!message || !this._status) return; this._status.textContent = ""; requestAnimationFrame(() => { this._status.textContent = message; }); }
  _dispatch(name, detail = {}, options = {}) { const event = new CustomEvent(name, { detail, bubbles: options.bubbles ?? true, composed: options.composed ?? true, cancelable: options.cancelable ?? false }); this.dispatchEvent(event); return event; }
  _emitError(phase, error, recoverable = true, extra = {}) { this._dispatch("md-error", { phase, error, recoverable, ...extra }); }
  formResetCallback() { this.reset(); }
  formDisabledCallback(disabled) {
    const next = Boolean(disabled);
    if (this._formDisabled === next) return;
    this._formDisabled = next;
    if (!this._hasConnected) return;
    if (next) this._closeCompletion();
    this._syncAttributesToControls();
    if (next) this.blur();
    this._renderAll({ restoreSelection: !next, force: true });
    this._updateFormValue();
    this._updateValidity();
  }
  formStateRestoreCallback(state) { if (typeof state === "string") this.value = state; }
}

class MdLiveEditorElement extends WritemarkEditorElement {}

if (globalThis.customElements) {
  if (!customElements.get(TAG_NAME)) customElements.define(TAG_NAME, WritemarkEditorElement);
  if (!customElements.get(LEGACY_TAG_NAME)) customElements.define(LEGACY_TAG_NAME, MdLiveEditorElement);
}

export { WritemarkEditorElement, MdLiveEditorElement, renderMarkdown, renderInlineMarkdown, parseBlocks, parseListItem, parseHeading, parseBlockquote, htmlToMarkdown, tsvToMarkdownTable };
