use compose_rt::{Composer, Root};
use compose_taffy::{LayoutNode, LayoutTree, TaffyConfig};
use taffy::{AvailableSpace, Dimension, JustifyContent, Size, Style};

type Scope<T> = compose_taffy::Scope<T, ()>;

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
        move || style.clone(),
        |style| LayoutNode::new(style),
        |n, style| {
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
        move || style.clone(),
        |style| LayoutNode::new(style),
        |n, style| {
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
    let mut recomposer = Composer::compose(app);
    let root = recomposer.root_scope();
    recomposer.compute_layout(
        root,
        TaffyConfig::default(),
        Size {
            height: AvailableSpace::Definite(100.0),
            width: AvailableSpace::Definite(100.0),
        },
    );
    println!("{:#?}", recomposer);
    recomposer.print_layout_tree(root, TaffyConfig::default());
}
