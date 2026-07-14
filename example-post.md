July 10, 2026 | formatting showcase

# The Complete Nonograph Formatting Showcase

This post exercises **every** formatting option supported by Nonograph markup, serving as a reference for the experimental JSON parser.

## Text Formatting

Here is **bold text**, *italic text*, _underlined text_, ~strikethrough text~, ^superscript text^, ==highlighted text==, and `inline code`. All in one paragraph.

You can combine them across sentences. **This sentence is bold.** *This one is italic.* _This one is underlined._ And this one has a #hidden spoiler message# in it.

## Headings

# Heading Level 1

## Heading Level 2

### Heading Level 3

#### Heading Level 4

## Blockquotes

> This is a simple blockquote. It creates an indented quote block.

> A second blockquote to test multiple in sequence.

## Footnotes

This paragraph has a reference footnote[^1] and another one[^2].

This paragraph has an inline footnote^[This is defined right here in the text].

[^1]: First footnote — defined at the bottom.
[^2]: Second footnote — also defined at the bottom.

## Links

Here is a [labeled link](https://example.com) in a sentence.

Here is a bare link: [https://nonograph.net]

## Images

![A photo with a caption](https://example.com/photo.jpg)

![](https://example.com/no-caption.png)

## Videos

![Video with caption](https://example.com/clip.mp4)

![](https://example.com/silent.webm)

## Bulleted Lists

- First bullet item
- Second bullet item with **bold**
- Third bullet item with *italic*
- Fourth item with `code` and [a link](https://example.com)

Using alternate markers:

* Asterisk bullet
+ Plus bullet
- Dash bullet

## Numbered Lists

1. First step
2. Second step with _underline_
3. Third step with ==highlighting==

## Tables

| Language | Year | Paradigm |
|:---------|:----:|----------:|
| Rust     | 2015 | Systems  |
| Python   | 1991 | Scripting|
| Haskell  | 1990 | Functional|

## Code Blocks

```rust
fn main() {
    println!("Hello from a Rust code block!");
}
```

```json
{
  "name": "nonograph",
  "version": "1.0.0"
}
```

```py
def greet(name: str) -> str:
    return f"Hello, {name}!"
```

## Comments

// This is a comment — visible only in the .md source.

This paragraph is visible, but the line above is not rendered in HTML.

## Secret Text

Click to reveal: #this is a spoiler# — was it worth it?

#An entire line can be a spoiler too.#

## Dividers

Three-star divider:

***

Single-asterisk divider:

-*-

Thin horizontal line:

---

Double-line divider:

===

## Line Breaks and Paragraphs

This is line one.
This is line two (single line break).

This is a new paragraph (double line break).

## Putting It All Together

> **Note:** This final section mixes features. Here is a [link](https://example.com), some `code`, and a footnote[^1].

1. Read the **markup guide**
2. Write your *post*
3. Preview with `nonograph`

| Feature       | Supported |
|---------------|-----------|
| Bold          | **yes**   |
| Italic        | *yes*     |
| Tables        | yes       |

```bash
echo "That's all, folks!"
```

***
