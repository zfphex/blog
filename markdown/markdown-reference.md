~~~
title: Markdown Reference
date: 10/01/2023 +0930
~~~


Common text

_Emphasized text_

~~Strikethrough text~~

__Strong text__

___Strong emphasized text___

# H1

## H2

### H3

#### H4

##### H5

##### H6

| Heading       | Item   |
|---------------|--------|
| Sub-heading 1 | Item 1 |
| Sub-heading 2 | Item 2 |
| Sub-heading 3 | Item 3 |
| Sub-heading 4 | Item 4 |

Left aligned Header | Right aligned Header | Center aligned Header
| :--- | ---: | :---:
Content Cell  | Content Cell | Content Cell
Content Cell  | Content Cell | Content Cell

- Item 1
- Item 2
- Item 3

* Bullet list
    * Nested bullet
        * Sub-nested bullet etc
* Bullet list item 2

1. A numbered list
    1. A nested numbered list
    2. Which is numbered
2. Which is numbered


> Blockquote
>> Nested blockquote

---

$x + y{a \over b} \times 300$

$x={-b \pm \sqrt {b^2 - 4ac}\over2a}$

`inline codeblock`

```rs
pub fn highlight_line(code: &str) -> String {
    use syntect::{
        easy::HighlightLines,
        highlighting::ThemeSet,
        html::{
            append_highlighted_html_for_styled_line, start_highlighted_html_snippet,
            IncludeBackground,
        },
        parsing::SyntaxSet,
        util::LinesWithEndings,
    };

    let ss = SyntaxSet::load_defaults_newlines();
    let syntax = ss
        .find_syntax_by_token("rs")
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let mut highlighter = HighlightLines::new(syntax, theme);
    let (mut html, bg) = start_highlighted_html_snippet(theme);

    for line in LinesWithEndings::from(code) {
        let regions = highlighter.highlight_line(line, &ss).unwrap();
        append_highlighted_html_for_styled_line(
            &regions[..],
            IncludeBackground::IfDifferent(bg),
            &mut html,
        )
        .unwrap();
    }

    html.push_str("</pre>\n");

    html
}
```

```html
<a id="post" href="~link~">
    <div id="title">
        <span id="hash">#</span>
        <span id="text"><!-- title --></span>
    </div>
    <div id="metadata">
    </div>
    <summary>
        <!-- summary -->
    </summary>
</a>
```
