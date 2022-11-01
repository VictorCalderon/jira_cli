use ellipse::Ellipse;

pub fn get_column_string(text: &str, width: usize) -> String {
    // If string is empty, return a padded string of the given width
    if text.is_empty() {
        return " ".repeat(width);
    }
    // If length and with are the same, return the string
    if text.len() == width {
        return text.to_string();
    }
    // If width is between 1 and 3, return the same number of dots.
    if width < 4 {
        return ".".repeat(width);
    }
    // If width is larger than the string, return the string padded with spaces
    if text.len() < width {
        return format!("{:width$}", text, width = width);
    }
    // Get string length
    let truncated_string = text.truncate_ellipse(width - 3).to_string();
    // Return string
    return truncated_string;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_column_width_0_is_empty_string() {
        assert_eq!(get_column_string("thisisatest", 0), "");
    }

    #[test]
    fn get_column_width_1_is_single_dot_string() {
        assert_eq!(get_column_string("thisisatest", 1), ".");
    }

    #[test]
    fn get_column_width_2_is_double_dot_string() {
        assert_eq!(get_column_string("thisisatest", 2), "..");
    }

    #[test]
    fn get_column_width_3_is_triple_dot_string() {
        assert_eq!(get_column_string("thisisatest", 3), "...");
    }

    #[test]
    fn get_column_width_4_is_first_char_with_triple_dot_string() {
        assert_eq!(get_column_string("thisisatest", 4), "t...");
    }

    #[test]
    fn get_column_width_6_pads_empty_string_to_six_white_spaces() {
        assert_eq!(get_column_string("", 6), "      ");
    }

    #[test]
    fn get_column_width_6_pads_smaller_string_with_white_spaces() {
        assert_eq!(get_column_string("this", 6), "this  ");
    }

    #[test]
    fn get_column_string_returns_same_string_if_length_and_width_are_equal() {
        assert_eq!(get_column_string("thisisatest", 11), "thisisatest");
    }

    #[test]
    fn get_column_string_6_truncates_longer_string_with_ellipse() {
        assert_eq!(get_column_string("thisisatest", 6), "thi...");
    }
}
