//! Link extraction from HTML content.

use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

/// Extract all links from HTML content.
///
/// Parses the HTML and extracts all `<a href="...">` links,
/// resolving relative URLs against the base URL.
pub fn extract_links(html_content: &str, base_url: &str) -> Vec<String> {
    let document = Html::parse_document(html_content);
    let selector = match Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let base = Url::parse(base_url).ok();
    let mut links = HashSet::new();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Some(absolute_url) = resolve_url(href, &base) {
                links.insert(absolute_url);
            }
        }
    }

    links.into_iter().collect()
}

/// Resolve a potentially relative URL against a base URL.
fn resolve_url(href: &str, base: &Option<Url>) -> Option<String> {
    let href = href.trim();

    // Skip non-HTTP schemes, anchors, and special URLs
    if href.is_empty()
        || href.starts_with('#')
        || href.starts_with("javascript:")
        || href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("data:")
        || href.starts_with("blob:")
    {
        return None;
    }

    // Try parsing as absolute URL first
    if let Ok(url) = Url::parse(href) {
        if url.scheme() == "http" || url.scheme() == "https" {
            return Some(normalize_url(&url));
        }
        return None;
    }

    // Resolve relative URL against base
    if let Some(base_url) = base {
        if let Ok(resolved) = base_url.join(href) {
            if resolved.scheme() == "http" || resolved.scheme() == "https" {
                return Some(normalize_url(&resolved));
            }
        }
    }

    None
}

/// Normalize URL by removing fragment and trailing slashes.
fn normalize_url(url: &Url) -> String {
    let mut normalized = url.clone();
    normalized.set_fragment(None);

    let mut result = normalized.to_string();

    // Remove trailing slash for consistency (except for root paths)
    if result.ends_with('/') && result.chars().filter(|c| *c == '/').count() > 3 {
        result.pop();
    }

    result
}

/// Check if two URLs are on the same domain/host.
pub fn is_same_domain(url1: &str, url2: &str) -> bool {
    let parsed1 = Url::parse(url1).ok();
    let parsed2 = Url::parse(url2).ok();

    match (parsed1, parsed2) {
        (Some(u1), Some(u2)) => u1.host_str() == u2.host_str(),
        _ => false,
    }
}

/// Filter links to only include same-domain URLs.
pub fn filter_same_domain(links: Vec<String>, base_url: &str) -> Vec<String> {
    links
        .into_iter()
        .filter(|link| is_same_domain(link, base_url))
        .collect()
}

/// Get the domain/host from a URL.
pub fn get_domain(url: &str) -> Option<String> {
    Url::parse(url).ok().and_then(|u| u.host_str().map(|h| h.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links_absolute() {
        let html = r#"
            <html>
            <body>
                <a href="https://example.com/page1">Page 1</a>
                <a href="https://example.com/page2">Page 2</a>
            </body>
            </html>
        "#;

        let links = extract_links(html, "https://example.com/");
        assert!(links.contains(&"https://example.com/page1".to_string()));
        assert!(links.contains(&"https://example.com/page2".to_string()));
    }

    #[test]
    fn test_extract_links_relative() {
        let html = r#"
            <html>
            <body>
                <a href="/page1">Page 1</a>
                <a href="page2">Page 2</a>
                <a href="../page3">Page 3</a>
            </body>
            </html>
        "#;

        let links = extract_links(html, "https://example.com/dir/index.html");
        assert!(links.contains(&"https://example.com/page1".to_string()));
        assert!(links.contains(&"https://example.com/dir/page2".to_string()));
        assert!(links.contains(&"https://example.com/page3".to_string()));
    }

    #[test]
    fn test_extract_links_filters_invalid() {
        let html = r##"
            <html>
            <body>
                <a href="#anchor">Anchor</a>
                <a href="javascript:void(0)">JS</a>
                <a href="mailto:test@example.com">Email</a>
                <a href="tel:+1234567890">Phone</a>
                <a href="https://example.com/valid">Valid</a>
            </body>
            </html>
        "##;

        let links = extract_links(html, "https://example.com/");
        assert_eq!(links.len(), 1);
        assert!(links.contains(&"https://example.com/valid".to_string()));
    }

    #[test]
    fn test_same_domain() {
        assert!(is_same_domain(
            "http://192.168.1.1/page1",
            "http://192.168.1.1/page2"
        ));
        assert!(is_same_domain(
            "https://example.com/a",
            "https://example.com/b"
        ));
        assert!(!is_same_domain(
            "http://192.168.1.1/",
            "http://192.168.1.2/"
        ));
        assert!(!is_same_domain(
            "https://example.com/",
            "https://other.com/"
        ));
    }

    #[test]
    fn test_filter_same_domain() {
        let links = vec![
            "https://example.com/page1".to_string(),
            "https://example.com/page2".to_string(),
            "https://other.com/page3".to_string(),
        ];

        let filtered = filter_same_domain(links, "https://example.com/");
        assert_eq!(filtered.len(), 2);
        assert!(!filtered.contains(&"https://other.com/page3".to_string()));
    }

    #[test]
    fn test_ip_address_domain() {
        assert!(is_same_domain(
            "http://10.0.0.1:8080/page1",
            "http://10.0.0.1:8080/page2"
        ));
        // Different ports are same host
        assert!(is_same_domain(
            "http://10.0.0.1:8080/",
            "http://10.0.0.1:9090/"
        ));
    }

    #[test]
    fn test_get_domain() {
        assert_eq!(
            get_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            get_domain("http://192.168.1.1:8080/page"),
            Some("192.168.1.1".to_string())
        );
        assert_eq!(get_domain("invalid"), None);
    }
}
