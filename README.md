## Blog

```
/src - the compiler
/markdown - stores markdown files that will be compiled
/templates - stores the templates to be compiled
/site - stores the compiled pages of the website
/site/assets - fonts, styles, images, etc.
/site/img - images.
```

- [x] Better logging
- [x] Allow empty metadata
- [x] Sort posts by date.
- [x] Date posted metadata.
- [x] Summary text is too small and doesn't stand out.
- [x] Deleted files are left in memory.
- [x] Hyperlink colors are weird.
- [x] Re-work the color scheme.
- [x] Add time to logging.
- [x] HTML, CSS & JS minification.
- [x] Syntax highlighting https://github.com/trishume/syntect
- [x] ~~Blurry transform scaling.~~
- [x] Add a github logo in the bottom left or right. Mabye a 'Made by Bay ❤' or something.
- [x] Cleanup margins, there are different ones for paragraphs, tables, code, heading etc..
- [x] Write a package command that combines all files for shipping. Will need to fix paths too!
- [x] LaTex support.
- [x] Mobile support.
- [x] `<sup>` should be styled a different color.
- [x] Inline code is broken I think? \`inline code\`
- [x] Cleanup iframe styling.
- [x] Add `target="_blank"` to all `hrefs`.
- [x] Back button. Currently there is no way to get to the home screen from a post.
- [ ] Compile math instead of rendering at runtime. (HARD, LOW PRIORITY)
- [ ] Table of contents on right-side of post. (HARD, LOW PRIORITY)
- [ ] Simplify reference system. I know zola has a system for it. (HARD, MEDIUM PRIORITY)
- [ ] Diagrams.
- [ ] Since ID's are used a lot, they should be prefixed with something. When creating citations and in post links like [](#blog). There might be some overlap. So change, #post to #zx-post or something. Maybe id's can be localized or something?
- [ ] Multiline summaries in post metadata.
- [ ] Fix syntax highlighting theme.
- [ ] Delete compiled posts that aren't in the markdown list anymore.
- [ ] All the margins are wrong especially around the footnotes.
- [ ] CLS on KaTex
- [ ] Reference section needs indentation on the content see (i. Sample rate, ii. Sampels).
- [ ] Pagination (or should there be infinite scrolling)
- [ ] Improve performance of syntax highlighting
- [ ] Add a nav bar or improve page navigation with better back button.
- [ ] Minify `style.css`

h1 = 24 pixels

h2 = 22 pixels

h3 = 20 pixels

h4 = 18 pixels

h5 = 16 pixels

h6 = 16 pixels

https://stackoverflow.com/questions/55696808/why-do-h5-and-h6-have-smaller-font-sizes-than-p-in-most-user-agent-default

Nice syntax highting with diffing
https://www.11ty.dev/docs/plugins/syntaxhighlight/

Maybe I don't need diagram support I can just render images and add them.

https://www.mermaid.live

https://github.com/mermaid-js/mermaid

https://mermaid.js.org/intro/n00b-gettingStarted.html#_3-calling-the-javascript-api