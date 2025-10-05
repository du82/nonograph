September 23, 2025

# Markup Help
Telegraph supports a limited set of markdown formatting to keep things simple and clean.

**Text Formatting**

`*bold*` - *Wrap text with single asterisks*
`**italic**` - **Wrap text with double asterisks**
`_underline_` - _Wrap text with underscores_
`~strikethrough~` - ~Wrap text with tildes~
`^superscript^` - ^Wrap text with carets^
`inline code` - `Wrap text with backticks`

Like the markup you see in someone elses page? Append ".md" to the end of their URL to get the raw markdown!

**Links**

`[link text](https://example.com)` - Link with custom text
`[https://example.com]` - Link showing the URL

**Media**

Paste image or video URLs directly and they'll be embedded automatically:
- Images: .jpg, .jpeg, .png, .gif, .webp
- Videos: .mp4, .webm, .ogg

**Tables**

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Data A   | Data B   | Data C   |

Align columns with colons in separator row:
- `:---` Left align
- `:---:` Center align
- `---:` Right align

**Code Blocks**

Use triple backticks for code blocks:

```language
your code here
```

You can append a filetype immediately after the backticks (on the same line, no space) to tell the site what language you're using. Hover over the copy button to reveal it.

```json
{
  "name": "telegraph-rs",
  "version": "1.0.0",
  "settings": {
    "debug": true,
    "port": 8080
  }
}
```

**Supported Languages**:

ada, apache, apex, arduino/ino, asm/assembly, asciidoc/adoc, awk, bash/sh/shell, bibtex/bib, c, clj/clojure, cmake, cobol/cob, coffeescript/coffee, cpp/c++, cr/crystal, cs/csharp, css, d, dart, diff, dockerfile/docker, ejs, elisp/emacs-lisp, elm, erl/erlang, ex/elixir, fsharp/fs, f90/f95/fortran, go/golang, gql/graphql, groovy, handlebars/hbs, hs/haskell, html, ini, jade/pug, java, javascript/js, jl/julia, json, jsx, kt/kotlin, latex/tex, less, lisp, lua, make/makefile, markdown/md, matlab, ml/ocaml, nginx, nim, njk/nunjucks, objc/objective-c, org, pas/pascal, patch, pde/processing, perl, php, pl/prolog, plaintext/text/txt, powershell/ps1, properties, purs/purescript, py/python, r, racket/rkt, re/reason/reasonml, rmarkdown/rmd, rs/rust, rst/restructuredtext, ruby/rb, sass, scad/openscad, scala, scm/scheme, scss, sed, smalltalk/st, sol/solidity, sql, st/smalltalk, svelte, swift, tcl, textile, toml, tsx, typescript/ts, v/vlang, verilog, vhdl, vim/vimscript, vue, xml, yaml/yml, zig

**Secret Text**

#hidden message# - Click to reveal hidden text

**Line Breaks**

Press Enter once for a line break
Press Enter twice for a paragraph break

**Limits**

- Posts: 32,000 characters maximum
- Links: 4,096 characters maximum
- Titles: 128 characters maximum

**What's NOT Supported**

No headings, lists, blockquotes. Keep it simple and focused.
