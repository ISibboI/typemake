//! Types describing a typemake workflow.

use traitgraph::interface::GraphBase;
use std::collections::BTreeMap;

/// The stage of a value of a tool property.
///
/// The value of a tool property is computed using the script interpreter, but its evaluation is allowed to fail.
/// Such a failed state needs to be captured to indicate that the tool is not usable yet in the given configuration.
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum ToolPropertyStage<PreliminaryType, FinalType = PreliminaryType> {
    /// The property is not defined on the tool.
    Empty,
    /// The property is defined on the tool, but no attempt at evaluation was made up to now.
    String,
    /// The property was evaluated, but the evaluation failed completely or partially.
    Preliminary(PreliminaryType),
    /// The property was successfully evaluated.
    Final(FinalType),
}

impl<S, T> Default for ToolPropertyStage<S, T> {
    fn default() -> Self {
        Self::Empty
    }
}

/// A property value of a tool.
#[derive(Default, Eq, PartialEq, Debug, Clone)]
pub struct ToolProperty<PreliminaryType, FinalType = PreliminaryType> {
    /// The original string that is used to define the value in the typefile.
    string_value: String,
    /// The value of the tool property as computed by the script interpreter.
    /// This field captures all possible stages of evaluation.
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
    /// Returns true if the tool property value is empty, i.e. not set.
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

    /// The interpreter executing the tool.
    /// Typically this would be a bash interpreter executing another program or a set of programs.
    pub script: ToolProperty<String>,

    //pub input: ToolProperty<ToolInputDefinition>,

    //pub output: ToolProperty<ToolOutputDefinition>,
}

// TODO create separate workflow definition and workflow instantiation types.

/*pub struct ToolInstance {

}

pub struct WorkflowGraph<Graph: GraphBase<NodeData = ToolInstance>> {
    graph: Graph,
    output_node_map: BTreeMap<ToolOutput, Graph::NodeIndex>,
}*/