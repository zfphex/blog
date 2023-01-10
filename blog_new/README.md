```
/markdown - stores markdown files that will be compiled
/build - stores the compiled parts of the website
/site - the website which will link to files in /build
/templates - stores the templates to be compiled
/assets - fonts, styles, images, etc.
/src - the compiler
```

TODO:
Maybe markdown files need a heading for the page and a title for the post list. IDK.
Path's should all be set relative to the project, it's kind of all over the place right now. Just look at build/post_list.html

- [x] Better logging
- [x] Allow empty metadata
- [ ] Sort posts by date.
- [ ] Date posted metadata.
- [ ] Summary text is too small.
- [ ] Syntax highlighting https://github.com/trishume/syntect


Nice syntax highting with diffing

https://www.11ty.dev/docs/plugins/syntaxhighlight/