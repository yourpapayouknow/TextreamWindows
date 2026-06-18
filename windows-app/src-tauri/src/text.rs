use crate::models::WordItem;

pub fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x4E00..=0x9FFF
            | 0x3400..=0x4DBF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0xF900..=0xFAFF
            | 0x2F800..=0x2FA1F
            | 0x3040..=0x309F
            | 0x30A0..=0x30FF
            | 0xAC00..=0xD7AF
    )
}

pub fn split_text_into_words(text: &str) -> Vec<String> {
    let tokens = text.split_whitespace();
    let mut result = Vec::new();

    for token in tokens {
        if !token.chars().any(is_cjk) {
            result.push(token.to_string());
            continue;
        }

        let mut buffer = String::new();
        for ch in token.chars() {
            if is_cjk(ch) {
                if !buffer.is_empty() {
                    result.push(std::mem::take(&mut buffer));
                }
                result.push(ch.to_string());
            } else {
                buffer.push(ch);
            }
        }
        if !buffer.is_empty() {
            result.push(buffer);
        }
    }

    result
}

pub fn is_annotation_word(word: &str) -> bool {
    if word.starts_with('[') && word.ends_with(']') {
        return true;
    }
    !word.chars().any(|ch| ch.is_alphanumeric())
}

pub fn build_word_items(words: &[String]) -> Vec<WordItem> {
    let mut offset = 0;
    words
        .iter()
        .enumerate()
        .map(|(id, word)| {
            let item = WordItem {
                id,
                word: word.clone(),
                char_offset: offset,
                is_annotation: is_annotation_word(word),
            };
            offset += word.chars().count() + 1;
            item
        })
        .collect()
}

pub fn total_char_count(words: &[String]) -> usize {
    words.join(" ").chars().count()
}

pub fn char_offset_for_word_progress(words: &[String], progress: f64) -> usize {
    let total = total_char_count(words);
    let whole_word = progress.floor().max(0.0) as usize;
    let frac = progress - whole_word as f64;
    let mut offset = 0;

    for word in words.iter().take(whole_word.min(words.len())) {
        offset += word.chars().count() + 1;
    }

    if whole_word < words.len() {
        offset += (words[whole_word].chars().count() as f64 * frac.clamp(0.0, 1.0)) as usize;
    }

    offset.min(total)
}

pub fn word_progress_for_char_offset(words: &[String], char_offset: usize) -> f64 {
    let mut offset = 0;
    for (index, word) in words.iter().enumerate() {
        let word_len = word.chars().count();
        let end = offset + word_len;
        if char_offset <= end {
            let frac = (char_offset.saturating_sub(offset)) as f64 / word_len.max(1) as f64;
            return index as f64 + frac;
        }
        offset = end + 1;
    }
    words.len() as f64
}

pub fn normalize_for_matching(input: &str) -> String {
    input
        .chars()
        .filter_map(|ch| {
            if ch.is_alphanumeric() || ch.is_whitespace() {
                Some(ch.to_lowercase().collect::<String>())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_cjk_chars_inside_mixed_tokens() {
        let words = split_text_into_words("Hello世界 test 日本語abc");
        assert_eq!(words, vec!["Hello", "世", "界", "test", "日", "本", "語", "abc"]);
    }

    #[test]
    fn detects_annotations_and_emoji() {
        assert!(is_annotation_word("[pause]"));
        assert!(is_annotation_word("✓"));
        assert!(!is_annotation_word("hello"));
    }

    #[test]
    fn maps_progress_to_offsets_and_back() {
        let words = vec!["one".to_string(), "three".to_string()];
        assert_eq!(char_offset_for_word_progress(&words, 1.5), 6);
        assert!((word_progress_for_char_offset(&words, 6) - 1.4).abs() < 0.001);
    }

    #[test]
    fn normalizes_for_speech_matching() {
        assert_eq!(normalize_for_matching("Hello, WORLD!  [pause]"), "hello world pause");
    }
}

