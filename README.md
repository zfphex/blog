#### Tasks: 

- [ ] Create basic template for website
- [ ] Move all css into single file. The styling should be consistant across the website. Use h1/h2 over changing the font size.
- [ ] Markdown to html compiler
- [ ] Compiler to replace tags with html (#tag)

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

This could be complied into:
```html
<header>
        <div class="header-bar"></div>
        <div class="main"><a href="https://not-matthias.github.io">not-matthias</a></div>
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
                    <span>Kernel Driver with Rust in 2022</span>
                </h2>

                <div class="sub-heading">
                    <p>
                        <i class="fa fa-user"></i>
                        DeepThought
                        <i class="fa fa-calendar"></i>
                        September 16, 2022
                    </p>

                    <p>
                        <i class="fa fa-clock-o"></i>
                        19 min
                        <i class="fa fa-pencil"></i>
                        3697 words
                    </p>
                </div>

                <p>
                    A lot has changed since I wrote my first blog post on how to write kernel drivers with Rust. I
                    learned more about the
                    language and worked on more projects. The goal of this blog post is to keep you updated on the
                    changes from the last 2
                    years.
                </p>
            </a>
        </ul>
    </main>

```

### Markdown 

```
+++
title = "templates/title.html"
user = "Bay"
date = "25/09/22"
duration = "10 minutes"
words = "1500"
+++

# h1 Heading 8-)
## h2 Heading
### h3 Heading
#### h4 Heading
##### h5 Heading
###### h6 Heading

## Horizontal Rules

---

## Emphasis

**This is bold text**

```