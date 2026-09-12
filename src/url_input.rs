use std::fmt;

use url::Url;

const ALLOWED_SCHEMES: [&str; 2] = ["http", "https"];

#[derive(Debug, PartialEq, Eq)]
pub enum UrlInputError {
    Empty,
    InvalidUrl(String),
    UnsupportedScheme(String),
}

impl fmt::Display for UrlInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlInputError::Empty => write!(f, "URLを入力してください"),
            UrlInputError::InvalidUrl(raw) => write!(f, "URLとして解釈できません: {raw}"),
            UrlInputError::UnsupportedScheme(scheme) => {
                write!(f, "許可されていないスキームです: {scheme}（http / https のみ許可）")
            }
        }
    }
}

/// ユーザーが手打ちしたURL文字列を正規化・検証し、遷移可能なURL文字列を返す。
///
/// スキームを省略した入力には `https://` を補う。`http`/`https` 以外の
/// スキーム（`file:`, `javascript:` など）は配信者のローカルファイル露出や
/// スクリプト注入につながるため拒否する。
pub fn normalize_and_validate(raw: &str) -> Result<String, UrlInputError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(UrlInputError::Empty);
    }

    // コロンの直後が数字（ポート番号）の場合のみ "host:port" 形式とみなして
    // https:// を補う（"localhost:8080" はコロンの前がスキームと誤解釈される
    // ため）。それ以外にコロンを含む入力（"file:...", "javascript:..." 等）は
    // スキーム付きとみなしそのまま解釈し、危険なスキームを確実に検出できる
    // ようにする。コロンを含まない入力にも https:// を補う。
    let candidate = if !trimmed.contains(':') || has_numeric_port(trimmed) {
        format!("https://{trimmed}")
    } else {
        trimmed.to_string()
    };

    let url = Url::parse(&candidate).map_err(|_| UrlInputError::InvalidUrl(trimmed.to_string()))?;

    if !ALLOWED_SCHEMES.contains(&url.scheme()) {
        return Err(UrlInputError::UnsupportedScheme(url.scheme().to_string()));
    }

    if url.host_str().is_none() {
        return Err(UrlInputError::InvalidUrl(trimmed.to_string()));
    }

    Ok(url.to_string())
}

/// 最初のコロンの直後がポート番号（数字のみ）かどうかを判定する。
fn has_numeric_port(s: &str) -> bool {
    match s.find(':') {
        Some(pos) => {
            let after = &s[pos + 1..];
            let port_part = after.split(['/', '?', '#']).next().unwrap_or("");
            !port_part.is_empty() && port_part.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_rejected() {
        assert_eq!(normalize_and_validate(""), Err(UrlInputError::Empty));
        assert_eq!(normalize_and_validate("   "), Err(UrlInputError::Empty));
    }

    #[test]
    fn bare_host_gets_https_prefix() {
        assert_eq!(
            normalize_and_validate("example.com").unwrap(),
            "https://example.com/"
        );
    }

    #[test]
    fn surrounding_whitespace_is_trimmed() {
        assert_eq!(
            normalize_and_validate("  https://example.com  ").unwrap(),
            "https://example.com/"
        );
    }

    #[test]
    fn http_scheme_is_allowed() {
        assert_eq!(
            normalize_and_validate("http://example.com").unwrap(),
            "http://example.com/"
        );
    }

    #[test]
    fn file_scheme_is_rejected() {
        assert_eq!(
            normalize_and_validate("file:///etc/passwd"),
            Err(UrlInputError::UnsupportedScheme("file".to_string()))
        );
    }

    #[test]
    fn javascript_scheme_is_rejected() {
        assert_eq!(
            normalize_and_validate("javascript:alert(1)"),
            Err(UrlInputError::UnsupportedScheme("javascript".to_string()))
        );
    }

    #[test]
    fn ftp_scheme_is_rejected() {
        assert_eq!(
            normalize_and_validate("ftp://example.com"),
            Err(UrlInputError::UnsupportedScheme("ftp".to_string()))
        );
    }

    #[test]
    fn malformed_url_is_rejected() {
        assert!(matches!(
            normalize_and_validate("not a url with spaces"),
            Err(UrlInputError::InvalidUrl(_))
        ));
    }

    #[test]
    fn path_and_query_are_preserved() {
        assert_eq!(
            normalize_and_validate("example.com/path?q=1").unwrap(),
            "https://example.com/path?q=1"
        );
    }

    #[test]
    fn host_port_form_is_treated_as_host_not_scheme() {
        assert_eq!(
            normalize_and_validate("localhost:8080").unwrap(),
            "https://localhost:8080/"
        );
        assert_eq!(
            normalize_and_validate("127.0.0.1:4455").unwrap(),
            "https://127.0.0.1:4455/"
        );
    }
}
