```
/markdown - stores markdown files that will be compiled
/build - stores the compiled parts of the website
/templates - stores the templates to be compiled
/assets - fonts, styles, images, etc.
/src - the compiler
```

TODO:
Maybe markdown files need a heading for the page and a title for the post list. IDK.
Path's should all be set relative to the project, it's kind of all over the place right now. Just look at build/post_list.html

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
- [ ] Write a package command that combines all files for shipping. Will need to fix paths too!
- [ ] Table of contents on right-side of post.
- [ ] LaTex support.
- [ ] Diagrams.
- [ ] sup should be styled a different color.
- [ ] Since ID's are used a lot, they should be prefixed with something. When creating citations and in post links like [](#blog). There might be some overlap. So change, #post to #zx-post or something. Maybe id's can be localized or something?
- [ ] Multiline summaries in post metadata.
- [ ] Cleanup iframe styling.
- [ ] Fix syntax highlighting theme.
- [ ] Add `target="_blank"` to all `hrefs`.
- [ ] Inline code is broken I think? \`inline code\`

h1 = 24 pixels
h2 = 22 pixels
h3 = 20 pixels
h4 = 18 pixels
h5 = 16 pixels
h6 = 16 pixels

https://stackoverflow.com/questions/55696808/why-do-h5-and-h6-have-smaller-font-sizes-than-p-in-most-user-agent-default


Nice syntax highting with diffing
https://www.11ty.dev/docs/plugins/syntaxhighlight/