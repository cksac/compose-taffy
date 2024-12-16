use compose_rt::{Composer, Root};
use compose_taffy::impls::{LayoutNode, TaffyConfig};
use compose_taffy::LayoutTree;
use taffy::{AvailableSpace, Dimension, JustifyContent, Size, Style};

type Scope<T> = compose_taffy::impls::Scope<T, ()>;

struct Container;

#[track_caller]
fn container<P, C>(s: Scope<P>, style: Style, content: C)
where
    P: 'static,
    C: Fn(Scope<Container>) + Clone + 'static,
{
    let scope = s.child::<Container>();
    s.create_node(
        scope,
        content,
        move |_| style.clone(),
        |style, _| LayoutNode::new(style),
        |n, style, _| {
            if n.style != style {
                n.style = style;
            }
        },
    );
}

struct Leaf;
#[track_caller]
fn leaf<P>(s: Scope<P>, style: Style)
where
    P: 'static,
{
    let scope = s.child::<Leaf>();
    s.create_node(
        scope,
        |_| {},
        move |_| style.clone(),
        |style, _| LayoutNode::new(style),
        |n, style, _| {
            if n.style != style {
                n.style = style;
                n.mark_dirty();
            }
        },
    );
}

fn app(s: Scope<Root>) {
    container(
        s,
        Style {
            size: Size {
                width: Dimension::Length(100.0),
                height: Dimension::Length(100.0),
            },
            justify_content: Some(JustifyContent::Center),
            ..Default::default()
        },
        |s| {
            leaf(
                s,
                Style {
                    size: Size {
                        width: Dimension::Percent(0.5),
                        height: Dimension::Auto,
                    },
                    ..Default::default()
                },
            );
        },
    );
}

fn main() {
    let mut recomposer = Composer::compose(app, TaffyConfig::default());
    let _ = recomposer.compute_layout(Size {
        height: AvailableSpace::Definite(100.0),
        width: AvailableSpace::Definite(100.0),
    });
    let _ = recomposer.print_layout_tree();
}
