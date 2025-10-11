October 5, 2025

# Markup Help

Nonograph supports a limited set of markdown formatting to keep things simple and clean.

## Text Formatting

```md
**bold**
```
**Wrap text with single asterisks**

```md
*italic*
```
*Wrap text with double asterisks*

```md
_underline_
```
_Wrap text with underscores_

```md
~strikethrough~
```
~Wrap text with tildes~

```md
^superscript^
```
You can^Wrap text with carets^ to make it ^superscript!^

```md
==highlight==
```
==Wrap text with double equals==

```md
`inline code`
```
`Wrap text with backticks`

## Headings

```md
# Heading 1
```

# Large heading

```md
## Heading 2
```

## Medium heading

```md
### Heading 3
```

### Small heading

```md
#### Heading 4
```

#### Smaller heading

## Blockquotes

```md
> This is a quoted text
```

> Creates an indented quote block

## Footnotes

**Reference footnotes** - mark position with `[^1]` and define at bottom:

```md
This has a footnote[^1] and another[^2].

[^1]: First footnote definition.
[^2]: Second footnote definition.
```

**Inline footnotes** - define directly in text:

```md
This has an inline footnote^[The footnote text goes here].
```

Like the markup you see in someone else's page? Append ".md" to the end of their URL to get the raw markdown!

## Links

```md
[link text](https://example.com)
```
Link with custom text

```md
[https://example.com]
```
Link showing the URL

## Media

### Images

Paste image or video URLs directly and they'll be embedded automatically:
- Images: .jpg, .jpeg, .png, .gif, .webp

```md
![Alt text](https://example.com/image.jpg)
```
Image with alt text

```md
![](https://example.com/image.png)
```
Image without alt text

### Videos

Paste video URLs directly and they'll be embedded automatically:
- Videos: .mp4, .webm, .ogg

### Legacy Media Embedding

You can still paste image or video URLs directly and they'll be embedded automatically:
- Images: .jpg, .jpeg, .png, .gif, .webp

## Tables

```md
| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Data A   | Data B   | Data C   |
```

Align columns with colons in separator row:
- `:---` Left align
- `:---:` Center align
- `---:` Right align

## Code Blocks

Use triple backticks for code blocks:

````md
```language
your code here
```
````

You can append a filetype immediately after the backticks (on the same line, no space) to tell the site what language you're using. Hover over the copy button to reveal it.

````md
```json
{
  "name": "nonograph",
  "version": "1.0.0",
  "settings": {
    "debug": true,
    "port": 8080
  }
}
```
````

**Supported Languages**:

ada, apache, apex, arduino/ino, asm/assembly, asciidoc/adoc, awk, bash/sh/shell, bibtex/bib, c, clj/clojure, cmake, cobol/cob, coffeescript/coffee, cpp/c++, cr/crystal, cs/csharp, css, d, dart, diff, dockerfile/docker, ejs, elisp/emacs-lisp, elm, erl/erlang, ex/elixir, fsharp/fs, f90/f95/fortran, go/golang, gql/graphql, groovy, handlebars/hbs, hs/haskell, html, ini, jade/pug, java, javascript/js, jl/julia, json, jsx, kt/kotlin, latex/tex, less, lisp, lua, make/makefile, markdown/md, matlab, ml/ocaml, nginx, nim, njk/nunjucks, objc/objective-c, org, pas/pascal, patch, pde/processing, perl, php, pl/prolog, plaintext/text/txt, powershell/ps1, properties, purs/purescript, py/python, r, racket/rkt, re/reason/reasonml, rmarkdown/rmd, rs/rust, rst/restructuredtext, ruby/rb, sass, scad/openscad, scala, scm/scheme, scss, sed, smalltalk/st, sol/solidity, sql, st/smalltalk, svelte, swift, tcl, textile, toml, tsx, typescript/ts, v/vlang, verilog, vhdl, vim/vimscript, vue, xml, yaml/yml, zig

## Comments

```md
// This is a comment
```
Comments start with `// ` at the beginning of a line. They appear in the .md version but are hidden from the HTML output. Anyone can append `.md` to any URL and see the comments, so don't leave sensitive things there!
// You can see this only from the .md version!

## Secret Text

```md
#hidden message#
```
Click to reveal #hidden text# which can be in-line or on its own line. These are also commonly known as #spoilers#.

## Line Breaks

Press Enter once for a line break

Press Enter twice for a paragraph break

## Limits

- Posts: 128,000 characters maximum
- Links: 4,096 characters maximum
- Titles: 128 characters maximum
- Alias: 32 characters maximum

## What's NOT Supported

No nested formatting. Keep it simple and focused.
