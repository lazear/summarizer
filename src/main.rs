use std::io::Read;
use std::fs::File;
use std::collections::HashMap;

/// Results from summary analysis
pub struct Digest<'a> {
    pub text: &'a str,
    pub score: usize,
    pub index: usize,
}

/// Type of paragraph to delim
pub enum Paragraph {
    /// Defined as two newline characters `"\n\n"`
    Unix,
    /// Defined as `"\r\n\r\n"`
    Windows,
}

/// Type of summary to return
#[allow(dead_code)]
pub enum Summary {
    /// Return whole paragraphs
    Paragraph(Paragraph),
    /// Return individual sentences
    Sentence,
    /// A custom delimiter pattern
    Pattern(String),
}

fn read<T: AsRef<std::path::Path> + Sized>(path: T) -> std::io::Result<String> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

/// Word separators
fn is_delimiter(c: char) -> bool {
    match c {
        '.' | '!' | '?' | ',' | ';' | ')' | 
        '(' | '{' | '}' | '[' | ']' | ':' | 
        '"' | '\'' | '\r' | '\n' | '\t' | ' ' => true,
        _ => false
    }
}

/// Return a word count HashMap
fn freq_analysis<'a, I: Iterator<Item=&'a str> + Sized>(wordlist: I) -> HashMap<&'a str, usize> {
    let mut map: HashMap<&str, usize> = HashMap::new();
    for word in wordlist {
        let count = map.entry(word).or_insert(0);
        *count += 1;
    };
    map
}

/// Analyze a piece of text for the most important paragraphs or sentences
/// 
/// * `exclude` - A string slice containing words to be excluded from analysis
/// 
/// * `text` - A string slice containing the text to be analyzed
/// 
/// * `summary` - Type of summary result to be returned
/// 
pub fn analyze<'a>(exclude: &str, text: &'a str, summary: Summary) -> std::io::Result<Vec<Digest<'a>>> {
    let paragraphs = match summary {
        Summary::Paragraph(Paragraph::Windows) => text.split("\r\n\r\n").collect::<Vec<&str>>(), 
        Summary::Paragraph(Paragraph::Unix) => text.split("\n\n").collect::<Vec<&str>>(), 
        Summary::Sentence   => text.split(|c| c == '.' || c == '?' || c == '!').collect::<Vec<&str>>(),
        Summary::Pattern(p) => text.split(&p).collect::<Vec<&str>>(),
    };
    let mut words = freq_analysis(text.split(is_delimiter));
    let mut scores: Vec<Digest> = Vec::new();

    // Remove excluded words from the HashMap
    for c in exclude.split(is_delimiter) {
        words.remove(c);
    }

    // Enumerate through the paragraphs. We include the index so that we can later sort the paragraphs
    // in order of their occurence in the text, if so desired
    for (index, paragraph) in paragraphs.into_iter().enumerate() {
        let mut score = paragraph.split(is_delimiter).fold(0, |acc, x| acc + *words.entry(x).or_insert(0));
        scores.push(Digest { text: paragraph, score: score, index: index});
    }

    // Sort by highest scoring
    scores.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(scores)
}

/// Run analysis on plain text files
/// 
/// * `exclude_path` - Path to file containing a list of words to be excluded from analysis; e.g. a list of common words
/// 
/// * `text_path` - Path to file containing input text to be analyzed
/// 
/// * `summary` - Type of summary to return
/// 
/// * `take` - How many results to return and combine into output string
pub fn run<T: AsRef<std::path::Path> + Sized>(exclude_path: T, text_path: T, summary: Summary, take: usize) -> std::io::Result<String> {
    match (read(exclude_path), read(text_path)) {
        (Ok(exclude), Ok(text)) => {
            let pg = analyze(&exclude, &text, summary)?;
            let mut top = pg.into_iter().take(take).collect::<Vec<Digest>>();
            top.sort_by(|a, b| a.index.cmp(&b.index));
            Ok(top.into_iter().map(|p| p.text).collect::<Vec<&str>>().join("\n\n"))            
        },
        (e, Ok(_)) => e,
        (Ok(_), e) => e,
        (e, _) => e,
    }
}

fn main() {
    println!("{}", run("common.txt", "test.txt", Summary::Paragraph(Paragraph::Windows), 5).expect("Error!"));
}
