use anyhow::Result;
use clap::Parser;
use regex::Regex;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use url::Url;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    permalink: Option<String>,
    #[clap(short, long, action, help = "Format code snippet with Markdown")]
    markdown: bool,
}

fn print_matches(document: &Html, selector: &Selector) {
    for element in document.select(selector) {
        let text = element.text().fold(String::new(), |a, b| a + b);
        // we trim end because else lines that look like "\n" cause problems
        // and on other lines it's always safe to do so
        println!("{}", text.trim_end());
    }
}

fn parse_file_path(url: &Url) -> String {
    let regex = Regex::new(r".*/blob/[a-z0-9]+/(.*$)").unwrap();
    regex.captures(url.path()).unwrap()[1].to_string()
}

fn parse_github_url_line_numbers(fragment: &str) -> Result<Vec<u32>> {
    let mut line_numbers = fragment
        .split('-')
        .map(|str| str.replace('L', ""))
        .map(|num| {
            let num = str::parse::<u32>(&num)?;
            Ok(num)
        })
        .collect::<Result<Vec<u32>>>()?;
    line_numbers.sort_unstable();
    Ok(line_numbers)
}

fn print_markdown_header(url: &Url, line_numbers: Option<&Vec<u32>>) {
    match line_numbers {
        Some(numbers) => {
            println!(
                "[{}:L{}-L{}]({})",
                parse_file_path(url),
                numbers[0],
                numbers.get(1).unwrap_or(&numbers[0]),
                url
            )
        }
        None => {
            println!("[{}]({})", parse_file_path(url), url)
        }
    };
    println!("```")
}

fn print_markdown_footer() {
    println!("```");
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
                let line_numbers = parse_github_url_line_numbers(line_numbers)?;

                if cli.markdown {
                    print_markdown_header(&url, Some(&line_numbers));
                }

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
                // anyhow that I don't want to get to the bottom of right now (so that's why
                // I'm not using `?`)
                let selector = Selector::parse(".blob-code-inner").unwrap();

                if cli.markdown {
                    print_markdown_header(&url, None)
                }

                print_matches(&document, &selector);
            }
        }

        if cli.markdown {
            print_markdown_footer();
        }
    }

    Ok(())
}
