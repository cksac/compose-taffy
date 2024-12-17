use compose_rt::{NodeKey, Recomposer};
use taffy::{compute_root_layout, print_tree, round_layout, AvailableSpace, NodeId, Size};

use crate::traits::{IntoNodeId, TaffyConfig, TaffyNode};
use crate::{TaffyTree, TaffyTreeView};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    /// The supplied node was not found in the composer instance.
    InvalidInputNode(NodeKey),
}

pub type LayoutResult = std::result::Result<(), LayoutError>;

pub trait TaffyLayout<NodeContext, CoreContainerStyle> {
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
            &CoreContainerStyle,
        ) -> Size<f32>;

    fn compute_layout(&mut self, available_space: Size<AvailableSpace>) -> LayoutResult;

    fn print_layout_tree_with(&self, node_key: NodeKey) -> LayoutResult;

    fn print_layout_tree(&self) -> LayoutResult;
}

impl<T> TaffyLayout<T::NodeContext, T::CoreContainerStyle> for Recomposer<T>
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
