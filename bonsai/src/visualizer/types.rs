use imnodes::{AttributeId, EditorContext, IdentifierGenerator, InputPinId, LinkId, NodeId, OutputPinId};

#[derive(Debug, Clone)]
pub(crate) struct Graph<A> {
    pub nodes: Vec<Node<A>>,
    pub links: Vec<Link>,
}
pub(crate) struct State<A> {
    pub editor_context: EditorContext,
    pub id_gen: IdentifierGenerator,
    pub graph: Graph<A>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Link {
    pub id: LinkId,
    pub start: OutputPinId,
    pub end: InputPinId,
}

#[derive(Debug, Clone)]
pub(crate) struct Node<A> {
    pub id: NodeId,
    pub typ: UiNodeType<A>,
}

#[derive(Debug, Clone)]
pub(crate) enum UiNodeType<A> {
    Root(RootNode),
    Wait(WaitNode),
    WaitForever(WaitForeverNode),
    Action(ActionNode<A>),
    Invert(InternalNode),
    AlwaysSucceed(InternalNode),
    Select(InternalNode),
    If(InternalNode),
    Sequence(InternalNode),
    While(InternalNode),
    WhenAll(InternalNode),
    WhenAny(InternalNode),
    After(InternalNode),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RootNode {
    pub output: OutputPinId,
    pub attribute: AttributeId,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WaitNode {
    pub input: InputPinId,
    pub wait: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WaitForeverNode {
    pub input: InputPinId,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ActionNode<A> {
    pub input: InputPinId,
    pub action: A,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InternalNode {
    pub input: InputPinId,
    pub output: OutputPinId,
}
