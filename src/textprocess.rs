use regex::Regex;

pub struct X1TextProcess {
    lowercase: Regex,
    uppercase: Regex,
    digit: Regex,
    special: Regex,
}

impl X1TextProcess {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            lowercase: Regex::new(r"[a-z]")?,
            uppercase: Regex::new(r"[A-Z]")?,
            digit: Regex::new(r"\d")?,
            special: Regex::new(r"[@$!%*?&]")?,
        })
    }

    pub fn sanitize(&self, text: &str) -> String {
        text.trim()
            .replace('\n', "")
            .replace('\t', "")
            .replace('\r', "")
    }

    pub fn is_valid_text(&self, text: &str, max_len: usize) -> bool {
        !text.is_empty()
            && !text.chars().all(|c| c.is_whitespace())
            && text.len() >= 10
            && text.len() <= max_len
    }

    pub fn is_password(&self, text: &str) -> bool {
        if text.contains(' ') || text.contains('\n') {
            return false;
        }

        if !(8..=128).contains(&text.len()) {
            return false;
        }

        let mut score = 0;

        if self.lowercase.is_match(text) {
            score += 1;
        }

        if self.uppercase.is_match(text) {
            score += 1;
        }

        if self.digit.is_match(text) {
            score += 1;
        }

        if self.special.is_match(text) {
            score += 1;
        }

        score >= 3
    }
}
