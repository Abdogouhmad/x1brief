use regex::Regex;

pub struct X1BriefRegex {
    lowercase: Regex,
    uppercase: Regex,
    digit: Regex,
    special: Regex,
}

impl X1BriefRegex {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            lowercase: Regex::new(r"[a-z]")?,
            uppercase: Regex::new(r"[A-Z]")?,
            digit: Regex::new(r"\d")?,
            special: Regex::new(r"[@$!%*?&]")?,
        })
    }


    pub fn is_password(&self, text: &str) -> bool {
        // Reject obvious normal sentences
        if text.contains(' ') || text.contains('\n') {
            return false;
        }

        // Passwords are usually not gigantic
        if text.len() < 8 || text.len() > 128 {
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

        // Require strong mix
        score >= 3
    }
}
