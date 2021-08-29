use crate::parser::{nom_typefile, Typefile};

#[test]
fn test_empty_typefile() {
    assert_eq!(nom_typefile(""), Ok(("", Typefile::default())));
}