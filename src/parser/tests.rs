use crate::parser::{parse_typefile_content, Typefile};

#[test]
fn test_empty_typefile() {
    assert_eq!(parse_typefile_content(""), Ok(Typefile::default()));
}

#[test]
fn test_few_python_lines() {
    assert_eq!(
        parse_typefile_content("abc\ndef"),
        Ok(Typefile {
            python_lines: vec!["abc", "def"].into_iter().map(String::from).collect(),
            ..Default::default()
        })
    );
}

#[test]
fn test_tool_definition() {
    assert_eq!(
        parse_typefile_content("abc\n\ntool mytool:\ndef\nefg"),
        Ok(Typefile {
            python_lines: vec!["abc", "def", "efg"]
                .into_iter()
                .map(String::from)
                .collect(),
            ..Default::default()
        })
    );
}
