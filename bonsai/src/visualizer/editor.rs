#![allow(dead_code, unused_imports)]
use imgui::{PopupModal, Slider, Ui};
use imnodes::{
    editor, AttributeFlag, AttributeId, Context, EditorContext, IdentifierGenerator, InputPinId, LinkId, NodeId,
    OutputPinId, PinShape,
};
use std::fmt::Debug;

use super::types::{
    ActionNode, Graph, InternalNode, Link, Node, RootNode, State, UiNodeType, WaitForeverNode, WaitNode,
};

impl<A: Clone + Debug> Graph<A> {
    /// generate a new graph
    fn new(id_gen: &mut IdentifierGenerator) -> Self {
        let root_node_id = id_gen.next_node();
        let wait_input_id = id_gen.next_input_pin();
        let root_output_id = id_gen.next_output_pin();

        Self {
            nodes: vec![
                Node {
                    id: id_gen.next_node(),
                    typ: UiNodeType::<A>::Wait(WaitNode {
                        input: wait_input_id,
                        wait: 30.0,
                    }),
                },
                Node {
                    id: root_node_id,
                    typ: UiNodeType::<A>::Root(RootNode {
                        output: root_output_id,
                        attribute: id_gen.next_attribute(),
                    }),
                },
            ],
            links: vec![Link {
                id: id_gen.next_link(),
                start: root_output_id,
                end: wait_input_id,
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
pub(crate) fn show<A>(ui: &Ui, state: &mut State<A>)
where
    A: Clone + Debug,
{
    state.editor_context.set_style_colors_classic();

    ui.text("press \"A\" or right click to add a Node");

    let title_bar_color =
        imnodes::ColorStyle::TitleBar.push_color([11.0 / 255.0, 109.0 / 255.0, 191.0 / 255.0], &state.editor_context);
    let title_bar_hovewait_input_id_color = imnodes::ColorStyle::TitleBarHovered
        .push_color([45.0 / 255.0, 126.0 / 255.0, 194.0 / 255.0], &state.editor_context);
    let title_bar_selected_color = imnodes::ColorStyle::TitleBarSelected
        .push_color([81.0 / 255.0, 148.0 / 255.0, 204.0 / 255.0], &state.editor_context);

    let link_color = imnodes::ColorStyle::Link.push_color([0.8, 0.5, 0.1], &state.editor_context);

    // used to position a node
    state.graph.nodes[0]
        .id
        .set_position(
            0.9 * ui.window_content_region_width(),
            300.0,
            imnodes::CoordinateSystem::ScreenSpace,
        )
        .set_draggable(false);

    // node and link behaviour setup
    let on_snap = state.editor_context.push(AttributeFlag::EnableLinkCreationOnSnap);
    let detach = state.editor_context.push(AttributeFlag::EnableLinkDetachWithDragClick);

    let State {
        ref mut editor_context,
        ref mut graph,
        ref mut id_gen,
        ..
    } = state;

    // main node ui
    let outer_scope = create_the_editor(ui, editor_context, graph, id_gen);

    // user interaction handling
    if let Some(link) = outer_scope.links_created() {
        state.graph.links.push(Link {
            id: state.id_gen.next_link(),
            start: link.start_pin,
            end: link.end_pin,
        })
    }

    if let Some(link) = outer_scope.get_dropped_link() {
        state
            .graph
            .links
            .swap_remove(state.graph.links.iter().position(|e| e.id == link).unwrap());
    }

    // cleanup in the end
    title_bar_color.pop();
    title_bar_hovewait_input_id_color.pop();
    title_bar_selected_color.pop();
    link_color.pop();

    on_snap.pop();
    detach.pop();
}

/// main node ui
fn create_the_editor<A>(
    ui: &Ui,
    editor_context: &mut EditorContext,
    graph: &mut Graph<A>,
    _id_gen: &mut IdentifierGenerator,
) -> imnodes::OuterScope
where
    A: Clone + Debug,
{
    editor(editor_context, |mut editor| {
        editor.add_mini_map(imnodes::MiniMapLocation::BottomLeft);

        let popup_modal = "popup_add_node";

        if editor.is_hovered() && (ui.is_mouse_clicked(imgui::MouseButton::Right) || ui.is_key_released(imgui::Key::A))
        {
            ui.open_popup(popup_modal);
        }

        for curr_node in graph.nodes.iter_mut() {
            match &curr_node.typ {
                UiNodeType::Root(RootNode { output, attribute }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("Root");
                        });

                        node.attribute(*attribute, || {
                            ui.set_next_item_width(130.0);
                        });

                        node.add_output(*output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                UiNodeType::Wait(WaitNode { input, wait }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text(format!("Wait ({wait:?})"));
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });
                    });
                }
                UiNodeType::WaitForever(WaitForeverNode { input }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text(format!("WaitForever"));
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });
                    });
                }
                UiNodeType::Action(ActionNode { input, action }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text(format!("Action ({action:?})"));
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });
                    });
                }
                UiNodeType::Invert(InternalNode { input, output }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("Invert");
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        node.add_output(*output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                UiNodeType::AlwaysSucceed(InternalNode { input, output }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("AlwaysSucceed");
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        node.add_output(*output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                UiNodeType::Select(InternalNode { input, output }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("Select");
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        node.add_output(*output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                UiNodeType::If(InternalNode { input, output }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("If");
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        node.add_output(*output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                UiNodeType::Sequence(InternalNode { input, output }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("Sequence");
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        node.add_output(*output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                UiNodeType::While(InternalNode { input, output }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("While");
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        node.add_output(*output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                UiNodeType::WhenAll(InternalNode { input, output }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("WhenAll");
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        node.add_output(*output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                UiNodeType::WhenAny(InternalNode { input, output }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("WhenAny");
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        node.add_output(*output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                UiNodeType::After(InternalNode { input, output }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("After");
                        });

                        node.add_input(*input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        node.add_output(*output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
            }
        }

        for Link { id, start, end } in &graph.links {
            editor.add_link(*id, *end, *start);
        }
    })
}
