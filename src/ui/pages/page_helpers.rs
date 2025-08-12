use ellipse::Ellipse;
use std::cmp::Ordering;

pub fn get_column_string(text: &str, width: usize) -> String {
    match text.len().cmp(&width) {
        Ordering::Equal => String::from(text),
        Ordering::Less => format!(
            "{}{}",
            text,
            vec![' '; width - text.len()]
                .into_iter()
                .collect::<String>()
        ),
        Ordering::Greater => {
            if width <= 3 {
                vec!['.'; width].into_iter().collect::<String>()
            } else {
                text.truncate_ellipse(width - 3).to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_column_string() {
        let text1 = "";
        let text2 = "test";
        let text3 = "testme";
        let text4 = "testmetest";

        let width = 0;

        assert_eq!(get_column_string(text4, width), "".to_owned());

        let width = 1;

        assert_eq!(get_column_string(text4, width), ".".to_owned());

        let width = 2;

        assert_eq!(get_column_string(text4, width), "..".to_owned());

        let width = 3;

        assert_eq!(get_column_string(text4, width), "...".to_owned());

        let width = 4;

        assert_eq!(get_column_string(text4, width), "t...".to_owned());

        let width = 6;

        assert_eq!(get_column_string(text1, width), "      ".to_owned());
        assert_eq!(get_column_string(text2, width), "test  ".to_owned());
        assert_eq!(get_column_string(text3, width), "testme".to_owned());
        assert_eq!(get_column_string(text4, width), "tes...".to_owned());
    }
}
