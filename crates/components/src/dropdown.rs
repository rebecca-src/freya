use std::fmt::Display;

use dioxus::prelude::*;
use freya_core::{
    platform::CursorIcon,
    types::AccessibilityId,
};
use freya_elements::{
    self as dioxus_elements,
    events::{
        keyboard::Key,
        KeyboardEvent,
        MouseEvent,
    },
};
use freya_hooks::{
    theme_with,
    use_applied_theme,
    use_focus,
    use_platform,
    DropdownItemTheme,
    DropdownItemThemeWith,
    DropdownTheme,
    DropdownThemeWith,
    IconThemeWith,
    UseFocus,
};

use crate::icons::ArrowIcon;

/// Properties for the [`DropdownItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DropdownItemProps<T: 'static + Clone + PartialEq> {
    /// Theme override.
    pub theme: Option<DropdownItemThemeWith>,
    /// Selectable items, like [`DropdownItem`]
    pub children: Element,
    /// Selected value.
    pub value: T,
    /// Handler for the `onpress` event.
    pub onpress: Option<EventHandler<()>>,
}

/// Current status of the DropdownItem.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum DropdownItemStatus {
    /// Default state.
    #[default]
    Idle,
    /// Dropdown item is being hovered.
    Hovering,
}

/// # Styling
/// Inherits the [`DropdownItemTheme`](freya_hooks::DropdownItemTheme) theme.
#[allow(non_snake_case)]
pub fn DropdownItem<T>(
    DropdownItemProps {
        theme,
        children,
        value,
        onpress,
    }: DropdownItemProps<T>,
) -> Element
where
    T: Clone + PartialEq + 'static,
{
    let selected = use_context::<Signal<T>>();
    let theme = use_applied_theme!(&theme, dropdown_item);
    let focus = use_focus();
    let mut status = use_signal(DropdownItemStatus::default);
    let platform = use_platform();
    let dropdown_group = use_context::<DropdownGroup>();

    let a11y_id = focus.attribute();
    let a11y_member_of = UseFocus::attribute_for_id(dropdown_group.group_id);
    let is_focused = focus.is_focused();
    let is_selected = *selected.read() == value;

    let DropdownItemTheme {
        font_theme,
        background,
        hover_background,
        select_background,
        border_fill,
        select_border_fill,
    } = &theme;

    let background = match *status.read() {
        _ if is_selected => select_background,
        DropdownItemStatus::Hovering => hover_background,
        DropdownItemStatus::Idle => background,
    };
    let border = if focus.is_focused_with_keyboard() {
        format!("2 inner {select_border_fill}")
    } else {
        format!("1 inner {border_fill}")
    };

    use_drop(move || {
        if *status.peek() == DropdownItemStatus::Hovering {
            platform.set_cursor(CursorIcon::default());
        }
    });

    let onmouseenter = move |_| {
        platform.set_cursor(CursorIcon::Pointer);
        status.set(DropdownItemStatus::Hovering);
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        status.set(DropdownItemStatus::default());
    };

    let onglobalkeydown = {
        to_owned![onpress];
        move |ev: KeyboardEvent| {
            if ev.key == Key::Enter && is_focused {
                if let Some(onpress) = &onpress {
                    onpress.call(())
                }
            }
        }
    };

    let onclick = move |_: MouseEvent| {
        if let Some(onpress) = &onpress {
            onpress.call(())
        }
    };

    rsx!(
        rect {
            width: "fill-min",
            color: "{font_theme.color}",
            a11y_id,
            a11y_role: "button",
            a11y_member_of,
            background: "{background}",
            border,
            padding: "6 10",
            corner_radius: "6",
            main_align: "center",
            onmouseenter,
            onmouseleave,
            onclick,
            onglobalkeydown,
            {children}
        }
    )
}

/// Properties for the [`Dropdown`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DropdownProps<T: 'static + Clone + PartialEq> {
    /// Theme override.
    pub theme: Option<DropdownThemeWith>,
    /// Selectable items, like [`DropdownItem`]
    pub children: Element,
    /// Selected value.
    pub value: T,
}

/// Current status of the Dropdown.
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum DropdownStatus {
    /// Default state.
    #[default]
    Idle,
    /// Dropdown is being hovered.
    Hovering,
}

#[derive(Clone)]
struct DropdownGroup {
    group_id: AccessibilityId,
}

/// Select from multiple options, use alongside [`DropdownItem`].
///
/// # Styling
/// Inherits the [`DropdownTheme`](freya_hooks::DropdownTheme) theme.
///
/// # Example
/// ```rust
/// # use freya::prelude::*;
///
/// fn app() -> Element {
///     let values = use_hook(|| vec!["Value A".to_string(), "Value B".to_string(), "Value C".to_string()]);
///     let mut selected_dropdown = use_signal(|| "Value A".to_string());
///     rsx!(
///         Dropdown {
///             value: selected_dropdown.read().clone(),
///             for ch in values {
///                 DropdownItem {
///                     value: ch.to_string(),
///                     onpress: {
///                         to_owned![ch];
///                         move |_| selected_dropdown.set(ch.clone())
///                     },
///                     label { "{ch}" }
///                 }
///             }
///         }
///     )
/// }
/// # use freya_testing::prelude::*;
/// # launch_doc(|| {
/// #   rsx!(
/// #       Preview {
/// #           {app()}
/// #       }
/// #   )
/// # }, (250., 250.).into(), "./images/gallery_dropdown.png");
/// ```
///
/// # Preview
/// ![Dropdown Preview][dropdown]
#[cfg_attr(feature = "docs",
    doc = embed_doc_image::embed_image!("dropdown", "images/gallery_dropdown.png")
)]
#[allow(non_snake_case)]
pub fn Dropdown<T>(props: DropdownProps<T>) -> Element
where
    T: PartialEq + Clone + Display + 'static,
{
    let mut selected = use_context_provider(|| Signal::new(props.value.clone()));
    let theme = use_applied_theme!(&props.theme, dropdown);
    let mut focus = use_focus();
    let mut status = use_signal(DropdownStatus::default);
    let mut opened = use_signal(|| false);
    let platform = use_platform();

    use_context_provider(|| DropdownGroup {
        group_id: focus.id(),
    });

    let is_opened = *opened.read();
    let is_focused = focus.is_focused();
    let a11y_id = focus.attribute();
    let a11y_member_of = focus.attribute();

    if *selected.peek() != props.value {
        *selected.write() = props.value;
    }

    // Close if the focused node is not part of the Dropdown
    use_effect(move || {
        if let Some(member_of) = focus.focused_node().read().member_of() {
            if member_of != focus.id() {
                opened.set(false);
            }
        }
    });

    use_drop(move || {
        if *status.peek() == DropdownStatus::Hovering {
            platform.set_cursor(CursorIcon::default());
        }
    });

    // Close the dropdown if clicked anywhere
    let onglobalclick = move |_: MouseEvent| {
        opened.set(false);
    };

    let onclick = move |_| {
        focus.focus();
        opened.set(true)
    };

    let onglobalkeydown = move |e: KeyboardEvent| {
        match e.key {
            // Close when `Escape` key is pressed
            Key::Escape => {
                opened.set(false);
            }
            // Open the dropdown items when the `Enter` key is pressed
            Key::Enter if is_focused && !is_opened => {
                opened.set(true);
            }
            _ => {}
        }
    };

    let onmouseenter = move |_| {
        platform.set_cursor(CursorIcon::Pointer);
        status.set(DropdownStatus::Hovering);
    };

    let onmouseleave = move |_| {
        platform.set_cursor(CursorIcon::default());
        status.set(DropdownStatus::default());
    };

    let DropdownTheme {
        width,
        margin,
        font_theme,
        dropdown_background,
        background_button,
        hover_background,
        border_fill,
        focus_border_fill,
        arrow_fill,
    } = &theme;

    let background = match *status.read() {
        DropdownStatus::Hovering => hover_background,
        DropdownStatus::Idle => background_button,
    };
    let border = if focus.is_focused_with_keyboard() {
        format!("2 inner {focus_border_fill}")
    } else {
        format!("1 inner {border_fill}")
    };

    let selected = selected.read().to_string();

    rsx!(
        rect {
            direction: "vertical",
            spacing: "4",
            rect {
                width: "{width}",
                onmouseenter,
                onmouseleave,
                onclick,
                onglobalkeydown,
                margin: "{margin}",
                a11y_id,
                a11y_member_of,
                background: "{background}",
                color: "{font_theme.color}",
                corner_radius: "8",
                padding: "6 16",
                border,
                direction: "horizontal",
                main_align: "center",
                cross_align: "center",
                label {
                    "{selected}"
                }
                ArrowIcon {
                    rotate: "0",
                    fill: "{arrow_fill}",
                    theme: theme_with!(IconTheme {
                        margin : "0 0 0 8".into(),
                    })
                }
            }
            if *opened.read() {
                rect {
                    height: "0",
                    width: "0",
                    rect {
                        width: "100v",
                        rect {
                            onglobalclick,
                            onglobalkeydown,
                            layer: "-1000",
                            margin: "{margin}",
                            border: "1 inner {border_fill}",
                            overflow: "clip",
                            corner_radius: "8",
                            background: "{dropdown_background}",
                            shadow: "0 2 4 0 rgb(0, 0, 0, 0.15)",
                            padding: "6",
                            content: "fit",
                            {props.children}
                        }
                    }
                }
            }
        }
    )
}

#[cfg(test)]
mod test {
    use freya::prelude::*;
    use freya_testing::prelude::*;

    #[tokio::test]
    pub async fn dropdown() {
        fn dropdown_app() -> Element {
            let values = use_hook(|| {
                vec![
                    "Value A".to_string(),
                    "Value B".to_string(),
                    "Value C".to_string(),
                ]
            });
            let mut selected_dropdown = use_signal(|| "Value A".to_string());

            rsx!(
                Dropdown {
                    value: selected_dropdown.read().clone(),
                    for ch in values {
                        DropdownItem {
                            value: ch.clone(),
                            onpress: {
                                to_owned![ch];
                                move |_| selected_dropdown.set(ch.clone())
                            },
                            label { "{ch}" }
                        }
                    }
                }
            )
        }

        let mut utils = launch_test(dropdown_app);
        let root = utils.root();
        let label = root.get(0).get(0).get(0);
        utils.wait_for_update().await;

        // Currently closed
        let start_size = utils.sdom().get().layout().size();

        // Default value
        assert_eq!(label.get(0).text(), Some("Value A"));

        // Open the dropdown
        utils.click_cursor((15., 15.)).await;
        utils.wait_for_update().await;

        // Now that the dropwdown is opened, there are more nodes in the layout
        assert!(utils.sdom().get().layout().size() > start_size);

        // Close the dropdown by clicking outside of it
        utils.click_cursor((200., 200.)).await;

        // Now the layout size is like in the begining
        assert_eq!(utils.sdom().get().layout().size(), start_size);

        // Open the dropdown again
        utils.click_cursor((15., 15.)).await;

        // Click on the second option
        utils.click_cursor((45., 90.)).await;
        utils.wait_for_update().await;
        utils.wait_for_update().await;

        // Now the layout size is like in the begining, again
        assert_eq!(utils.sdom().get().layout().size(), start_size);

        // The second optio was selected
        assert_eq!(label.get(0).text(), Some("Value B"));
    }

    #[tokio::test]
    pub async fn dropdown_keyboard_navigation() {
        fn dropdown_keyboard_navigation_app() -> Element {
            let values = use_hook(|| {
                vec![
                    "Value A".to_string(),
                    "Value B".to_string(),
                    "Value C".to_string(),
                ]
            });
            let mut selected_dropdown = use_signal(|| "Value A".to_string());

            rsx!(
                Dropdown {
                    value: selected_dropdown.read().clone(),
                    for ch in values {
                        DropdownItem {
                            value: ch.clone(),
                            onpress: {
                                to_owned![ch];
                                move |_| selected_dropdown.set(ch.clone())
                            },
                            label { "{ch}" }
                        }
                    }
                }
            )
        }

        let mut utils = launch_test(dropdown_keyboard_navigation_app);
        let root = utils.root();
        let label = root.get(0).get(0).get(0);
        utils.wait_for_update().await;

        // Currently closed
        let start_size = utils.sdom().get().layout().size();

        // Default value
        assert_eq!(label.get(0).text(), Some("Value A"));

        // Open the dropdown
        utils.push_event(TestEvent::Keyboard {
            name: EventName::KeyDown,
            key: Key::Tab,
            code: Code::Tab,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;
        utils.wait_for_update().await;
        utils.push_event(TestEvent::Keyboard {
            name: EventName::KeyDown,
            key: Key::Enter,
            code: Code::Enter,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;
        utils.wait_for_update().await;
        utils.wait_for_update().await;

        // Now that the dropwdown is opened, there are more nodes in the layout
        assert!(utils.sdom().get().layout().size() > start_size);

        // Close the dropdown by pressinc Esc
        utils.push_event(TestEvent::Keyboard {
            name: EventName::KeyDown,
            key: Key::Escape,
            code: Code::Escape,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;

        // Now the layout size is like in the begining
        assert_eq!(utils.sdom().get().layout().size(), start_size);

        // Open the dropdown again
        utils.push_event(TestEvent::Keyboard {
            name: EventName::KeyDown,
            key: Key::Enter,
            code: Code::Enter,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;

        // Click on the second option
        utils.push_event(TestEvent::Keyboard {
            name: EventName::KeyDown,
            key: Key::Tab,
            code: Code::Tab,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;
        utils.wait_for_update().await;
        utils.push_event(TestEvent::Keyboard {
            name: EventName::KeyDown,
            key: Key::Tab,
            code: Code::Tab,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;
        utils.wait_for_update().await;
        utils.push_event(TestEvent::Keyboard {
            name: EventName::KeyDown,
            key: Key::Enter,
            code: Code::Enter,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;
        utils.wait_for_update().await;

        // Close with Escape
        utils.push_event(TestEvent::Keyboard {
            name: EventName::KeyDown,
            key: Key::Escape,
            code: Code::Escape,
            modifiers: Modifiers::default(),
        });
        utils.wait_for_update().await;
        utils.wait_for_update().await;

        // Now the layout size is like in the begining, again
        assert_eq!(utils.sdom().get().layout().size(), start_size);

        // The second option was selected
        assert_eq!(label.get(0).text(), Some("Value B"));
    }
}
