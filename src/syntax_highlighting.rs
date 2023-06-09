use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    html::{append_highlighted_html_for_styled_line, IncludeBackground},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

pub struct Highlighter {
    pub ss: SyntaxSet,
    pub buffer: String,
    pub theme: Theme,
    pub language: String,
}

impl Highlighter {
    //TODO: Create custom theme.
    pub(crate) fn new() -> Self {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let theme = ts.themes["base16-ocean.dark"].clone();

        Self {
            ss,
            theme,
            buffer: String::new(),
            language: String::new(),
        }
    }
    pub fn push_str(&mut self, str: &str) {
        self.buffer.push_str(str);
    }
    pub fn highlight(&mut self) -> String {
        let syntax = self
            .ss
            .find_syntax_by_token(&self.language)
            .unwrap_or_else(|| self.ss.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut html = String::new();

        let lines = std::mem::take(&mut self.buffer);
        for line in LinesWithEndings::from(&lines) {
            let regions = highlighter.highlight_line(line, &self.ss).unwrap();
            append_highlighted_html_for_styled_line(&regions[..], IncludeBackground::No, &mut html)
                .unwrap();
        }

        html
    }
}
