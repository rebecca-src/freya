use std::time::Duration;

use dioxus::prelude::*;
use node::launch;
use tokio::time::sleep;
mod elements;
mod node;
mod run;

fn main() {
    launch(app);
}

fn app(cx: Scope) -> Element {
    let colors = use_state(&cx, || vec!["green", "blue", "red"]);
    let padding = use_state(&cx, || 10.0);

    use_effect(&cx, colors, |colors| async move {
        sleep(Duration::from_millis(200)).await;
        colors.with_mut(|colors| colors.reverse());
    });

    use_effect(&cx, padding, |padding| async move {
        sleep(Duration::from_millis(1)).await;
        padding.with_mut(|padding| {
            if *padding < 80.0 {
                *padding += 1.0;
            } else {
                *padding = 5.0;
            }
        });
    });

    let big = colors[0];
    let mid = colors[1];
    let small = colors[2];

    cx.render(rsx! {
        div {
            background: "{big}",
            height: "stretch",
            width: "stretch",
            padding: "50.0",

            div {
                background: "{mid}",
                height: "stretch",
                width: "stretch",
                padding: "{padding}",
                div {
                    background: "{small}",
                    height: "stretch",
                    width: "stretch",
                    padding: "20.0",
                    p {
                        "! :D"
                    }
                }
                p {
                    "World",
                }
            },
            p {
                "Hello",
            }
        }
    })
}
