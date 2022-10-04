#![allow(dead_code, unused_imports)]
use imgui::{PopupModal, Slider, Ui};
use imnodes::{
    editor, AttributeFlag, AttributeId, Context, EditorContext, IdentifierGenerator, InputPinId, LinkId, NodeId,
    OutputPinId, PinShape,
};
use std::fmt::Debug;

use crate::graph::NodeType;
#[derive(Debug, Clone)]
struct Graph<A> {
    nodes: Vec<Node<A>>,
    links: Vec<Link>,
}
pub struct State<A> {
    editor_context: EditorContext,
    id_gen: IdentifierGenerator,
    graph: Graph<A>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Link {
    id: LinkId,
    start: OutputPinId,
    end: InputPinId,
}

#[derive(Debug, Clone)]
struct Node<A> {
    id: NodeId,
    typ: NodeType<A>,
    input: InputPinId,
    output: OutputPinId,
    attribute: AttributeId,
}

impl<A: Clone + Debug> Graph<A> {
    /// generate a new graph
    fn new(id_gen: &mut IdentifierGenerator) -> Self {
        let output = id_gen.next_node();
        let red = id_gen.next_input_pin();
        let constant = id_gen.next_output_pin();

        Self {
            nodes: vec![Node {
                id: output,
                typ: NodeType::<A>::Root,
                input: id_gen.next_input_pin(),
                output: id_gen.next_output_pin(),
                attribute: id_gen.next_attribute(),
            }],
            links: vec![Link {
                id: id_gen.next_link(),
                start: constant,
                end: red,
            }],
        }
    }
}

/// State of the visualizer
impl<A: Clone + Debug> State<A> {
    pub fn new(context: &Context) -> Self {
        let editor_context = context.create_editor();
        let mut id_gen = editor_context.new_identifier_generator();
        let nodes = Graph::new(&mut id_gen);

        Self {
            id_gen,
            editor_context,
            graph: nodes,
        }
    }
}

/// https://github.com/Nelarius/imnodes/blob/master/example/color_node_editor.cpp
///
/// TODO
/// - add more mouse keyboard modifiers/ more vibrant colors
pub fn show<A>(ui: &Ui, state: &mut State<A>) {
    state.editor_context.set_style_colors_classic();

    ui.text("press \"A\" or right click to add a Node");
}
