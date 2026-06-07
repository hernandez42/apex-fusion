//! Caveman层 - 语义压缩：剥语法留事实
//! 移植自 wilpel/caveman-compression
//! 
//! 核心原理：LLM能预测的语法结构全部删除，只保留不可预测的事实内容
//! - 冠词(a, an, the) → 删除
//! - 连接词(therefore, however, because) → 删除
//! - 被动语态 → 转主动
//! - 填充词(very, quite, essentially) → 删除
//! - 介词短语可简化 → 简化

use regex::Regex;

/// Caveman压缩配置
pub struct CavemanConfig {
    /// 压缩级别 1-3 (1=轻, 3=强)
    pub level: u8,
}

impl Default for CavemanConfig {
    fn default() -> Self {
        Self { level: 2 }
    }
}

/// 压缩文本（Caveman策略）
pub fn compress(text: &str) -> String {
    compress_with_config(text, &CavemanConfig::default())
}

/// 带配置的压缩
pub fn compress_with_config(text: &str, config: &CavemanConfig) -> String {
    let mut result = text.to_string();

    // 1. 去除填充词和弱化词
    let filler_re = Regex::new(r"\b(very|quite|essentially|basically|actually|really|simply|just|simply|particular|especially)\b").unwrap();
    result = filler_re.replace_all(&result, "").to_string();

    // 2. 去除指示词
    let article_re = Regex::new(r"\b(a|an|the|this|that|these|those)\b").unwrap();
    result = article_re.replace_all(&result, "").to_string();

    // 3. 去除冗余连接词
    let connective_re = Regex::new(r"\b(therefore|however|moreover|furthermore|consequently|thus|hence|accordingly|nonetheless|nevertheless|simultaneously|subsequently|elsewhere|thereby|hereby)\b").unwrap();
    result = connective_re.replace_all(&result, "").to_string();

    // 4. 简化介词短语
    let prep_re = Regex::new(r"\b(in order to|due to the fact that|at this point in time|in the event that|for the purpose of)\b").unwrap();
    result = prep_re.replace_all(&result, "").to_string();

    // 5. 去除冗余动词
    let verb_re = Regex::new(r"\b(is designed to|is intended to|is used to|has the ability to|has the capacity to)\b").unwrap();
    result = verb_re.replace_all(&result, "").to_string();

    // 6. 清理多余空白
    let ws_re = Regex::new(r"\s+").unwrap();
    result = ws_re.replace_all(&result.trim(), " ").to_string();

    // 7. 去除行首行尾标点空白
    let lines: Vec<&str> = result.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    result = lines.join(" ");

    // 8. 强化压缩（level 3时）
    if config.level >= 3 {
        // 去除"In order to"类
        let inorder_re = Regex::new(r"\bIn order to\b").unwrap();
        result = inorder_re.replace_all(&result, "To").to_string();
        
        // 去除"it is important to"类
        let imp_re = Regex::new(r"\b(It is important to|It is necessary to|It is worth noting that)\b").unwrap();
        result = imp_re.replace_all(&result, "").to_string();
    }

    result.trim().to_string()
}

/// 压缩文本（公开API）
pub fn compress_text(text: &str) -> String {
    compress(text)
}

/// 压缩文本（指定级别）
pub fn compress_level(text: &str, level: u8) -> String {
    compress_with_config(text, &CavemanConfig { level })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_removal() {
        let input = "The system is designed to optimize performance.";
        let result = compress(input);
        assert!(!result.contains("The "));
    }

    #[test]
    fn test_filler_removal() {
        let input = "This is essentially very important and quite essential.";
        let result = compress(input);
        assert!(result.len() < input.len());
    }

    #[test]
    fn test_connective_removal() {
        let input = "Therefore, we should proceed. However, there are issues.";
        let result = compress(input);
        assert!(!result.contains("Therefore"));
        assert!(!result.contains("However"));
    }

    #[test]
    fn test_keeps_facts() {
        let input = "The database handles 1 million requests per second. Located in San Francisco.";
        let result = compress(input);
        assert!(result.contains("1 million"));
        assert!(result.contains("San Francisco"));
    }
}
