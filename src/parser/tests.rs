use crate::parser::{parse_typefile_content, Typefile};
use crate::workflow::Tool;

#[test]
fn test_empty_typefile() {
    assert_eq!(parse_typefile_content(""), Ok(Typefile::default()));
}

#[test]
fn test_few_code_lines() {
    assert_eq!(
        parse_typefile_content("abc\ndef"),
        Ok(Typefile {
            code_lines: vec!["abc", "def"].into_iter().map(String::from).collect(),
            ..Default::default()
        })
    );
}

#[test]
fn test_tool_definition() {
    assert_eq!(
        parse_typefile_content("abc\n\ntool mytool:\ndef\nefg"),
        Ok(Typefile {
            code_lines: vec!["abc", "def", "efg"]
                .into_iter()
                .map(String::from)
                .collect(),
            tools: [(
                "mytool".to_owned(),
                Tool {
                    name: "mytool".to_string()
                }
            )]
            .iter()
            .cloned()
            .collect(),
            ..Default::default()
        })
    );
}
