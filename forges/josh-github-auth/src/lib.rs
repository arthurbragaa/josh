pub mod app_flow;
pub mod device_flow;
pub mod middleware;

pub const APP_CLIENT_ID: &str = "Ov23lijvAWwDiQDwZGhN";

/// Check if the given URL is a GitHub URL.
pub fn is_github_url(url: &str) -> bool {
    if url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.host_str()
                .map(|h| h == "github.com" || h.ends_with(".github.com"))
        })
        .unwrap_or(false)
    {
        return true;
    }

    url.strip_prefix("git@github.com:")
        .is_some_and(|path| !path.is_empty())
}

#[cfg(test)]
mod tests {
    use super::is_github_url;

    #[test]
    fn detects_github_code_button_urls() {
        assert!(is_github_url("https://github.com/josh-project/josh.git"));
        assert!(is_github_url("git@github.com:josh-project/josh.git"));
        assert!(!is_github_url("git@gitlab.com:josh-project/josh.git"));
    }
}
