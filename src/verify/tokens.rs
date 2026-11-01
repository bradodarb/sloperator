//! The token multisets an edit must preserve: numbers, URLs and configured terms (SPEC §8, FR-8.5).
//!
//! Each is a multiset (a count per token), so a rewrite that reorders its tokens still compares equal.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::doc::{BARE_URL, ProtectSettings};

/// Digit dates are listed first so `2024-01-05` is one token, not three numbers.
const NUMBER: &str = r"\d{1,4}[-/.]\d{1,2}[-/.]\d{1,4}|\d+(?:[.,]\d+)*%?";

static NUMBER_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(NUMBER) {
    Ok(re) => re,
    Err(e) => unreachable!("NUMBER is a constant and must compile: {e}"),
});

static URL_RE: LazyLock<Regex> = LazyLock::new(|| match Regex::new(BARE_URL) {
    Ok(re) => re,
    Err(e) => unreachable!("BARE_URL is a constant and must compile: {e}"),
});

/// Numbers, URLs and terms found in a piece of text, each as token -> occurrences.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Tokens {
    pub numbers: BTreeMap<String, u32>,
    pub urls: BTreeMap<String, u32>,
    pub terms: BTreeMap<String, u32>,
}

fn bump(map: &mut BTreeMap<String, u32>, key: &str, by: usize) {
    let slot = map.entry(key.to_owned()).or_insert(0);
    *slot = slot.saturating_add(u32::try_from(by).unwrap_or(u32::MAX));
}

/// The numbers (`\d+(?:[.,]\d+)*%?` and digit dates, dates first), URLs (`doc::BARE_URL`, trailing `.,;:!?)`
/// trimmed like the span map does) and configured terms (non-overlapping occurrences of each) of `text`.
///
/// URLs are cut out before numbers are read, so the digits of a URL count once, as part of the URL.
pub fn tokens(text: &str, protect: &ProtectSettings) -> Tokens {
    let mut out = Tokens::default();
    let mut rest = String::with_capacity(text.len());
    let mut at = 0;
    for m in URL_RE.find_iter(text) {
        let url = m
            .as_str()
            .trim_end_matches(['.', ',', ';', ':', '!', '?', ')']);
        if url.is_empty() {
            continue;
        }
        rest.push_str(&text[at..m.start()]);
        rest.push(' ');
        at = m.start() + url.len();
        bump(&mut out.urls, url, 1);
    }
    rest.push_str(&text[at..]);
    for m in NUMBER_RE.find_iter(&rest) {
        bump(&mut out.numbers, m.as_str(), 1);
    }
    for term in protect.terms.iter().filter(|t| !t.is_empty()) {
        let n = text.matches(term.as_str()).count();
        if n > 0 {
            bump(&mut out.terms, term, n);
        }
    }
    out
}

/// What changed between two multisets, e.g. `lost 42 ×1; added 43 ×1`.
pub(super) fn diff(a: &BTreeMap<String, u32>, b: &BTreeMap<String, u32>) -> String {
    let mut parts = Vec::new();
    for (k, &n) in a {
        let m = b.get(k).copied().unwrap_or(0);
        if n > m {
            parts.push(format!("lost {k} ×{}", n - m));
        }
    }
    for (k, &m) in b {
        let n = a.get(k).copied().unwrap_or(0);
        if m > n {
            parts.push(format!("added {k} ×{}", m - n));
        }
    }
    parts.join("; ")
}
