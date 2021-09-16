use crate::parser::{parse_typefile_content, Typefile};
use crate::workflow::Tool;

#[test]
fn test_empty_typefile() {
    assert_eq!(parse_typefile_content("").unwrap(), Typefile::default());
}

#[test]
fn test_few_code_lines() {
    assert_eq!(
        parse_typefile_content("abc\ndef").unwrap(),
        Typefile {
            code_lines: "abc\ndef\n".into(),
            ..Default::default()
        }
    );
}

#[test]
fn test_tool_name_definition() {
    assert_eq!(
        parse_typefile_content("abc\n\ntool mytool:\ndef\nefg").unwrap(),
        Typefile {
            code_lines: "abc\ndef\nefg\n".into(),
            tools: [(
                "mytool".to_owned(),
                Tool {
                    name: "mytool".to_string(),
                    ..Default::default()
                }
            )]
            .iter()
            .cloned()
            .collect(),
            ..Default::default()
        }
    );
}

#[test]
fn test_duplicate_tool_name_definition() {
    parse_typefile_content("abc\n\ntool mytool:\ntool mytool:\ndef\nefg").unwrap_err();
}

#[test]
fn test_tool_script_definition() {
    assert_eq!(
        parse_typefile_content("abc\n\ntool mytool:\n  interpreter: \"ls -l\"\ndef\nefg").unwrap(),
        Typefile {
            code_lines: "abc\ndef\nefg\n".into(),
            tools: [(
                "mytool".to_owned(),
                Tool {
                    name: "mytool".to_string(),
                    script: "\"ls -l\"".into(),
                    ..Default::default()
                }
            )]
            .iter()
            .cloned()
            .collect(),
            ..Default::default()
        }
    );
}

#[test]
fn test_tool_multiline_script_definition() {
    assert_eq!(
        parse_typefile_content(
            "abc\n\ntool mytool:\n  interpreter: \"\"\"ls -l\n    pwd\"\"\"\ndef\nefg"
        )
        .unwrap(),
        Typefile {
            code_lines: "abc\ndef\nefg\n".into(),
            tools: [(
                "mytool".to_owned(),
                Tool {
                    name: "mytool".to_string(),
                    script: "\"\"\"ls -l\npwd\"\"\"".into(),
                    ..Default::default()
                }
            )]
            .iter()
            .cloned()
            .collect(),
            ..Default::default()
        }
    );
}

#[test]
fn test_tool_multiline_script_definition_start_second_line() {
    assert_eq!(
        parse_typefile_content(
            "abc\n\ntool mytool:\n  interpreter: \n    \"\"\"ls -l\n    pwd\"\"\"\ndef\nefg"
        )
        .unwrap(),
        Typefile {
            code_lines: "abc\ndef\nefg\n".into(),
            tools: [(
                "mytool".to_owned(),
                Tool {
                    name: "mytool".to_string(),
                    script: "\"\"\"ls -l\npwd\"\"\"".into(),
                    ..Default::default()
                }
            )]
            .iter()
            .cloned()
            .collect(),
            ..Default::default()
        }
    );
}

#[test]
fn test_code_line_indentation() {
    assert_eq!(
        parse_typefile_content("abc\n def \n  efg\nfgh\n\tghi").unwrap(),
        Typefile {
            code_lines: "abc\n def \n  efg\nfgh\n\tghi\n".into(),
            ..Default::default()
        }
    );
}

#[test]
fn test_wrong_tool_property_indentation() {
    parse_typefile_content(
        "abc\n\ntool mytool:\n  interpreter:\n    interpreter\n   missing-indentation\n    blub\ndef\nefg",
    )
    .unwrap_err();
}
