use compose_rt::{ComposeNode, Composer, NodeKey, Recomposer};
#[cfg(feature = "block_layout")]
use taffy::{compute::compute_block_layout, LayoutBlockContainer};
#[cfg(feature = "flexbox")]
use taffy::{compute::compute_flexbox_layout, LayoutFlexboxContainer};
#[cfg(feature = "grid")]
use taffy::{compute::compute_grid_layout, LayoutGridContainer};
use taffy::{
    compute_cached_layout, compute_hidden_layout, compute_leaf_layout, compute_root_layout,
    print_tree, round_layout, AvailableSpace, CacheTree, Display, FlexDirection,
    FlexboxContainerStyle, Layout, LayoutPartialTree, NodeId, PrintTree, RoundTree, RunMode, Size,
    TraversePartialTree, TraverseTree,
};

pub trait NodeIdLike {
    fn into_node_id(self) -> NodeId;
}

impl NodeIdLike for NodeKey {
    #[inline(always)]
    fn into_node_id(self) -> NodeId {
        NodeId::from(self)
    }
}

pub trait NodeKeyLike {
    fn into_node_key(self) -> NodeKey;
}

impl NodeKeyLike for NodeId {
    #[inline(always)]
    fn into_node_key(self) -> NodeKey {
        self.into()
    }
}

pub trait TaffyConfig {
    fn use_rounding(&self) -> bool;
}

pub trait TaffyNode: ComposeNode + 'static {
    type NodeContext;

    /// The style type representing the core container styles that all containers should have
    /// Used when laying out the root node of a tree
    type CoreContainerStyle: taffy::CoreStyle;

    /// The style type representing the CSS Block container's styles
    type BlockContainerStyle<'a>: taffy::BlockContainerStyle
    where
        Self: 'a;
    /// The style type representing each CSS Block item's styles
    type BlockItemStyle<'a>: taffy::BlockItemStyle
    where
        Self: 'a;

    /// The style type representing the Flexbox container's styles
    type FlexboxContainerStyle<'a>: taffy::FlexboxContainerStyle
    where
        Self: 'a;
    /// The style type representing each Flexbox item's styles
    type FlexboxItemStyle<'a>: taffy::FlexboxItemStyle
    where
        Self: 'a;

    /// The style type representing the CSS Grid container's styles
    type GridContainerStyle<'a>: taffy::GridContainerStyle
    where
        Self: 'a;

    /// The style type representing each CSS Grid item's styles
    type GridItemStyle<'a>: taffy::GridItemStyle
    where
        Self: 'a;

    fn get_node_context(&self) -> Option<&Self::NodeContext>;
    fn get_node_context_mut(&mut self) -> Option<&mut Self::NodeContext>;

    fn get_node_context_mut_with_style(
        &mut self,
    ) -> (Option<&mut Self::NodeContext>, &Self::CoreContainerStyle);

    fn get_display(&self) -> Display;
    fn get_final_layout(&self) -> &Layout;
    fn set_final_layout(&mut self, layout: &Layout);
    fn get_unrounded_layout(&self) -> &Layout;
    fn set_unrounded_layout(&mut self, layout: &Layout);
    fn cache_get(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput>;
    fn cache_store(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    );
    fn cache_clear(&mut self);
    fn get_core_container_style(&self) -> &Self::CoreContainerStyle;
    fn get_block_container_style(&self) -> Self::BlockContainerStyle<'_>;
    fn get_block_item_style(&self) -> Self::BlockItemStyle<'_>;
    fn get_flexbox_container_style(&self) -> Self::FlexboxContainerStyle<'_>;
    fn get_flexbox_item_style(&self) -> Self::FlexboxItemStyle<'_>;
    fn get_grid_container_style(&self) -> Self::GridContainerStyle<'_>;
    fn get_grid_item_style(&self) -> Self::GridItemStyle<'_>;
}

pub struct ChildIter<'a>(core::slice::Iter<'a, NodeKey>);
impl Iterator for ChildIter<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied().map(NodeIdLike::into_node_id)
    }
}

pub struct TaffyTreeView<'a, T>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
{
    composer: &'a Composer<T>,
}

impl<'a, T> TaffyTreeView<'a, T>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
{
    pub fn new(composer: &'a Composer<T>) -> Self {
        Self { composer }
    }
}

impl<T> TraversePartialTree for TaffyTreeView<'_, T>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
{
    type ChildIter<'a>
        = ChildIter<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn child_ids(&self, node_id: NodeId) -> Self::ChildIter<'_> {
        let node_key = node_id.into_node_key();
        ChildIter(self.composer.nodes[node_key].children.iter())
    }

    #[inline(always)]
    fn child_count(&self, node_id: NodeId) -> usize {
        let node_key = node_id.into_node_key();
        self.composer.nodes[node_key].children.len()
    }

    #[inline(always)]
    fn get_child_id(&self, node_id: NodeId, child_index: usize) -> NodeId {
        let node_key = node_id.into_node_key();
        self.composer.nodes[node_key].children[child_index].into_node_id()
    }
}

impl<T> TraverseTree for TaffyTreeView<'_, T>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
{
}

impl<T> PrintTree for TaffyTreeView<'_, T>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
{
    #[inline(always)]
    fn get_debug_label(&self, node_id: NodeId) -> &'static str {
        let node = self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap();
        let display = node.get_display();
        let num_children = self.child_count(node_id);
        match (num_children, display) {
            (_, Display::None) => "NONE",
            (0, _) => "LEAF",
            #[cfg(feature = "block_layout")]
            (_, Display::Block) => "BLOCK",
            #[cfg(feature = "flexbox")]
            (_, Display::Flex) => match node.get_flexbox_container_style().flex_direction() {
                FlexDirection::Row | FlexDirection::RowReverse => "FLEX ROW",
                FlexDirection::Column | FlexDirection::ColumnReverse => "FLEX COL",
            },
            #[cfg(feature = "grid")]
            (_, Display::Grid) => "GRID",
        }
    }

    #[inline(always)]
    fn get_final_layout(&self, node_id: NodeId) -> &Layout {
        if self.composer.context.use_rounding() {
            self.composer.nodes[node_id.into_node_key()]
                .data
                .as_ref()
                .unwrap()
                .get_final_layout()
        } else {
            self.composer.nodes[node_id.into_node_key()]
                .data
                .as_ref()
                .unwrap()
                .get_unrounded_layout()
        }
    }
}

pub struct TaffyTree<'a, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
    composer: &'a mut Composer<T>,
    measure_function: M,
}

impl<'a, T, M> TaffyTree<'a, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
    pub fn new(composer: &'a mut Composer<T>, measure_function: M) -> Self {
        Self {
            composer,
            measure_function,
        }
    }
}

impl<T, M> TraversePartialTree for TaffyTree<'_, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
    type ChildIter<'a>
        = ChildIter<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn child_ids(&self, node_id: NodeId) -> Self::ChildIter<'_> {
        let node_key = node_id.into_node_key();
        ChildIter(self.composer.nodes[node_key].children.iter())
    }

    #[inline(always)]
    fn child_count(&self, node_id: NodeId) -> usize {
        let node_key = node_id.into_node_key();
        self.composer.nodes[node_key].children.len()
    }

    #[inline(always)]
    fn get_child_id(&self, node_id: NodeId, child_index: usize) -> NodeId {
        let node_key = node_id.into_node_key();
        self.composer.nodes[node_key].children[child_index].into_node_id()
    }
}

impl<T, M> TraverseTree for TaffyTree<'_, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
}

impl<T, M> CacheTree for TaffyTree<'_, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
    fn cache_get(
        &self,
        node_id: NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap()
            .cache_get(known_dimensions, available_space, run_mode)
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
            .get_mut(node_id.into_node_key())
            .unwrap()
            .data
            .as_mut()
            .unwrap()
            .cache_store(known_dimensions, available_space, run_mode, layout_output)
    }

    fn cache_clear(&mut self, node_id: NodeId) {
        self.composer
            .nodes
            .get_mut(node_id.into_node_key())
            .unwrap()
            .data
            .as_mut()
            .unwrap()
            .cache_clear();
    }
}

impl<T, M> PrintTree for TaffyTree<'_, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
    #[inline(always)]
    fn get_debug_label(&self, node_id: NodeId) -> &'static str {
        let node = self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap();
        let display = node.get_display();
        let num_children = self.child_count(node_id);

        match (num_children, display) {
            (_, Display::None) => "NONE",
            (0, _) => "LEAF",
            #[cfg(feature = "block_layout")]
            (_, Display::Block) => "BLOCK",
            #[cfg(feature = "flexbox")]
            (_, Display::Flex) => match node.get_flexbox_container_style().flex_direction() {
                FlexDirection::Row | FlexDirection::RowReverse => "FLEX ROW",
                FlexDirection::Column | FlexDirection::ColumnReverse => "FLEX COL",
            },
            #[cfg(feature = "grid")]
            (_, Display::Grid) => "GRID",
        }
    }

    #[inline(always)]
    fn get_final_layout(&self, node_id: NodeId) -> &Layout {
        if self.composer.context.use_rounding() {
            self.composer.nodes[node_id.into_node_key()]
                .data
                .as_ref()
                .unwrap()
                .get_final_layout()
        } else {
            self.composer.nodes[node_id.into_node_key()]
                .data
                .as_ref()
                .unwrap()
                .get_unrounded_layout()
        }
    }
}

impl<T, M> LayoutPartialTree for TaffyTree<'_, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
    type CoreContainerStyle<'a>
        = &'a T::CoreContainerStyle
    where
        Self: 'a;

    #[inline(always)]
    fn get_core_container_style(&self, node_id: NodeId) -> Self::CoreContainerStyle<'_> {
        self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap()
            .get_core_container_style()
    }

    #[inline(always)]
    fn set_unrounded_layout(&mut self, node_id: NodeId, layout: &Layout) {
        self.composer
            .nodes
            .get_mut(node_id.into_node_key())
            .unwrap()
            .data
            .as_mut()
            .unwrap()
            .set_unrounded_layout(layout);
    }

    #[inline(always)]
    fn compute_child_layout(
        &mut self,
        node_id: NodeId,
        inputs: taffy::LayoutInput,
    ) -> taffy::LayoutOutput {
        // If RunMode is PerformHiddenLayout then this indicates that an ancestor node is `Display::None`
        // and thus that we should lay out this node using hidden layout regardless of it's own display style.
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            return compute_hidden_layout(self, node_id);
        }

        // We run the following wrapped in "compute_cached_layout", which will check the cache for an entry matching the node and inputs and:
        //   - Return that entry if exists
        //   - Else call the passed closure (below) to compute the result
        //
        // If there was no cache match and a new result needs to be computed then that result will be added to the cache
        compute_cached_layout(self, node_id, inputs, |tree, node_id, inputs| {
            let node_key = node_id.into_node_key();
            let display_mode = tree.composer.nodes[node_key]
                .data
                .as_ref()
                .unwrap()
                .get_display();
            let has_children = tree.child_count(node_id) > 0;

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (display_mode, has_children) {
                (Display::None, _) => compute_hidden_layout(tree, node_id),
                #[cfg(feature = "block_layout")]
                (Display::Block, true) => compute_block_layout(tree, node_id, inputs),
                #[cfg(feature = "flexbox")]
                (Display::Flex, true) => compute_flexbox_layout(tree, node_id, inputs),
                #[cfg(feature = "grid")]
                (Display::Grid, true) => compute_grid_layout(tree, node_id, inputs),
                (_, false) => {
                    let data = tree
                        .composer
                        .nodes
                        .get_mut(node_key)
                        .unwrap()
                        .data
                        .as_mut()
                        .unwrap();
                    //let style = data.get_core_container_style();
                    //let node_context = data.get_node_context_mut();
                    let (node_context, style) = data.get_node_context_mut_with_style();
                    let measure_function = |known_dimensions, available_space| {
                        (tree.measure_function)(
                            known_dimensions,
                            available_space,
                            node_id,
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

#[cfg(feature = "block_layout")]
impl<T, M> LayoutBlockContainer for TaffyTree<'_, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
    type BlockContainerStyle<'a>
        = T::BlockContainerStyle<'a>
    where
        Self: 'a;

    type BlockItemStyle<'a>
        = T::BlockItemStyle<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn get_block_container_style(&self, node_id: NodeId) -> Self::BlockContainerStyle<'_> {
        self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap()
            .get_block_container_style()
    }

    #[inline(always)]
    fn get_block_child_style(&self, node_id: NodeId) -> Self::BlockItemStyle<'_> {
        self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap()
            .get_block_item_style()
    }
}

#[cfg(feature = "flexbox")]
impl<T, M> LayoutFlexboxContainer for TaffyTree<'_, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
    type FlexboxContainerStyle<'a>
        = T::FlexboxContainerStyle<'a>
    where
        Self: 'a;

    type FlexboxItemStyle<'a>
        = T::FlexboxItemStyle<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn get_flexbox_container_style(&self, node_id: NodeId) -> Self::FlexboxContainerStyle<'_> {
        self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap()
            .get_flexbox_container_style()
    }

    #[inline(always)]
    fn get_flexbox_child_style(&self, node_id: NodeId) -> Self::FlexboxItemStyle<'_> {
        self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap()
            .get_flexbox_item_style()
    }
}

#[cfg(feature = "grid")]
impl<T, M> LayoutGridContainer for TaffyTree<'_, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
    type GridContainerStyle<'a>
        = T::GridContainerStyle<'a>
    where
        Self: 'a;

    type GridItemStyle<'a>
        = T::GridItemStyle<'a>
    where
        Self: 'a;

    #[inline(always)]
    fn get_grid_container_style(&self, node_id: NodeId) -> Self::GridContainerStyle<'_> {
        self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap()
            .get_grid_container_style()
    }

    #[inline(always)]
    fn get_grid_child_style(&self, node_id: NodeId) -> Self::GridItemStyle<'_> {
        self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap()
            .get_grid_item_style()
    }
}

impl<T, M> RoundTree for TaffyTree<'_, T, M>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
    M: FnMut(
        Size<Option<f32>>,
        Size<AvailableSpace>,
        NodeId,
        Option<&mut T::NodeContext>,
        &T::CoreContainerStyle,
    ) -> Size<f32>,
{
    #[inline(always)]
    fn get_unrounded_layout(&self, node_id: NodeId) -> &Layout {
        self.composer.nodes[node_id.into_node_key()]
            .data
            .as_ref()
            .unwrap()
            .get_unrounded_layout()
    }

    #[inline(always)]
    fn set_final_layout(&mut self, node_id: NodeId, layout: &Layout) {
        self.composer
            .nodes
            .get_mut(node_id.into_node_key())
            .unwrap()
            .data
            .as_mut()
            .unwrap()
            .set_final_layout(layout);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// The supplied node was not found in the composer instance.
    InvalidInputNode(NodeKey),
}

pub type LayoutResult = std::result::Result<(), LayoutError>;

pub trait LayoutTree<NodeContext, MeasureFnStyle> {
    fn compute_layout_with<MeasureFn>(
        &mut self,
        available_space: Size<AvailableSpace>,
        node_key: NodeKey,
        measure_fn: MeasureFn,
    ) -> LayoutResult
    where
        MeasureFn: FnMut(
            Size<Option<f32>>,
            Size<AvailableSpace>,
            NodeId,
            Option<&mut NodeContext>,
            &MeasureFnStyle,
        ) -> Size<f32>;

    fn compute_layout(&mut self, available_space: Size<AvailableSpace>) -> LayoutResult;

    fn print_layout_tree_with(&self, node_key: NodeKey) -> LayoutResult;

    fn print_layout_tree(&self) -> LayoutResult;
}

impl<T> LayoutTree<T::NodeContext, T::CoreContainerStyle> for Recomposer<T>
where
    T: TaffyNode,
    T::Context: TaffyConfig,
{
    fn compute_layout_with<MeasureFn>(
        &mut self,
        available_space: Size<AvailableSpace>,
        node_key: NodeKey,
        measure_function: MeasureFn,
    ) -> LayoutResult
    where
        MeasureFn: FnMut(
            Size<Option<f32>>,
            Size<AvailableSpace>,
            NodeId,
            Option<&mut T::NodeContext>,
            &T::CoreContainerStyle,
        ) -> Size<f32>,
    {
        self.with_composer_mut(|composer| {
            if !composer.nodes.contains(node_key) {
                return Err(LayoutError::InvalidInputNode(node_key));
            }
            let node_id = node_key.into_node_id();
            let mut tree = TaffyTree::new(composer, measure_function);
            compute_root_layout(&mut tree, node_id, available_space);
            if tree.composer.context.use_rounding() {
                round_layout(&mut tree, node_id);
            }
            Ok(())
        })
    }

    #[inline(always)]
    fn compute_layout(&mut self, available_space: Size<AvailableSpace>) -> LayoutResult {
        let node_key = self.root_node_key();
        self.compute_layout_with(available_space, node_key, |_, _, _, _, _| Size::ZERO)
    }

    fn print_layout_tree_with(&self, node_key: NodeKey) -> LayoutResult {
        self.with_composer(|composer| {
            if !composer.nodes.contains(node_key) {
                return Err(LayoutError::InvalidInputNode(node_key));
            }
            let node_id = node_key.into_node_id();
            let tree = TaffyTreeView::new(composer);
            print_tree(&tree, node_id);
            Ok(())
        })
    }

    #[inline(always)]
    fn print_layout_tree(&self) -> LayoutResult {
        let node_key = self.root_node_key();
        self.print_layout_tree_with(node_key)
    }
}
