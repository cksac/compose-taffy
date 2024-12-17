use compose_rt::{ComposeNode, NodeKey};
use taffy::{Display, Layout, NodeId};

pub trait IntoNodeId {
    fn into_node_id(self) -> NodeId;
}

impl IntoNodeId for NodeKey {
    #[inline(always)]
    fn into_node_id(self) -> NodeId {
        NodeId::from(self)
    }
}

pub trait IntoNodeKey {
    fn into_node_key(self) -> NodeKey;
}

impl IntoNodeKey for NodeId {
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

    #[cfg(feature = "block_layout")]
    /// The style type representing the CSS Block container's styles
    type BlockContainerStyle<'a>: taffy::BlockContainerStyle
    where
        Self: 'a;

    #[cfg(feature = "block_layout")]
    /// The style type representing each CSS Block item's styles
    type BlockItemStyle<'a>: taffy::BlockItemStyle
    where
        Self: 'a;

    #[cfg(feature = "flexbox")]
    /// The style type representing the Flexbox container's styles
    type FlexboxContainerStyle<'a>: taffy::FlexboxContainerStyle
    where
        Self: 'a;

    #[cfg(feature = "flexbox")]
    /// The style type representing each Flexbox item's styles
    type FlexboxItemStyle<'a>: taffy::FlexboxItemStyle
    where
        Self: 'a;

    #[cfg(feature = "grid")]
    /// The style type representing the CSS Grid container's styles
    type GridContainerStyle<'a>: taffy::GridContainerStyle
    where
        Self: 'a;

    #[cfg(feature = "grid")]
    /// The style type representing each CSS Grid item's styles
    type GridItemStyle<'a>: taffy::GridItemStyle
    where
        Self: 'a;

    fn get_node_context(&self) -> Option<&Self::NodeContext>;
    fn get_node_context_mut(&mut self) -> Option<&mut Self::NodeContext>;
    fn get_node_context_mut_with_core_style(
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
    #[cfg(feature = "block_layout")]
    fn get_block_container_style(&self) -> Self::BlockContainerStyle<'_>;
    #[cfg(feature = "block_layout")]
    fn get_block_item_style(&self) -> Self::BlockItemStyle<'_>;
    #[cfg(feature = "flexbox")]
    fn get_flexbox_container_style(&self) -> Self::FlexboxContainerStyle<'_>;
    #[cfg(feature = "flexbox")]
    fn get_flexbox_item_style(&self) -> Self::FlexboxItemStyle<'_>;
    #[cfg(feature = "grid")]
    fn get_grid_container_style(&self) -> Self::GridContainerStyle<'_>;
    #[cfg(feature = "grid")]
    fn get_grid_item_style(&self) -> Self::GridItemStyle<'_>;
}
