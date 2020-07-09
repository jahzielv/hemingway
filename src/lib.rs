use ansi_term::Style;
use itertools::Itertools;
#[derive(Debug)]
pub struct ProcessedFeed {
    pub title: String,
    pub items: Vec<String>,
}

impl std::fmt::Display for ProcessedFeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "📖 {}\n\t{}",
            Style::new().bold().paint(&self.title),
            format!("{}", self.items.iter().format("\n\t"))
        )
    }
}
