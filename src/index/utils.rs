use unicode_segmentation::UnicodeSegmentation; // 1.9.0

pub fn find_length_string(text: &str) -> usize {
    return text.graphemes(true).count();
}
