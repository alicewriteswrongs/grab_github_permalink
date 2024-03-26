use anyhow::Result;
use clap::Parser;
use regex::Regex;
use reqwest::blocking::get;
use std::path::Path;
use url::Url;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    permalink: Option<String>,
    #[clap(short, long, action, help = "Format code snippet with Markdown")]
    markdown: bool,
}

/// Data parsed from a permalink URL which can be used to construct a 'raw' github URL
struct ParsedUrl {
    owner: String,
    repo: String,
    sha: String,
    file_path: String,
    line_numbers: Option<Vec<usize>>,
}

impl ParsedUrl {
    /// Parse a url like
    /// https://github.com/ionic-team/stencil/blob/d5011d0ca5988ca862263fb990f6eb1a34b9c4d3/src/declarations/stencil-public-runtime.ts#L572-L578
    fn new(url: &Url) -> ParsedUrl {
        let regex = Regex::new(r"/([a-z0-9-]+)/([a-z0-9-]+)/blob/([a-z0-9]+)/(.*$)")
            .expect("should be able to compile this regex no problem");

        let captures = regex.captures(url.path()).unwrap();
        let owner = captures[1].to_string();
        let repo = captures[2].to_string();
        let sha = captures[3].to_string();
        let filepath = captures[4].to_string();

        let line_numbers = url
            .fragment()
            .and_then(|fragment| parse_github_url_line_numbers(fragment).ok());

        ParsedUrl {
            owner,
            repo,
            sha,
            file_path: filepath,
            line_numbers,
        }
    }
}

/// Parse line numbers from a URL fragment (i.e. the bit of a URL after `#`)
fn parse_github_url_line_numbers(fragment: &str) -> Result<Vec<usize>> {
    let mut line_numbers = fragment
        .split('-')
        .map(|str| str.replace('L', ""))
        .map(|num| {
            let num = str::parse::<usize>(&num)?;
            Ok(num)
        })
        .collect::<Result<Vec<usize>>>()?;
    line_numbers.sort_unstable();
    Ok(line_numbers)
}

fn print_markdown_header(url: &Url, parsed_url: &ParsedUrl) {
    let extension = Path::new(&parsed_url.file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap();

    match &parsed_url.line_numbers {
        Some(numbers) => {
            println!(
                "[{}:L{}-L{}]({})",
                parsed_url.file_path,
                numbers[0],
                numbers.get(1).unwrap_or(&numbers[0]),
                url
            )
        }
        None => {
            println!("[{}]({})", parsed_url.file_path, url)
        }
    };
    println!("```{}", extension)
}

fn print_markdown_footer() {
    println!("```");
}

fn construct_raw_github_link(parsed_url: &ParsedUrl) -> Url {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        parsed_url.owner, parsed_url.repo, parsed_url.sha, parsed_url.file_path
    );
    Url::parse(&url).expect("should be able to parse this constructed URL string")
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(permalink) = cli.permalink.as_deref() {
        let url = Url::parse(permalink)?;
        let parsed_url = ParsedUrl::new(&url);
        let raw_url = construct_raw_github_link(&parsed_url);

        let response = get(raw_url)?;
        let raw_file_contents = response.text()?;

        if cli.markdown {
            print_markdown_header(&url, &parsed_url);
        }

        match parsed_url.line_numbers {
            Some(line_numbers) => {
                // this is ugly but don't worry about it
                // we make a range between the first and last line number, and then go through
                let first = line_numbers.first().unwrap();
                let last = line_numbers.last().unwrap();

                let lines = raw_file_contents
                    .lines()
                    .skip(first - 1)
                    .take(last - first + 1);

                // constructing selectors to grab that line out of the HTML
                for line in lines {
                    println!("{}", line);
                }
            }
            None => {
                // print the whole file
                println!("{}", raw_file_contents);
            }
        }

        if cli.markdown {
            print_markdown_footer();
        }
    }

    Ok(())
}
