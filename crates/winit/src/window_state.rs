use std::mem;

use dioxus_core::VirtualDom;
use freya_core::{
    dom::SafeDOM,
    event_loop_messages::EventLoopMessage,
};
use freya_engine::prelude::*;
use winit::{
    dpi::LogicalSize,
    event_loop::{
        ActiveEventLoop,
        EventLoopProxy,
    },
    window::{
        Window,
        WindowAttributes,
    },
};

use crate::{
    accessibility::WinitAcessibilityTree,
    app::Application,
    config::WindowConfig,
    devtools::Devtools,
    drivers::GraphicsDriver,
    size::WinitSize,
    LaunchConfig,
};

pub struct NotCreatedState<'a, State: Clone + 'static> {
    pub(crate) sdom: SafeDOM,
    pub(crate) vdom: VirtualDom,
    pub(crate) devtools: Option<Devtools>,
    pub(crate) config: LaunchConfig<'a, State>,
}

pub struct CreatedState {
    pub(crate) app: Application,
    pub(crate) surface: Surface,
    pub(crate) dirty_surface: Surface,
    pub(crate) graphics_driver: GraphicsDriver,
    pub(crate) window: Window,
    pub(crate) window_config: WindowConfig,
    pub(crate) is_window_focused: bool,
}

pub enum WindowState<'a, State: Clone + 'static> {
    NotCreated(NotCreatedState<'a, State>),
    Creating,
    Created(CreatedState),
}

impl<'a, State: Clone + 'a> WindowState<'a, State> {
    pub fn created_state(&mut self) -> &mut CreatedState {
        let Self::Created(created) = self else {
            unreachable!("Infallible, window should be created at this point.")
        };
        created
    }

    pub fn has_been_created(&self) -> bool {
        matches!(self, Self::Created(..))
    }

    pub fn create(
        &mut self,
        event_loop: &ActiveEventLoop,
        event_loop_proxy: &EventLoopProxy<EventLoopMessage>,
    ) {
        let Self::NotCreated(NotCreatedState {
            sdom,
            vdom,
            devtools,
            mut config,
        }) = mem::replace(self, WindowState::Creating)
        else {
            unreachable!("Infallible, window should not be created at this point.")
        };

        let window_attributes = Self::create_window_attributes(&mut config.window_config);

        let (graphics_driver, window, mut surface) =
            GraphicsDriver::new(event_loop, window_attributes, &config);

        let accessibility = WinitAcessibilityTree::new(&window, event_loop_proxy.clone());

        if config.window_config.visible {
            window.set_visible(true);
        }

        // Allow IME
        window.set_ime_allowed(true);

        let mut dirty_surface = surface
            .new_surface_with_dimensions(window.inner_size().to_skia())
            .unwrap();

        let scale_factor = window.scale_factor();

        surface
            .canvas()
            .scale((scale_factor as f32, scale_factor as f32));
        surface.canvas().clear(config.window_config.background);

        dirty_surface
            .canvas()
            .scale((scale_factor as f32, scale_factor as f32));
        dirty_surface
            .canvas()
            .clear(config.window_config.background);

        let mut app = Application::new(
            sdom,
            vdom,
            event_loop_proxy,
            devtools,
            &window,
            config.embedded_fonts,
            config.plugins,
            config.default_fonts,
            accessibility,
        );

        app.init_doms(scale_factor as f32, config.state);
        app.process_layout(window.inner_size(), scale_factor);

        *self = WindowState::Created(CreatedState {
            surface,
            dirty_surface,
            graphics_driver,
            window,
            app,
            window_config: config.window_config,
            is_window_focused: false,
        });
    }

    pub fn resume(&mut self, event_loop: &ActiveEventLoop) {
        let created = self.created_state();
        let window_attributes = Self::create_window_attributes(&mut created.window_config);
        let mut config: LaunchConfig<State> = LaunchConfig::default();
        config.window_config.transparent = created.window_config.transparent;

        let (graphics_driver, window, surface) =
            GraphicsDriver::new(event_loop, window_attributes, &config);

        created.window = window;
        created.surface = surface;
        created.graphics_driver = graphics_driver;
    }

    fn create_window_attributes(window_config: &mut WindowConfig) -> WindowAttributes {
        let mut window_attributes = Window::default_attributes()
            .with_visible(false)
            .with_title(window_config.title)
            .with_decorations(window_config.decorations)
            .with_transparent(window_config.transparent)
            .with_window_icon(window_config.icon.take())
            .with_inner_size(LogicalSize::<f64>::from(window_config.size));

        set_resource_cache_total_bytes_limit(1000000); // 1MB
        set_resource_cache_single_allocation_byte_limit(Some(500000)); // 0.5MB

        if let Some(min_size) = window_config.min_size {
            window_attributes =
                window_attributes.with_min_inner_size(LogicalSize::<f64>::from(min_size));
        }
        if let Some(max_size) = window_config.max_size {
            window_attributes =
                window_attributes.with_max_inner_size(LogicalSize::<f64>::from(max_size));
        }

        if let Some(with_window_attributes) = window_config.window_attributes_hook.take() {
            window_attributes = (with_window_attributes)(window_attributes);
        }

        window_attributes
    }
}
