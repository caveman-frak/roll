use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    content: Option<String>,
}

impl Args {
    pub fn content(&self) -> Option<&str> {
        self.content.as_ref().map(|s| &s[..])
    }
}
