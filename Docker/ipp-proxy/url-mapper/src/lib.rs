use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::collections::HashSet;
use regex::Regex;
use const_format::concatcp;

const ACCESS_TOKEN_FILE_DEFAULT: &str = "/etc/squid/ipp_access_tokens";

// Terminology
// ----------
// Frontend: Connection between PC and proxy
// Backend: Connection between proxy and printer

// Authentication keys consist of numbers, letters, hyphens, and underscores, 8 to 80 characters in length.
const ACCESS_TOKEN_PATTERN: &str = "[0-9A-Za-z_-]{8,80}";
// Equivalent to format!("^((?:ipps?|https?)://[^/]+)(/ipp/({token}).*)$", token = &ACCESS_TOKEN_PATTERN)
const FRONTEND_URL_PATTERN: &str = concatcp!("^((?:ipps?|https?)://[^/]+)(/ipp/(", ACCESS_TOKEN_PATTERN, ").*)$");
// Replace the path portion of the URL with this string
const BACKEND_PATH: &str = "${1}/ipp/print";

#[derive(Debug)]
pub struct UrlTransform {
    credentials_db: HashSet<String>,
    url_regex: Regex,
    backend_path: String
}

impl UrlTransform {

    fn new(access_tokens: HashSet<String>, url_regex: Regex, backend_path: &str) -> Self {
        UrlTransform { credentials_db: access_tokens, url_regex, backend_path: backend_path.to_string() }
    }

    fn transform(&self, _channel: &str, url: &str, _ip: &str, _ident: &str, _method: &str) -> Option<String> {
        let mut new_url = None;
        // Check that the Auth Token in the URL is valid
        if !self.credentials_db.is_empty() {
            if self.url_regex.is_match(url) {
                let captures = self.url_regex.captures(url).unwrap();
                let access_token = &captures[3];
                if self.credentials_db.contains(&access_token.to_string()) {
                    // For the backend, substitute a URL the printer expects and which doesn't leak
                    // the access token.
                    new_url = Some(self.url_regex.replace(url, &self.backend_path).to_string());
                }
            }
        }
        new_url
    }
}


pub fn process_urls() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let access_token_file = env::var("IPP_ACCESS_TOKEN_FILE")
                                        .unwrap_or(ACCESS_TOKEN_FILE_DEFAULT.to_string());

    // "Access tokens" are what the client sends us in an attempt to authenticate, and which we
    // validate against the key database.
    let access_token_regex = Regex::new(format!("^{}$", &ACCESS_TOKEN_PATTERN).as_str()).unwrap();
    let access_tokens: HashSet<String> = BufReader::new(
        File::open(&access_token_file)
                .expect(format!("Failed to open {}", access_token_file).as_str())
    )
    .lines()
    .map(|line| line.expect("Failed to read line").trim().to_string())
    .filter(|key| access_token_regex.is_match(key))
    .collect();
    // dbg!(&access_tokens);

    let frontend_url_regex = Regex::new(&FRONTEND_URL_PATTERN).unwrap();
    let url_transform = UrlTransform::new(access_tokens, frontend_url_regex, BACKEND_PATH);

    // Read lines from stdin
    for line in stdin.lock().lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Ensure the line has at least 5 parts (channel, url, ip, ident, method)
        if parts.len() < 5 {
            continue; // Skip invalid lines
        }

        let channel = parts[0];
        let url = parts[1];
        let ip = parts[2];
        let ident = parts[3];
        let method = parts[4];

        // As a possible enhancement to support multiple printers at different URL paths,
        // if..else cases could be inserted here based on `url`.
        let new_url = url_transform.transform(
                                            &channel,
                                            &url,
                                            &ip,
                                            &ident,
                                            &method);

        /* Expected output of a url_rewrite_program:
            OK rewrite-url="new-url" to rewrite the URL.
            OK to leave it unchanged.
            ERR to deny the request.
        */
        match new_url {
            // The access token was valid. Provide the URL *WITHOUT* the access token to send to the printer.
            Some(new_url) => {
                writeln!(stdout, "{} OK rewrite-url=\"{}\"", channel, new_url)
                    .expect("Failed to write to stdout");
            }
            // The access token could not be validated. Deny the request.
            None => {
                writeln!(stdout, "{} ERR", channel).expect("Failed to write to stdout");
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform() {
        let mut tokens = HashSet::<String>::new();
        tokens.insert("WWWWWWWW".to_string());
        tokens.insert("XXXXXXXX".to_string());
        tokens.insert("YYYYYYYY".to_string());
        tokens.insert("ZZZZZZZZ".to_string());

        let url_transform = UrlTransform::new(tokens,
            Regex::new(&FRONTEND_URL_PATTERN).unwrap(),
            BACKEND_PATH
        );

        // test_transform_good()
        assert_eq!(url_transform.transform("1", "ipps://11.11.11.11/ipp/YYYYYYYY", "11.11.11.11", "-", "GET"), Some("ipps://11.11.11.11/ipp/print".to_string()));
        assert_eq!(url_transform.transform("1", "https://11.11.11.11/ipp/YYYYYYYY/", "11.11.11.11", "-", "GET"), Some("https://11.11.11.11/ipp/print".to_string()));

        // test_transform_unknown_token()
        assert_eq!(url_transform.transform("1", "ipps://11.11.11.11/ipp/AAAAAAAA", "11.11.11.11", "-", "GET"), None);

        // test_transform_bad_url()
        assert_eq!(url_transform.transform("1", "ipps://11.11.11.11/krugman/YYYYYYYY", "11.11.11.11", "-", "GET"), None);
    }

}
