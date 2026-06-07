//! Caveman层 - 语义压缩：剥语法留事实

use regex::Regex;

pub struct CavemanConfig { pub level: u8 }
impl Default for CavemanConfig { fn default() -> Self { Self { level: 2 } } }

pub fn compress(text: &str) -> String { compress_with_config(text, &CavemanConfig::default()) }

pub fn compress_with_config(text: &str, config: &CavemanConfig) -> String {
    let mut result = text.to_string();
    result = Regex::new(r"\b(very|quite|essentially|basically|actually|really|simply|just|particular|especially)\b").unwrap().replace_all(&result, "").to_string();
    result = Regex::new(r"\b(a|an|the|this|that|these|those)\b").unwrap().replace_all(&result, "").to_string();
    result = Regex::new(r"\b(therefore|however|moreover|furthermore|consequently|thus|hence|accordingly|nonetheless|nevertheless|simultaneously|subsequently|elsewhere|thereby|hereby)\b").unwrap().replace_all(&result, "").to_string();
    result = Regex::new(r"\b(in order to|due to the fact that|at this point in time|in the event that|for the purpose of)\b").unwrap().replace_all(&result, "").to_string();
    result = Regex::new(r"\b(is designed to|is intended to|is used to|has the ability to|has the capacity to)\b").unwrap().replace_all(&result, "").to_string();
    result = Regex::new(r"\s+").unwrap().replace_all(&result.trim(), " ").to_string();
    let lines: Vec<&str> = result.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    result = lines.join(" ");
    if config.level >= 3 {
        result = Regex::new(r"\bIn order to\b").unwrap().replace_all(&result, "To").to_string();
        result = Regex::new(r"\b(It is important to|It is necessary to|It is worth noting that)\b").unwrap().replace_all(&result, "").to_string();
    }
    result.trim().to_string()
}
pub fn compress_text(text: &str) -> String { compress(text) }
pub fn compress_level(text: &str, level: u8) -> String { compress_with_config(text, &CavemanConfig { level }) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_article_removal() { assert!(!compress("The system is designed to optimize performance.").contains("The ")); }
    #[test] fn test_filler_removal() { assert!(compress("This is essentially very important and quite essential.").len() < 50); }
    #[test] fn test_connective_removal() { let r = compress("Therefore, we should proceed. However, there are issues."); assert!(!r.contains("Therefore")); }
    #[test] fn test_keeps_facts() { let r = compress("The database handles 1 million requests per second. Located in San Francisco."); assert!(r.contains("1 million")); }
}
