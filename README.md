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

Updated template:

```rust
#[derive(Template)]
struct Post {
    path: PathBuf,
    title: String,
    user: String,
    words: usize,
    date: SystemTime,
    last_edited: u64,
    read_duration: Duration,
}

html!("templates/posts.html", posts: Vec<Post>);
//^ Something like this?

```

```html
<header>
    <div class="header-bar"></div>
    <div ><a href="index.html">zX3no</a></div>
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
        {% for post in posts %}
        <a class="test" href="{{ post.name }}">
            <h2>
                <span>{{ post.title }}</span>
            </h2>
            <div class="sub-heading">
                <p>
                    <i class="fa fa-user"></i>
                    {{ post.user }}
                    <i class="fa fa-calendar"></i>
                    {{ post.date }}
                </p>
                <p>
                    <i class="fa fa-clock-o"></i>
                    {{ post.duration }}
                    <i class="fa fa-pencil"></i>
                    {{ post.words }}
                </p>
            </div>
            <p>
                {{ post.overview }}
            </p>
        </a>
        {% endfor %}
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

