use anyhow::Result;
use clap::Parser;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use url::Url;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    permalink: Option<String>,
}

fn print_matches(document: &Html, selector: &Selector) {
    for element in document.select(selector) {
        let text = element.text().fold(String::new(), |a, b| a + b);
        // we trim end because else lines that look like "\n" cause problems
        // and on other lines it's always safe to do so
        println!("{}", text.trim_end());
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(permalink) = cli.permalink.as_deref() {
        let url = Url::parse(permalink)?;

        let response = get(permalink)?;
        let raw_html = response.text()?;

        let document = Html::parse_document(&raw_html);

        match url.fragment() {
            Some(line_numbers) => {
                // the line numbers look like "L24" or "L24-L34"
                let mut line_numbers: Vec<u32> = line_numbers
                    .split('-')
                    .map(|str| str.replace('L', ""))
                    .map(|num| {
                        let num = str::parse::<u32>(&num)?;
                        Ok(num)
                    })
                    .collect::<Result<Vec<u32>>>()?;
                line_numbers.sort_unstable();

                // this is ugly but don't worry about it
                // we make a range between the first and last line number, and then go through
                // constructing selectors to grab that line out of the HTML
                for line_number in
                    *line_numbers.first().unwrap()..(*line_numbers.last().unwrap() + 1)
                {
                    // ids look like "LC14", "LC234", etc
                    let selector = Selector::parse(&format!("#LC{}", line_number)).unwrap();
                    print_matches(&document, &selector);
                }
            }
            None => {
                // this shouldn't ever fail and there seems to be some incompatibility with
                // anyhow that I don't want to get to the bottom of right now
                let selector = Selector::parse(".blob-code-inner").unwrap();
                print_matches(&document, &selector);
            }
        }
    }

    Ok(())
}
