use compose_rt::{ComposeNode, Composer, Recomposer, ScopeId};
use taffy::{
    compute_block_layout, compute_cached_layout, compute_flexbox_layout, compute_grid_layout,
    compute_hidden_layout, compute_leaf_layout, compute_root_layout, print_tree, round_layout,
    AvailableSpace, Cache, CacheTree, Display, FlexDirection, Layout, LayoutBlockContainer,
    LayoutFlexboxContainer, LayoutGridContainer, LayoutPartialTree, NodeId, PrintTree, RoundTree,
    RunMode, Size, Style, TraversePartialTree, TraverseTree,
};

pub trait NodeIdLike {
    fn into_node_id(self) -> NodeId;
}

impl<T> NodeIdLike for T
where
    T: Into<u64>,
{
    #[inline(always)]
    fn into_node_id(self) -> NodeId {
        NodeId::new(self.into())
    }
}

pub trait ScopeIdLike {
    fn into_scope_id(self) -> ScopeId;
}

impl<T> ScopeIdLike for T
where
    T: Into<u64>,
{
    #[inline(always)]
    fn into_scope_id(self) -> ScopeId {
        ScopeId::with(self.into())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TaffyConfig {
    /// Whether to round layout values
    pub use_rounding: bool,
}

impl Default for TaffyConfig {
    fn default() -> Self {
        Self { use_rounding: true }
    }
}

impl TaffyConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rounding(mut self, use_rounding: bool) -> Self {
        self.use_rounding = use_rounding;
        self
    }

    pub fn enable_rounding(&mut self) {
        self.use_rounding = true;
    }

    pub fn disable_rounding(&mut self) {
        self.use_rounding = false;
    }
}

#[derive(Debug, Clone)]
pub struct LayoutNode<T>
where
    T: 'static,
{
    pub style: Style,
    pub unrounded_layout: Layout,
    pub final_layout: Layout,
    pub cache: Cache,
    pub context: Option<T>,
}

impl<T> ComposeNode for LayoutNode<T>
where
    T: 'static,
{
    type Context = TaffyConfig;
}

impl<T> LayoutNode<T>
where
    T: 'static,
{
    pub fn new(style: Style) -> Self {
        Self {
            style,
            unrounded_layout: Layout::new(),
            final_layout: Layout::new(),
            cache: Cache::new(),
            context: None,
        }
    }

    pub fn with_context(style: Style, context: T) -> Self {
        Self {
            style,
            unrounded_layout: Layout::new(),
            final_layout: Layout::new(),
            cache: Cache::new(),
            context: Some(context),
        }
    }

    #[inline]
    pub fn mark_dirty(&mut self) {
        self.cache.clear()
    }
}

pub struct ChildIter<'a>(core::slice::Iter<'a, ScopeId>);
impl Iterator for ChildIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied().map(NodeIdLike::into_node_id)
    }
}

pub struct TaffyTree<'a, T, M>
where
    T: 'static,
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>,
{
    composer: &'a mut Composer<LayoutNode<T>>,
    measure_function: M,
}

impl<'a, T, M> TaffyTree<'a, T, M>
where
    T: 'static,
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>,
{
    pub fn new(composer: &'a mut Composer<LayoutNode<T>>, measure_function: M) -> Self {
        Self {
            composer,
            measure_function,
        }
    }
}

impl<T, M> TraversePartialTree for TaffyTree<'_, T, M>
where
    T: 'static,
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>,
{
    type ChildIter<'a>
        = ChildIter<'a>
    where
        Self: 'a;

    #[inline(always)]

    fn child_ids(&self, parent_node_id: NodeId) -> Self::ChildIter<'_> {
        let scope_id = parent_node_id.into_scope_id();
        ChildIter(self.composer.nodes[&scope_id].children.iter())
    }

    #[inline(always)]

    fn child_count(&self, parent_node_id: NodeId) -> usize {
        let scope_id = parent_node_id.into_scope_id();
        self.composer.nodes[&scope_id].children.len()
    }

    #[inline(always)]
    fn get_child_id(&self, parent_node_id: NodeId, child_index: usize) -> NodeId {
        let scope_id = parent_node_id.into_scope_id();
        NodeId::new(self.composer.nodes[&scope_id].children[child_index].into())
    }
}

impl<T, M> TraverseTree for TaffyTree<'_, T, M> where
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>
{
}

impl<T, M> CacheTree for TaffyTree<'_, T, M>
where
    T: 'static,
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>,
{
    fn cache_get(
        &self,
        node_id: NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.composer.nodes[&node_id.into_scope_id()]
            .data
            .as_ref()
            .unwrap()
            .cache
            .get(known_dimensions, available_space, run_mode)
    }

    fn cache_store(
        &mut self,
        node_id: NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        self.composer
            .nodes
            .get_mut(&node_id.into_scope_id())
            .unwrap()
            .data
            .as_mut()
            .unwrap()
            .cache
            .store(known_dimensions, available_space, run_mode, layout_output)
    }

    fn cache_clear(&mut self, node_id: NodeId) {
        self.composer
            .nodes
            .get_mut(&node_id.into_scope_id())
            .unwrap()
            .data
            .as_mut()
            .unwrap()
            .cache
            .clear();
    }
}

impl<T, M> PrintTree for TaffyTree<'_, T, M>
where
    T: 'static,
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>,
{
    #[inline(always)]
    fn get_debug_label(&self, node_id: NodeId) -> &'static str {
        let node = self.composer.nodes[&node_id.into_scope_id()]
            .data
            .as_ref()
            .unwrap();
        let display = node.style.display;
        let num_children = self.child_count(node_id);

        match (num_children, display) {
            (_, Display::None) => "NONE",
            (0, _) => "LEAF",
            (_, Display::Block) => "BLOCK",
            (_, Display::Flex) => match node.style.flex_direction {
                FlexDirection::Row | FlexDirection::RowReverse => "FLEX ROW",
                FlexDirection::Column | FlexDirection::ColumnReverse => "FLEX COL",
            },
            (_, Display::Grid) => "GRID",
        }
    }

    fn get_final_layout(&self, node_id: NodeId) -> &Layout {
        if self.composer.context.use_rounding {
            &self.composer.nodes[&node_id.into_scope_id()]
                .data
                .as_ref()
                .unwrap()
                .final_layout
        } else {
            &self.composer.nodes[&node_id.into_scope_id()]
                .data
                .as_ref()
                .unwrap()
                .unrounded_layout
        }
    }
}

impl<T, M> LayoutPartialTree for TaffyTree<'_, T, M>
where
    T: 'static,
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>,
{
    type CoreContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[inline(always)]
    fn get_core_container_style(&self, node_id: NodeId) -> Self::CoreContainerStyle<'_> {
        &self.composer.nodes[&node_id.into_scope_id()]
            .data
            .as_ref()
            .unwrap()
            .style
    }

    fn set_unrounded_layout(&mut self, node_id: NodeId, layout: &Layout) {
        self.composer
            .nodes
            .get_mut(&node_id.into_scope_id())
            .unwrap()
            .data
            .as_mut()
            .unwrap()
            .unrounded_layout = *layout;
    }

    fn compute_child_layout(
        &mut self,
        node: NodeId,
        inputs: taffy::LayoutInput,
    ) -> taffy::LayoutOutput {
        // If RunMode is PerformHiddenLayout then this indicates that an ancestor node is `Display::None`
        // and thus that we should lay out this node using hidden layout regardless of it's own display style.
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            return compute_hidden_layout(self, node);
        }

        // We run the following wrapped in "compute_cached_layout", which will check the cache for an entry matching the node and inputs and:
        //   - Return that entry if exists
        //   - Else call the passed closure (below) to compute the result
        //
        // If there was no cache match and a new result needs to be computed then that result will be added to the cache
        compute_cached_layout(self, node, inputs, |tree, node, inputs| {
            let scope_id = node.into_scope_id();
            let display_mode = tree.composer.nodes[&scope_id]
                .data
                .as_ref()
                .unwrap()
                .style
                .display;
            let has_children = tree.child_count(node) > 0;

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (display_mode, has_children) {
                (Display::None, _) => compute_hidden_layout(tree, node),
                (Display::Block, true) => compute_block_layout(tree, node, inputs),
                (Display::Flex, true) => compute_flexbox_layout(tree, node, inputs),
                (Display::Grid, true) => compute_grid_layout(tree, node, inputs),
                (_, false) => {
                    let data = tree
                        .composer
                        .nodes
                        .get_mut(&scope_id)
                        .unwrap()
                        .data
                        .as_mut()
                        .unwrap();
                    let style = &data.style;
                    let node_context = data.context.as_mut();
                    let measure_function = |known_dimensions, available_space| {
                        (tree.measure_function)(
                            known_dimensions,
                            available_space,
                            node,
                            node_context,
                            style,
                        )
                    };
                    compute_leaf_layout(inputs, style, measure_function)
                }
            }
        })
    }
}

impl<T, M> LayoutBlockContainer for TaffyTree<'_, T, M>
where
    T: 'static,
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>,
{
    type BlockContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type BlockItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[inline(always)]
    fn get_block_container_style(&self, node_id: NodeId) -> Self::BlockContainerStyle<'_> {
        self.get_core_container_style(node_id)
    }

    #[inline(always)]
    fn get_block_child_style(&self, child_node_id: NodeId) -> Self::BlockItemStyle<'_> {
        self.get_core_container_style(child_node_id)
    }
}

impl<T, M> LayoutFlexboxContainer for TaffyTree<'_, T, M>
where
    T: 'static,
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>,
{
    type FlexboxContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type FlexboxItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[inline(always)]
    fn get_flexbox_container_style(&self, node_id: NodeId) -> Self::FlexboxContainerStyle<'_> {
        &self.composer.nodes[&node_id.into_scope_id()]
            .data
            .as_ref()
            .unwrap()
            .style
    }

    #[inline(always)]
    fn get_flexbox_child_style(&self, child_node_id: NodeId) -> Self::FlexboxItemStyle<'_> {
        &self.composer.nodes[&child_node_id.into_scope_id()]
            .data
            .as_ref()
            .unwrap()
            .style
    }
}

impl<T, M> LayoutGridContainer for TaffyTree<'_, T, M>
where
    T: 'static,
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>,
{
    type GridContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    type GridItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[inline(always)]
    fn get_grid_container_style(&self, node_id: NodeId) -> Self::GridContainerStyle<'_> {
        &self.composer.nodes[&node_id.into_scope_id()]
            .data
            .as_ref()
            .unwrap()
            .style
    }

    #[inline(always)]
    fn get_grid_child_style(&self, child_node_id: NodeId) -> Self::GridItemStyle<'_> {
        &self.composer.nodes[&child_node_id.into_scope_id()]
            .data
            .as_ref()
            .unwrap()
            .style
    }
}

impl<T, M> RoundTree for TaffyTree<'_, T, M>
where
    T: 'static,
    M: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut T>, &Style) -> Size<f32>,
{
    #[inline(always)]
    fn get_unrounded_layout(&self, node_id: NodeId) -> &Layout {
        &self.composer.nodes[&node_id.into_scope_id()]
            .data
            .as_ref()
            .unwrap()
            .unrounded_layout
    }

    #[inline(always)]
    fn set_final_layout(&mut self, node_id: NodeId, layout: &Layout) {
        self.composer
            .nodes
            .get_mut(&node_id.into_scope_id())
            .unwrap()
            .data
            .as_mut()
            .unwrap()
            .final_layout = *layout;
    }
}

pub trait LayoutTree<NodeContext> {
    fn root(&self) -> ScopeId;

    fn compute_layout_with<MeasureFn>(
        &mut self,
        scope: ScopeId,
        available_space: Size<AvailableSpace>,
        measure_fn: MeasureFn,
    ) where
        MeasureFn: FnMut(
            Size<Option<f32>>,
            Size<AvailableSpace>,
            NodeId,
            Option<&mut NodeContext>,
            &Style,
        ) -> Size<f32>;

    #[inline(always)]
    fn compute_layout(&mut self, available_space: Size<AvailableSpace>) {
        let scope = self.root();
        self.compute_layout_with(scope, available_space, |_, _, _, _, _| Size::ZERO);
    }

    fn print_layout_tree_with(&mut self, scope: ScopeId);

    #[inline(always)]
    fn print_layout_tree(&mut self) {
        let scope = self.root();
        self.print_layout_tree_with(scope);
    }
}

impl<T> LayoutTree<T> for Recomposer<LayoutNode<T>>
where
    T: 'static,
{
    #[inline(always)]
    fn root(&self) -> ScopeId {
        self.root_scope()
    }

    fn compute_layout_with<MeasureFn>(
        &mut self,
        scope: ScopeId,
        available_space: Size<AvailableSpace>,
        measure_function: MeasureFn,
    ) where
        MeasureFn: FnMut(
            Size<Option<f32>>,
            Size<AvailableSpace>,
            NodeId,
            Option<&mut T>,
            &Style,
        ) -> Size<f32>,
    {
        self.with_composer_mut(|composer| {
            let node_id = scope.into_node_id();
            let mut tree = TaffyTree::new(composer, measure_function);
            compute_root_layout(&mut tree, node_id, available_space);
            if tree.composer.context.use_rounding {
                round_layout(&mut tree, node_id);
            }
        });
    }

    fn print_layout_tree_with(&mut self, scope: ScopeId) {
        self.with_composer_mut(|composer| {
            let mut tree = TaffyTree::new(composer, |_, _, _, _, _| Size::ZERO);
            print_tree(&mut tree, scope.into_node_id());
        });
    }
}
