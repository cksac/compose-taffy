# compose-taffy
![Rust](https://github.com/cksac/compose-taffy/workflows/Rust/badge.svg)
[![Docs Status](https://docs.rs/compose-taffy/badge.svg)](https://docs.rs/compose-taffy)
[![Latest Version](https://img.shields.io/crates/v/compose-taffy.svg)](https://crates.io/crates/compose-taffy)

A layout tree implementation using [compose-rt](https://github.com/cksac/compose-rt) and [taffy](https://github.com/DioxusLabs/taffy) crate.

## Example
```rust
use compose_rt::{Composer, Root};
use compose_taffy::impls::{LayoutNode, TaffyConfig};
use compose_taffy::TaffyLayout;
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
                n.mark_dirty();
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
```


## LICENSE
This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.
