use compose_rt::ComposeNode;
use taffy::{Cache, Layout, Style};

use crate::traits;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaffyConfig {
    pub use_rounding: bool,
}

impl Default for TaffyConfig {
    fn default() -> Self {
        Self { use_rounding: true }
    }
}

impl TaffyConfig {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    pub fn with_rounding(mut self, use_rounding: bool) -> Self {
        self.use_rounding = use_rounding;
        self
    }

    #[inline(always)]
    pub fn enable_rounding(&mut self) {
        self.use_rounding = true;
    }

    #[inline(always)]
    pub fn disable_rounding(&mut self) {
        self.use_rounding = false;
    }
}

impl traits::TaffyConfig for TaffyConfig {
    #[inline(always)]
    fn use_rounding(&self) -> bool {
        self.use_rounding
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

impl<T> LayoutNode<T>
where
    T: 'static,
{
    #[inline(always)]
    pub fn new(style: Style) -> Self {
        Self {
            style,
            unrounded_layout: Layout::new(),
            final_layout: Layout::new(),
            cache: Cache::new(),
            context: None,
        }
    }

    #[inline(always)]
    pub fn with_context(style: Style, context: T) -> Self {
        Self {
            style,
            unrounded_layout: Layout::new(),
            final_layout: Layout::new(),
            cache: Cache::new(),
            context: Some(context),
        }
    }

    #[inline(always)]
    pub fn mark_dirty(&mut self) {
        self.cache.clear();
    }
}

impl<T> ComposeNode for LayoutNode<T>
where
    T: 'static,
{
    type Context = TaffyConfig;
}

impl<T> traits::TaffyNode for LayoutNode<T>
where
    T: 'static,
{
    type NodeContext = T;

    type CoreContainerStyle = Style;

    #[cfg(feature = "block_layout")]
    type BlockContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[cfg(feature = "block_layout")]
    type BlockItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[cfg(feature = "flexbox")]
    type FlexboxContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[cfg(feature = "flexbox")]
    type FlexboxItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[cfg(feature = "grid")]
    type GridContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[cfg(feature = "grid")]
    type GridItemStyle<'a>
        = &'a Style
    where
        Self: 'a;

    #[inline(always)]
    fn get_node_context(&self) -> Option<&Self::NodeContext> {
        self.context.as_ref()
    }

    #[inline(always)]
    fn get_node_context_mut(&mut self) -> Option<&mut Self::NodeContext> {
        self.context.as_mut()
    }

    #[inline(always)]
    fn get_node_context_mut_with_core_style(
        &mut self,
    ) -> (Option<&mut Self::NodeContext>, &Self::CoreContainerStyle) {
        (self.context.as_mut(), &self.style)
    }

    #[inline(always)]
    fn get_display(&self) -> taffy::Display {
        self.style.display
    }

    #[inline(always)]
    fn get_final_layout(&self) -> &taffy::Layout {
        &self.final_layout
    }

    #[inline(always)]
    fn set_final_layout(&mut self, layout: &taffy::Layout) {
        self.final_layout = *layout;
    }

    #[inline(always)]
    fn get_unrounded_layout(&self) -> &taffy::Layout {
        &self.unrounded_layout
    }

    #[inline(always)]
    fn set_unrounded_layout(&mut self, layout: &taffy::Layout) {
        self.unrounded_layout = *layout;
    }

    #[inline(always)]
    fn cache_get(
        &self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.cache.get(known_dimensions, available_space, run_mode)
    }

    #[inline(always)]
    fn cache_store(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        self.cache
            .store(known_dimensions, available_space, run_mode, layout_output)
    }

    #[inline(always)]
    fn cache_clear(&mut self) {
        self.cache.clear()
    }

    #[inline(always)]
    fn get_core_container_style(&self) -> &Self::CoreContainerStyle {
        &self.style
    }

    #[cfg(feature = "block_layout")]
    #[inline(always)]
    fn get_block_container_style(&self) -> Self::BlockContainerStyle<'_> {
        &self.style
    }

    #[cfg(feature = "block_layout")]
    #[inline(always)]
    fn get_block_item_style(&self) -> Self::BlockItemStyle<'_> {
        &self.style
    }

    #[cfg(feature = "flexbox")]
    #[inline(always)]
    fn get_flexbox_container_style(&self) -> Self::FlexboxContainerStyle<'_> {
        &self.style
    }

    #[cfg(feature = "flexbox")]
    #[inline(always)]
    fn get_flexbox_item_style(&self) -> Self::FlexboxItemStyle<'_> {
        &self.style
    }

    #[cfg(feature = "grid")]
    #[inline(always)]
    fn get_grid_container_style(&self) -> Self::GridContainerStyle<'_> {
        &self.style
    }

    #[cfg(feature = "grid")]
    #[inline(always)]
    fn get_grid_item_style(&self) -> Self::GridItemStyle<'_> {
        &self.style
    }
}

pub type Scope<T, C> = compose_rt::Scope<T, LayoutNode<C>>;
pub type State<T, C> = compose_rt::State<T, LayoutNode<C>>;
