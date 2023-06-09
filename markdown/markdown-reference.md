~~~
title: Markdown Reference
date: 10/01/2023 +0930
~~~

# H1

## H2

### H3

#### H4

##### H5

##### H6

---

| Heading       | Item   |
|---------------|--------|
| Sub-heading 1 | Item 1 |
| Sub-heading 2 | Item 2 |
| Sub-heading 3 | Item 3 |
| Sub-heading 4 | Item 4 |

List
- Item 1
- Item 2
- Item 3

> Quote 1
>
> Quote 2

> Quote 3

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
