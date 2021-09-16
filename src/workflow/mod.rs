#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ToolPropertyStage<PreliminaryType, FinalType = PreliminaryType> {
    Empty,
    String,
    Preliminary(PreliminaryType),
    Final(FinalType),
}

impl<S, T> Default for ToolPropertyStage<S, T> {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct ToolProperty<PreliminaryType, FinalType = PreliminaryType> {
    string_value: String,
    value_stage: ToolPropertyStage<PreliminaryType, FinalType>,
}

impl<PreliminaryType, FinalType, T: Into<String>> From<T>
    for ToolProperty<PreliminaryType, FinalType>
{
    fn from(string_value: T) -> Self {
        Self {
            string_value: string_value.into(),
            value_stage: ToolPropertyStage::String,
        }
    }
}

impl<PreliminaryType, FinalType> ToolProperty<PreliminaryType, FinalType> {
    pub fn is_empty(&self) -> bool
    where
        PreliminaryType: PartialEq,
        FinalType: PartialEq,
    {
        self.value_stage == ToolPropertyStage::Empty
    }
}

/// A tool definition.
/// A tool is the basic building block of a workflow.
/// It describes how files with certain properties are transformed into files with other properties.
#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct Tool {
    /// Each tool has a unique name.
    pub name: String,

    /// The script executing the tool.
    /// Typically this would be a bash script executing another program or a set of programs.
    pub script: ToolProperty<String>,
}
