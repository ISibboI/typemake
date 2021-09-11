/// A tool definition.
/// A tool is the basic building block of a workflow.
/// It describes how files with certain properties are transformed into files with other properties.
#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct Tool {
    /// Each tool has a unique name.
    pub name: String,

    /// The script executing the tool.
    /// Typically this would be a bash script executing another program or a set of programs.
    pub script: Option<String>,
}
