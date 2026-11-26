// Fighting slop with slop, disabled by default, but this captcha works really well! (tested on prod)

use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

const CHALLENGE_TTL_SECS: u64 = 600;
const MAX_CHALLENGE_TOKEN_LEN: usize = 256;

pub fn content_has_link(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    lower.contains("http://") || lower.contains("https://")
}

fn captcha_secret() -> u64 {
    static SECRET: OnceLock<u64> = OnceLock::new();
    *SECRET.get_or_init(|| {
        use rand::Rng;
        rand::thread_rng().gen::<u64>()
    })
}

fn sign(data: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    captcha_secret().hash(&mut hasher);
    data.hash(&mut hasher);
    captcha_secret().hash(&mut hasher);
    hasher.finish()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

const MIN_ANSWER_DIGITS: usize = 2;
const MAX_ANSWER_DIGITS: usize = 10;

fn normalize_answer(answer: &str) -> String {
    answer.trim().to_string()
}

fn is_valid_answer_format(answer: &str) -> bool {
    let trimmed = answer.trim();
    (MIN_ANSWER_DIGITS..=MAX_ANSWER_DIGITS).contains(&trimmed.len())
        && trimmed.bytes().all(|b| b.is_ascii_digit())
}

pub struct Challenge {
    pub image_base64: String,
    pub token: String,
}

pub fn new_challenge() -> Option<Challenge> {
    use captcha::filters::Noise;
    use captcha::Captcha;
    use rand::Rng;

    const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    let len = rand::thread_rng().gen_range(2..=3);

    let mut captcha = Captcha::new();
    captcha.set_chars(&DIGITS);
    for _ in 0..len {
        captcha.add_char();
    }
    captcha.apply_filter(Noise::new(0.1)).view(220, 120);

    let solution = normalize_answer(&captcha.chars_as_string());
    let image_base64 = captcha.as_base64()?;

    let token = make_token(&solution);

    Some(Challenge {
        image_base64,
        token,
    })
}

// Token format: <solution>:<issued_at>.<hex_signature>
fn make_token(solution: &str) -> String {
    let issued_at = now_secs();
    let payload = format!("{}:{}", solution, issued_at);
    let signature = sign(&payload);
    format!("{}.{:x}", payload, signature)
}

pub fn verify(token: &str, answer: &str) -> bool {
    if token.is_empty() || token.len() > MAX_CHALLENGE_TOKEN_LEN {
        return false;
    }

    let (payload, provided_sig) = match token.rsplit_once('.') {
        Some(parts) => parts,
        None => return false,
    };

    let expected_sig = format!("{:x}", sign(payload));
    if provided_sig != expected_sig {
        return false;
    }

    let (solution, issued_at) = match payload.rsplit_once(':') {
        Some(parts) => parts,
        None => return false,
    };

    let issued_at: u64 = match issued_at.parse() {
        Ok(ts) => ts,
        Err(_) => return false,
    };

    if now_secs().saturating_sub(issued_at) > CHALLENGE_TTL_SECS {
        return false;
    }

    if !is_valid_answer_format(answer) {
        return false;
    }

    normalize_answer(answer) == solution
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_http_and_https_links() {
        assert!(content_has_link("see http://example.com"));
        assert!(content_has_link("see https://example.com"));
        assert!(content_has_link("HTTPS://EXAMPLE.COM shouting"));
        assert!(content_has_link("[link](http://x.y)"));
    }

    #[test]
    fn ignores_text_without_links() {
        assert!(!content_has_link("just some words"));
        assert!(!content_has_link("mentioning http without a scheme"));
        assert!(!content_has_link("ftp://example.com only"));
    }

    #[test]
    fn verify_accepts_correct_answer() {
        let token = make_token("427");
        assert!(verify(&token, "427"));
        assert!(verify(&token, "  427 "));
    }

    #[test]
    fn verify_rejects_wrong_answer() {
        let token = make_token("427");
        assert!(!verify(&token, "918"));
        assert!(!verify(&token, ""));
    }

    #[test]
    fn verify_rejects_non_digit_answer() {
        let token = make_token("427");
        assert!(!verify(&token, "abc"));
        assert!(!verify(&token, "4a7"));
    }

    #[test]
    fn answer_format_enforces_two_to_ten_digits() {
        assert!(!is_valid_answer_format(""));
        assert!(!is_valid_answer_format("7"));
        assert!(is_valid_answer_format("42"));
        assert!(is_valid_answer_format("0123456789"));
        assert!(!is_valid_answer_format("01234567891"));
        assert!(!is_valid_answer_format("12a"));
        assert!(!is_valid_answer_format("12 34"));
        assert!(is_valid_answer_format("  42  "));
    }

    #[test]
    fn verify_rejects_tampered_token() {
        let token = make_token("427");
        let (_, sig) = token.rsplit_once('.').unwrap();
        let forged = format!("918:{}.{}", now_secs(), sig);
        assert!(!verify(&forged, "918"));
    }

    #[test]
    fn verify_rejects_expired_token() {
        let old_issued = now_secs().saturating_sub(CHALLENGE_TTL_SECS + 60);
        let payload = format!("427:{}", old_issued);
        let token = format!("{}.{:x}", payload, sign(&payload));
        assert!(!verify(&token, "427"));
    }

    #[test]
    fn verify_rejects_malformed_tokens() {
        assert!(!verify("", "42"));
        assert!(!verify("nodot", "42"));
        assert!(!verify(&"a".repeat(MAX_CHALLENGE_TOKEN_LEN + 1), "42"));
    }

    #[test]
    fn new_challenge_round_trips() {
        let challenge = new_challenge().expect("captcha image should render");
        assert!(!challenge.image_base64.is_empty());
        let (payload, _) = challenge.token.rsplit_once('.').unwrap();
        let (solution, _) = payload.rsplit_once(':').unwrap();
        assert!(verify(&challenge.token, solution));
    }
}
