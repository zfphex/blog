#### Tasks: 

- [ ] Create basic template for website
- [ ] Move all css into single file. The styling should be consistant across the website. Use h1/h2 over changing the font size.
- [ ] Markdown to html compiler
- [ ] Custom HTML to html compiler

#### Compiler

Imagine the following:

```html 
 #header

 <h1 style="text-align: center;">Posts</h1>

 <main class="list">
        <ul>
            {{ posts }}
        </ul>
 </main>
```

Posts would be defined as some schema:
```html
 <a class="test" href="#link">
 <h2>
     <span>{{ title }}</span>
 </h2>
 <div class="sub-heading">
    <p>
        <i class="fa fa-user"></i>
        {{ user }}
        <i class="fa fa-calendar"></i>
        {{ date }}
    </p>

    <p>
        <i class="fa fa-clock-o"></i>
        {{ duration }}
        <i class="fa fa-pencil"></i>
        {{ words }}
    </p>
</div>

 <p>
    {{ overview }} 
 </p>

</a>
```

Post in markdown:

```
+++
title = Example Post
user = Bay
+++

# Heading 8-)

> Use math as the language to enable KaTeX parsing.
```math
  c = \pm\sqrt{a^2 + b^2}
.```

```

Alternative syntax:
```
---
title: Example Post
user: Bay
---
```

This will be complied into a list of posts:
```html
  <header>
        <div class="header-bar"></div>
        <div class="main"><a href="index.html">zX3no</a></div>
        <nav>
            <a href="index.html">/home</a>
            <a href="posts.html">/posts</a>
            <a href="projects.html">/projects</a>
            <a href="about.html">/about</a>
        </nav>
    </header>

    <h1 style="text-align: center;">Posts</h1>

    <main class="list">
        <ul>
            <a class="test" href="https://not-matthias.github.io/posts/first-year-of-uni/">
                <h2>
                    <span>Example Post</span>
                </h2>

                <div class="sub-heading">
                    <p>
                        <i class="fa fa-user"></i>
                        Bay
                        <i class="fa fa-calendar"></i>
                        September 19, 2022
                    </p>

                    <p>
                        <i class="fa fa-clock-o"></i>
                        < 1 minute read 
                        <i class="fa fa-pencil"></i>
                        30 words
                    </p>
                </div>

                <summary>
                </summary>

            </a>
        </ul>
    </main>

```


```
|
|- src
|   |- index.html
|   |- about.html
|   |- posts.html
|   |- projects.html
|- build
|   |- posts.html //posts are compiled into a list with word count, date modified etc.
|- templates
|   |- 
|- posts
     |- post processing.md
```