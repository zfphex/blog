## A fast static site generator written in Rust.

### Features

- Simple and lightweight (single file, 500 LOC)
- Hot reloading
- Extended Markdown syntax with metadata
- Syntax highlighting
- LaTeX Support
- Template system (can embed anything directly into html)
- 100/100 Lighthouse performance

### Design

> Currently only Windows is supported.

```
/src - site generator
/markdown - stores markdown files that will be compiled
/templates - stores the templates to be compiled
/themes - syntax highlighting themes
/site - stores the compiled pages of the website
/site/assets - fonts, styles, images, etc
/site/img - images
```

### Metadata

```
<!--
title: This is a title
summary: This is a summary of the post
date: dd/mm/yy
-->
```

### Templates

HTML comments can be replaced with anything.
This may seem slow, but it's not; at least compared to syntax highlighting.

```
replace (42 runs) src\main.rs:370
  - total: 369.1µs
  - mean:  8.788µs
  - min:   6.2µs
  - max:   32.7µs
```

```rs
for file in &files {
    let item = item
        .replace("<!-- title -->", &file.title)
        .replace("<!-- summary -->", &file.summary)
        .replace("<!-- date -->", &file.index_date)
        .replace("<!-- read_time -->", &file.read_time())
        .replace("<!-- word_count -->", &file.word_count())
        .replace(
            "<!-- link -->",
            file.build_path.file_name().unwrap().to_str().unwrap(),
        );
}
```

```html
<a class="post" href="<!-- link -->">
  <div id="title">
    <span id="hash">#</span>
    <span id="text"><!-- title --></span>
  </div>
  <div id="metadata">
    <div id="metadata-left">
      <svg>
        <use xlink:href="#user" />
      </svg>
      <span style="padding-right: 4px">Bay</span>
      <svg>
        <use xlink:href="#calender" />
      </svg>
      <!-- date -->
    </div>
    <div id="metadata-right">
      <svg>
        <use xlink:href="#clock" />
      </svg>
      <!-- read_time -->
    </div>
  </div>
  <summary>
    <!-- summary -->
  </summary>
</a>
```

### TODO

- [ ] Strip tailwind colors in build css.
- [ ] Compile math to MathML instead of rendering with javascript.
- [ ] Improve performance of syntax highlighting.
- [ ] Table of contents on right-side of post.
- [ ] Simplify reference system. I know zola has a system for it.
- [ ] Since ID's are used a lot, they should be prefixed with something. When creating citations and in post links like [](#blog). There might be some overlap. So change, #post to #zx-post or something. Maybe id's can be localized or something?
- [ ] Multiline summaries in post metadata.
- [ ] Match syntax highlighting theme with site theme.
- [ ] Delete compiled posts that aren't in the markdown list anymore.
- [ ] Markdown files that reference images are not copied to `/site/img/`
- [ ] Improve margins, especially around footnotes.
- [ ] `html` files that are in `site/` but note `markdown/` will not be removed automatically.
- [ ] The first h1 in a markdown file should probably be the heading instead of using metadata.
- [ ] Just use the creation date instead of user defining it. We might not even need metadata in the markdown files.
- [ ] Diagram/Graph support. Could use https://github.com/plotters-rs/plotters
- [ ] Move github icon and back button to center of screen when page is too small.
- [ ] Many of the example on katex do not work, probably broken in the markdown to html conversion step
