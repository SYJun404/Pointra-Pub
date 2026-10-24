use super::ocr_common::WordBox;
use std::ops::Range;

struct RegionParagraph {
    text: String,
    clipped: Vec<Range<usize>>,
}

/// 有完整句子时过滤残句，否则保留 OCR 内容；两条路径都合并段落内的视觉换行。
pub fn region_translation_text(lines: &[WordBox], width: u32, height: u32) -> String {
    let paragraphs = region_paragraphs(lines, width, height);
    let complete = filtered_paragraphs(&paragraphs);
    if !complete.is_empty() {
        return complete;
    }
    paragraphs
        .iter()
        .map(|paragraph| paragraph.text.as_str())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// 只处理框选 OCR：按行距恢复段落，去掉裁切边缘和没有句末的残句。
/// 这是标点和版面的启发式过滤，无法判断所有语义上不完整的句子。
#[cfg(test)]
fn complete_region_text(lines: &[WordBox], width: u32, height: u32) -> String {
    filtered_paragraphs(&region_paragraphs(lines, width, height))
}

fn filtered_paragraphs(paragraphs: &[RegionParagraph]) -> String {
    paragraphs
        .iter()
        .map(|paragraph| complete_sentences(&paragraph.text, &paragraph.clipped))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn region_paragraphs(lines: &[WordBox], width: u32, height: u32) -> Vec<RegionParagraph> {
    let mut lines: Vec<_> = lines
        .iter()
        .filter(|line| !line.text.trim().is_empty())
        .collect();
    lines.sort_by(|a, b| a.y.total_cmp(&b.y).then_with(|| a.x.total_cmp(&b.x)));
    let mut paragraphs = Vec::new();
    let mut paragraph = RegionParagraph {
        text: String::new(),
        clipped: Vec::new(),
    };
    let mut previous: Option<&WordBox> = None;
    for line in lines {
        if let Some(prev) = previous {
            let gap = line.y - (prev.y + prev.h);
            if gap > prev.h.max(line.h) * 0.9 {
                paragraphs.push(paragraph);
                paragraph = RegionParagraph {
                    text: String::new(),
                    clipped: Vec::new(),
                };
            }
        }
        let edge_x = 1.0 / width.max(1) as f64;
        let edge_y = 1.0 / height.max(1) as f64;
        let clipped = line.x <= edge_x
            || line.y <= edge_y
            || line.x + line.w >= 1.0 - edge_x
            || line.y + line.h >= 1.0 - edge_y;
        for fragment in line.text.lines().map(str::trim) {
            // 显式空行也是段落分隔，不作为普通视觉换行删除。
            if fragment.is_empty() {
                if !paragraph.text.is_empty() {
                    paragraphs.push(paragraph);
                    paragraph = RegionParagraph {
                        text: String::new(),
                        clipped: Vec::new(),
                    };
                }
                continue;
            }
            let begin = if paragraph.text.ends_with('\u{ad}') {
                paragraph.text.len() - '\u{ad}'.len_utf8()
            } else {
                paragraph.text.len()
            };
            append_ocr_line(&mut paragraph.text, fragment);
            if clipped {
                paragraph.clipped.push(begin..paragraph.text.len());
            }
        }
        previous = Some(line);
    }
    if !paragraph.text.is_empty() {
        paragraphs.push(paragraph);
    }
    paragraphs
}

fn append_ocr_line(text: &mut String, content: &str) {
    let first = content.chars().next().unwrap();
    if let Some(last) = text.chars().last() {
        let soft_hyphen = last == '\u{ad}';
        let hard_hyphen = last == '-'
            && first.is_ascii_lowercase()
            && text
                .chars()
                .rev()
                .nth(1)
                .map_or(false, |c| c.is_ascii_alphabetic());
        if soft_hyphen {
            text.pop();
        }
        let closes = closing_mark(first)
            || matches!(
                first,
                ',' | '.' | ';' | ':' | '!' | '?' | '，' | '。' | '；' | '：' | '！' | '？' | '、'
            );
        let opens = matches!(
            last,
            '(' | '[' | '{' | '（' | '【' | '「' | '『' | '“' | '‘'
        );
        if !soft_hyphen && !hard_hyphen && !closes && !opens && !(is_cjk(last) && is_cjk(first)) {
            text.push(' ');
        }
    }
    // 连续空白归一化，避免 OCR 的缩进、Tab 混入句子。
    text.push_str(&content.split_whitespace().collect::<Vec<_>>().join(" "));
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x3040..=0x30ff | 0x3400..=0x9fff | 0x20000..=0x2fa1f)
        || matches!(c, '。' | '！' | '？' | '」' | '』' | '”' | '’')
}

fn closing_mark(c: char) -> bool {
    matches!(
        c,
        '\'' | '"' | '’' | '”' | ')' | ']' | '}' | '）' | '】' | '」' | '』'
    )
}

fn sentence_boundary(text: &str, i: usize, c: char) -> bool {
    if matches!(c, '!' | '?' | '。' | '！' | '？') {
        return true;
    }
    if c != '.' {
        return false;
    }
    let after = &text[i + 1..];
    // 小数、域名、缩写内部的点不结束一句。
    if after.starts_with(|next: char| next.is_alphanumeric() || next == '.') {
        return false;
    }
    let token = text[..=i]
        .split_whitespace()
        .last()
        .unwrap_or("")
        .trim_start_matches(|c: char| !c.is_alphanumeric());
    if matches!(
        token.to_ascii_lowercase().as_str(),
        "mr." | "mrs." | "ms." | "dr." | "prof." | "sr." | "jr." | "e.g." | "i.e." | "vs."
    ) {
        return false;
    }
    let stem = token.trim_end_matches('.');
    // 段落开头的列表编号不是一个完整句子。
    if stem.chars().all(|c| c.is_ascii_digit()) && text[..=i].trim() == token {
        return false;
    }
    let initials = stem
        .split('.')
        .all(|part| part.len() == 1 && part.as_bytes()[0].is_ascii_uppercase());
    if initials {
        let next = after.trim_start().chars().next();
        if (stem.contains('.') && next.map_or(false, |c| c.is_lowercase()))
            || (!stem.contains('.') && next.map_or(false, |c| c.is_uppercase()))
        {
            return false;
        }
    }
    true
}

fn complete_sentences(text: &str, clipped: &[Range<usize>]) -> String {
    let mut sentences = Vec::new();
    let mut begin = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if !sentence_boundary(text, i, c) {
            continue;
        }
        let mut end = i + c.len_utf8();
        while let Some(&(j, next)) = chars.peek() {
            if !closing_mark(next) && !matches!(next, '.' | '!' | '?' | '！' | '？') {
                break;
            }
            end = j + next.len_utf8();
            chars.next();
        }
        let sentence = text[begin..end].trim();
        let starts_mid_sentence = sentence
            .chars()
            .find(|c| c.is_alphanumeric())
            .map_or(true, |c| c.is_ascii_lowercase());
        let crosses_clip = clipped
            .iter()
            .any(|range| range.start < end && range.end > begin);
        if !starts_mid_sentence && !crosses_clip {
            sentences.push(sentence);
        }
        begin = end;
    }
    // 最后一个句末之后的内容故意丢弃，不自动补句号。
    let mut result = String::new();
    for sentence in sentences {
        let chinese_boundary = result.chars().last().map_or(false, is_cjk)
            && sentence.chars().next().map_or(false, is_cjk);
        if !result.is_empty() && !chinese_boundary {
            result.push(' ');
        }
        result.push_str(sentence);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str, y: f64) -> WordBox {
        WordBox {
            text: text.into(),
            x: 0.05,
            y,
            w: 0.9,
            h: 0.05,
        }
    }

    #[test]
    fn falls_back_to_original_ocr_when_no_complete_sentence_exists() {
        assert_eq!(
            region_translation_text(&[line("hello", 0.1)], 1000, 400),
            "hello"
        );
        assert_eq!(
            region_translation_text(
                &[line("second line", 0.18), line("First line", 0.1)],
                1000,
                400
            ),
            "First line second line"
        );
        assert_eq!(region_translation_text(&[], 1000, 400), "");
    }

    #[test]
    fn normalizes_embedded_newlines_and_whitespace() {
        assert_eq!(
            region_translation_text(
                &[line("The sentence\r\n continues\t here.", 0.1)],
                1000,
                400
            ),
            "The sentence continues here."
        );
        assert_eq!(
            region_translation_text(&[line("An unfinished\n English phrase", 0.1)], 1000, 400),
            "An unfinished English phrase"
        );
        assert_eq!(
            region_translation_text(&[line("没有标点的\n中文短语", 0.1)], 1000, 400),
            "没有标点的中文短语"
        );
    }

    #[test]
    fn preserves_paragraphs_in_fallback_and_explicit_blank_lines() {
        assert_eq!(
            region_translation_text(
                &[
                    line("First paragraph", 0.1),
                    line("continued", 0.18),
                    line("Second paragraph", 0.4)
                ],
                1000,
                400
            ),
            "First paragraph continued\n\nSecond paragraph"
        );
        assert_eq!(
            region_translation_text(
                &[line("First paragraph.\n\nSecond paragraph.", 0.1)],
                1000,
                400
            ),
            "First paragraph.\n\nSecond paragraph."
        );
    }

    #[test]
    fn joins_hyphenated_words_without_corrupting_hard_hyphens() {
        assert_eq!(
            region_translation_text(
                &[line("An inter\u{ad}", 0.1), line("national word.", 0.18)],
                1000,
                400
            ),
            "An international word."
        );
        assert_eq!(
            region_translation_text(
                &[line("A well-", 0.1), line("known rule.", 0.18)],
                1000,
                400
            ),
            "A well-known rule."
        );
    }

    #[test]
    fn joins_punctuation_and_preserves_sentences_before_clipped_line() {
        assert_eq!(
            region_translation_text(&[line("Hello", 0.1), line(", world.", 0.18)], 1000, 400),
            "Hello, world."
        );
        let mut clipped = line("Cut sentence.", 0.18);
        clipped.x = 0.0;
        assert_eq!(
            region_translation_text(&[line("A complete sentence.", 0.1), clipped], 1000, 400),
            "A complete sentence."
        );
    }

    #[test]
    fn still_filters_fragments_when_a_complete_sentence_exists() {
        assert_eq!(
            region_translation_text(
                &[line("A complete sentence. The next sentence is", 0.1)],
                1000,
                400
            ),
            "A complete sentence."
        );
    }

    #[test]
    fn preserves_wrapped_sentence_and_discards_next_fragment() {
        let lines = [
            line(
                "When you open Zed without a folder, you see the welcome page in the main",
                0.1,
            ),
            line(
                "editor area. The welcome page offers quick actions to open a folder, clone a",
                0.18,
            ),
        ];
        assert_eq!(
            complete_region_text(&lines, 1462, 350),
            "When you open Zed without a folder, you see the welcome page in the main editor area."
        );
    }

    #[test]
    fn removes_leading_fragment_and_keeps_multiple_complete_sentences() {
        let lines = [line(
            "of the previous sentence. A complete sentence. Another full one! Trailing",
            0.1,
        )];
        assert_eq!(
            complete_region_text(&lines, 1000, 400),
            "A complete sentence. Another full one!"
        );
    }

    #[test]
    fn keeps_paragraph_breaks_and_does_not_merge_incomplete_paragraphs() {
        let lines = [
            line("An unfinished paragraph", 0.1),
            line("A complete paragraph.", 0.35),
            line("Another complete paragraph.", 0.6),
        ];
        assert_eq!(
            complete_region_text(&lines, 1000, 400),
            "A complete paragraph.\n\nAnother complete paragraph."
        );
    }

    #[test]
    fn handles_abbreviations_decimals_and_closing_quotes() {
        let lines = [line(
            "Dr. Smith uses version 3.14. He said “Hello!” Next complete sentence. An incomplete",
            0.1,
        )];
        assert_eq!(
            complete_region_text(&lines, 1000, 400),
            "Dr. Smith uses version 3.14. He said “Hello!” Next complete sentence."
        );
    }

    #[test]
    fn joins_chinese_lines_and_discards_incomplete_tail() {
        let lines = [
            line("这是跨行的", 0.1),
            line("完整句子。下一句也完整！最后这句没结束", 0.18),
        ];
        assert_eq!(
            complete_region_text(&lines, 1000, 400),
            "这是跨行的完整句子。下一句也完整！"
        );
    }

    #[test]
    fn rejects_sentences_intersecting_clipped_lines() {
        let mut clipped = line("A clipped line.", 0.0);
        clipped.x = 0.0;
        assert_eq!(
            complete_region_text(&[clipped, line("A complete paragraph.", 0.3)], 1000, 400),
            "A complete paragraph."
        );
        let mut right = line("A sentence that wraps", 0.1);
        right.w = 0.95;
        assert_eq!(
            complete_region_text(&[right, line("to the next line.", 0.18)], 1000, 400),
            ""
        );
    }

    #[test]
    fn no_complete_sentence_returns_empty_instead_of_partial_text() {
        assert_eq!(
            complete_region_text(&[line("Only an unfinished sentence", 0.1)], 1000, 400),
            ""
        );
        assert_eq!(complete_region_text(&[], 1000, 400), "");
    }

    #[test]
    fn keeps_acronyms_and_does_not_translate_a_bare_list_number() {
        assert_eq!(
            complete_region_text(
                &[line("She works in the U.S.A. today. Unfinished", 0.1)],
                1000,
                400
            ),
            "She works in the U.S.A. today."
        );
        assert_eq!(
            complete_region_text(&[line("1. An unfinished list item", 0.1)], 1000, 400),
            ""
        );
    }
}
