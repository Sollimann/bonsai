use imgui::{PopupModal, Slider, Ui};

use imnodes::{
    editor, AttributeFlag, AttributeId, Context, EditorContext, IdentifierGenerator, InputPinId, LinkId, NodeId,
    OutputPinId, PinShape,
};

pub struct State {
    editor_context: EditorContext,
    id_gen: IdentifierGenerator,

    graph: Graph,
}

#[derive(Debug, Clone)]
struct Graph {
    nodes: Vec<Node>,
    links: Vec<Link>,
}

impl Graph {
    fn new(id_gen: &mut IdentifierGenerator) -> Self {
        let output = id_gen.next_node();
        let red = id_gen.next_input_pin();
        let constant = id_gen.next_output_pin();
        // TODO does not work here?
        // output.set_position(1000.0, 300.0, imnodes::CoordinateSystem::ScreenSpace);

        Self {
            nodes: vec![
                Node {
                    id: output,
                    value: 0.0, // never used
                    typ: NodeType::Output(OutData {
                        input_red: red,
                        input_green: id_gen.next_input_pin(),
                        input_blue: id_gen.next_input_pin(),
                        red: 0.1,
                        green: 0.1,
                        blue: 0.1,
                    }),
                    updated: false,
                },
                Node {
                    id: id_gen.next_node(),
                    typ: NodeType::Constant(ConstData {
                        output: constant,
                        attribute: id_gen.next_attribute(),
                    }),
                    value: 0.4,
                    updated: false,
                },
            ],
            links: vec![Link {
                id: id_gen.next_link(),
                start: constant,
                end: red,
            }],
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Link {
    id: LinkId,
    start: OutputPinId,
    end: InputPinId,
}

#[derive(Debug, Clone)]
struct Node {
    id: NodeId,
    typ: NodeType,
    value: f32,
    // for cycle detection
    updated: bool,
}

impl Node {
    fn has_output(&self, out: OutputPinId) -> bool {
        match self.typ {
            NodeType::Add(AddData { output, .. })
            | NodeType::Multiply(MultData { output, .. })
            | NodeType::Sine(SineData { output, .. })
            | NodeType::Time(TimeData { output, .. })
            | NodeType::Constant(ConstData { output, .. }) => output == out,
            NodeType::Output(_) => false,
        }
    }
    fn get_inputs(&self) -> Vec<InputPinId> {
        match self.typ {
            NodeType::Add(AddData { input, .. })
            | NodeType::Multiply(MultData { input, .. })
            | NodeType::Sine(SineData { input, .. }) => vec![input],
            NodeType::Output(OutData {
                input_red,
                input_green,
                input_blue,
                ..
            }) => vec![input_red, input_green, input_blue],
            NodeType::Time(_) | NodeType::Constant(_) => vec![],
        }
    }
}

fn update(graph: &mut Graph, curr_node_idx: usize, input_pin: Option<InputPinId>) {
    let links = &graph.links;

    let curr_node = graph.nodes[curr_node_idx].clone();

    let predecessors = graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(i, input_node)| {
            let is_connected = |link: &Link| {
                if let Some(input_pin) = input_pin {
                    input_node.has_output(link.start) && input_pin == link.end
                } else {
                    input_node.has_output(link.start) && curr_node.get_inputs().contains(&link.end)
                }
            };

            match links.iter().any(|link| is_connected(link)) {
                true => Some(i),
                false => None,
            }
        })
        .collect::<Vec<_>>();

    for p in &predecessors {
        if !graph.nodes[*p].updated {
            graph.nodes[*p].updated = true;
            update(graph, *p, None);
        }
    }

    // TODO do is this the best way to do this?
    let nodes = &mut graph.nodes;
    // SAFETY because we have no cycles, `curr_node` is never accessed through `nodes`
    let curr_node = unsafe { &mut *((&mut nodes[curr_node_idx]) as *mut Node) };

    match curr_node.typ {
        NodeType::Add(_) => curr_node.value = predecessors.iter().fold(0.0, |acc, x| acc + nodes[*x].value),
        NodeType::Multiply(_) => curr_node.value = predecessors.iter().fold(1.0, |acc, x| acc * nodes[*x].value),
        NodeType::Output(OutData {
            ref mut red,
            ref mut green,
            ref mut blue,
            ref input_red,
            ref input_green,
            ref input_blue,
            ..
        }) => {
            let total_val = predecessors.iter().fold(0.0, |acc, x| acc + nodes[*x].value);
            let input_pin = input_pin.unwrap();
            if input_pin == *input_red {
                *red = total_val;
            } else if input_pin == *input_green {
                *green = total_val;
            } else if input_pin == *input_blue {
                *blue = total_val;
            }
        }
        NodeType::Sine(_) => {
            curr_node.value = if let Some(input) = predecessors.first() {
                (nodes[*input].value * std::f32::consts::PI).sin()
            } else {
                0.0
            }
        }
        NodeType::Time(_) => {
            curr_node.value = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % 1000) as f32
                / 1000.0;
        }
        NodeType::Constant(_) => {
            // &predecessors.iter().collect::<Vec<_>>();
            // nothing to do
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
enum NodeType {
    Add(AddData),
    Multiply(MultData),
    Output(OutData),
    Sine(SineData),
    Time(TimeData),
    Constant(ConstData),
}

#[derive(Debug, Clone, PartialEq)]
struct AddData {
    input: InputPinId,
    output: OutputPinId,
}

#[derive(Debug, Clone, PartialEq)]
struct MultData {
    input: InputPinId,
    output: OutputPinId,
}
#[derive(Debug, Clone, PartialEq)]
struct OutData {
    input_red: InputPinId,
    input_green: InputPinId,
    input_blue: InputPinId,
    red: f32,
    green: f32,
    blue: f32,
}
#[derive(Debug, Clone, PartialEq)]
struct SineData {
    input: InputPinId,
    output: OutputPinId,
}
#[derive(Debug, Clone, PartialEq)]
struct TimeData {
    input: InputPinId,
    output: OutputPinId,
}
#[derive(Debug, Clone, PartialEq)]
struct ConstData {
    output: OutputPinId,
    attribute: AttributeId,
}

impl State {
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
pub fn show(ui: &Ui, state: &mut State) {
    state.editor_context.set_style_colors_classic();

    ui.text("press \"A\" or right click to add a Node");

    // color setup
    let background = if let NodeType::Output(OutData { red, green, blue, .. }) = &state.graph.nodes[0].typ {
        imnodes::ColorStyle::GridBackground.push_color([*red, *green, *blue], &state.editor_context)
    } else {
        unreachable!()
    };

    let title_bar_color =
        imnodes::ColorStyle::TitleBar.push_color([11.0 / 255.0, 109.0 / 255.0, 191.0 / 255.0], &state.editor_context);
    let title_bar_hovered_color = imnodes::ColorStyle::TitleBarHovered
        .push_color([45.0 / 255.0, 126.0 / 255.0, 194.0 / 255.0], &state.editor_context);
    let title_bar_selected_color = imnodes::ColorStyle::TitleBarSelected
        .push_color([81.0 / 255.0, 148.0 / 255.0, 204.0 / 255.0], &state.editor_context);

    let link_color = imnodes::ColorStyle::Link.push_color([0.8, 0.5, 0.1], &state.editor_context);

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

    // propagate the values through the graph
    {
        let (input_red, input_green, input_blue) = if let NodeType::Output(OutData {
            input_red,
            input_green,
            input_blue,
            ..
        }) = graph.nodes[0].typ
        {
            (input_red, input_green, input_blue)
        } else {
            unreachable!()
        };

        update(graph, 0, Some(input_red));
        update(graph, 0, Some(input_green));
        update(graph, 0, Some(input_blue));
        for node in &mut graph.nodes {
            node.updated = false;
        }
    }

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

    // cleanup
    background.pop();

    title_bar_color.pop();
    title_bar_hovered_color.pop();
    title_bar_selected_color.pop();
    link_color.pop();

    on_snap.pop();
    detach.pop();
}

/// main node ui
fn create_the_editor(
    ui: &Ui,
    editor_context: &mut EditorContext,
    graph: &mut Graph,
    id_gen: &mut IdentifierGenerator,
) -> imnodes::OuterScope {
    editor(editor_context, |mut editor| {
        editor.add_mini_map(imnodes::MiniMapLocation::BottomLeft);

        let popup_modal = "popup_add_node";

        if editor.is_hovered() && (ui.is_mouse_clicked(imgui::MouseButton::Right) || ui.is_key_released(imgui::Key::A))
        {
            ui.open_popup(popup_modal);
        }

        PopupModal::new("popup_add_node")
            .resizable(false)
            .title_bar(false)
            .build(ui, || {
                let size = [100.0, 0.0];

                let mut gen_node = || {
                    let node = id_gen.next_node();
                    let [x, y] = ui.mouse_pos_on_opening_current_popup();
                    node.set_position(x, y, imnodes::CoordinateSystem::ScreenSpace);
                    node
                };

                if ui.button_with_size("Add", size) {
                    graph.nodes.push(Node {
                        id: gen_node(),
                        value: 0.0,
                        typ: NodeType::Add(AddData {
                            input: id_gen.next_input_pin(),
                            output: id_gen.next_output_pin(),
                        }),
                        updated: false,
                    });

                    ui.close_current_popup();
                } else if ui.button_with_size("Multiply", size) {
                    graph.nodes.push(Node {
                        id: gen_node(),
                        value: 0.0,
                        typ: NodeType::Multiply(MultData {
                            input: id_gen.next_input_pin(),
                            output: id_gen.next_output_pin(),
                        }),
                        updated: false,
                    });
                    ui.close_current_popup();
                } else if ui.button_with_size("Sine", size) {
                    graph.nodes.push(Node {
                        id: gen_node(),
                        value: 0.0,
                        typ: NodeType::Sine(SineData {
                            input: id_gen.next_input_pin(),
                            output: id_gen.next_output_pin(),
                        }),
                        updated: false,
                    });
                    ui.close_current_popup();
                } else if ui.button_with_size("Time", size) {
                    graph.nodes.push(Node {
                        id: gen_node(),
                        value: 0.0,
                        typ: NodeType::Time(TimeData {
                            input: id_gen.next_input_pin(),
                            output: id_gen.next_output_pin(),
                        }),
                        updated: false,
                    });
                    ui.close_current_popup();
                } else if ui.button_with_size("Constant", size) {
                    graph.nodes.push(Node {
                        id: gen_node(),
                        value: 0.0,
                        typ: NodeType::Constant(ConstData {
                            output: id_gen.next_output_pin(),
                            attribute: id_gen.next_attribute(),
                        }),
                        updated: false,
                    });
                    ui.close_current_popup();
                }

                ui.separator();

                if ui.button_with_size("Close", size) {
                    ui.close_current_popup();
                }

                ui.separator();
            });

        for curr_node in graph.nodes.iter_mut() {
            match curr_node.typ {
                NodeType::Add(AddData { input, output, .. }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("Add");
                        });

                        node.add_input(input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        ui.text(format!("Value: {:.2}", curr_node.value));

                        node.add_output(output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                NodeType::Multiply(MultData { input, output, .. }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("Multiply");
                        });

                        ui.text(format!("Value: {:.2}", curr_node.value));

                        node.add_input(input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        node.add_output(output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                NodeType::Output(OutData {
                    input_red,
                    input_green,
                    input_blue,
                    red,
                    green,
                    blue,
                    ..
                }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("Output");
                        });

                        node.add_input(input_red, PinShape::QuadFilled, || {
                            ui.text("red");
                        });

                        node.add_input(input_green, PinShape::QuadFilled, || {
                            ui.text("green");
                        });

                        node.add_input(input_blue, PinShape::QuadFilled, || {
                            ui.text("blue");
                        });

                        ui.text(format!("red: {:.2}", red));
                        ui.text(format!("gree: {:.2}", green));
                        ui.text(format!("blue: {:.2}", blue));
                    });
                }
                NodeType::Sine(SineData { input, output, .. }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("Sine");
                        });

                        node.add_input(input, PinShape::QuadFilled, || {
                            ui.text("input");
                        });

                        // TODO add modal for things other than sine?
                        ui.text(format!("Value: {:.2}", curr_node.value));

                        node.add_output(output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                NodeType::Time(TimeData { output, .. }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("Time");
                        });

                        ui.text(format!("Value: {:.2}", curr_node.value));

                        node.add_output(output, PinShape::CircleFilled, || {
                            ui.text("output");
                        });
                    });
                }
                NodeType::Constant(ConstData { attribute, output, .. }) => {
                    editor.add_node(curr_node.id, |mut node| {
                        node.add_titlebar(|| {
                            ui.text("Constant");
                        });

                        node.attribute(attribute, || {
                            ui.set_next_item_width(130.0);
                            Slider::new("value", 0.0, 1.0)
                                .display_format(format!("{:.2}", curr_node.value))
                                .build(ui, &mut curr_node.value);
                        });

                        node.add_output(output, PinShape::CircleFilled, || {
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
