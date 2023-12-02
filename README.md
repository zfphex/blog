A fast static site generator written in Rust.

### Features

- Simple and lightweight (single file, 500 loc)
- Hot reloading
- Extended Markdown syntax with metadata
- Syntax highlighting
- LaTeX Support
- Template system (can embed anything directly into html)
- 100/100 Lighthouse performance

### Design

```
/src - site generator
/markdown - stores markdown files that will be compiled
/templates - stores the templates to be compiled
/site - stores the compiled pages of the website
/site/assets - fonts, styles, images, etc
/site/img - images.
```

### Metadata

```
title: This is a title 
summary: This is a summary of the post
date: d/m/y

```

### TODO

- [ ] Remove chrono from dependencies.
- [ ] Github pages favicon
- [ ] Compile math instead of rendering at runtime. 
- [ ] Improve performance of syntax highlighting.
- [ ] Table of contents on right-side of post. 
- [ ] Use last modified date instead of blake3 hash (if it's faster).
- [ ] Simplify reference system. I know zola has a system for it.
- [ ] Since ID's are used a lot, they should be prefixed with something. When creating citations and in post links like [](#blog). There might be some overlap. So change, #post to #zx-post or something. Maybe id's can be localized or something?
- [ ] Multiline summaries in post metadata.
- [ ] Match syntax highlighting theme with site theme.
- [ ] Delete compiled posts that aren't in the markdown list anymore.
- [ ] Markdown files that reference images are not copied to `/site/img/`
- [ ] Improve margins, especially around footnotes.
- [ ] Add a nav bar or improve page navigation with better back button.
- [ ] `html` files that are in `site/` but note `markdown/` will not be removed automatically.
- [ ] The first h1 in a markdown file should probably be the heading instead of using metadata.
- [ ] Just use the creation date instead of user defining it. We might not even need metadata in the markdown files.
- [ ] Swap to MDC once it's stable.
- [ ] Statically generate LaTex with MDC. 
- [ ] Diagram/Graph support. Could use https://github.com/plotters-rs/plotters
- [ ] Syntax highting with diffing
      https://www.11ty.dev/docs/plugins/syntaxhighlight/

   I really like the green `+` and red `-` that Eleventy does.

   ```js
   +function myFunction() {
      // …
   -  return true;
   }
   ```