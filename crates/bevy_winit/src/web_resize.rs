use crate::WinitWindows;
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_utils::HashMap;
use bevy_window::{FitCanvasStrategy, WindowId, Windows};
use crossbeam_channel::{Receiver, Sender};
use wasm_bindgen::JsCast;
use winit::dpi::{LogicalSize, PhysicalSize};

pub(crate) struct CanvasParentResizePlugin;

impl Plugin for CanvasParentResizePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CanvasParentResizeEventChannel>()
            .add_system(canvas_parent_resize_event_handler);
    }
}

enum ResizeEvent {
    ToParent {
        size: LogicalSize<f32>,
        window_id: WindowId,
    },
    ToSelf {
        size: LogicalSize<f32>,
        window_id: WindowId,
        selector: String,
    },
}

#[derive(Resource)]
pub(crate) struct CanvasParentResizeEventChannel {
    sender: Sender<ResizeEvent>,
    receiver: Receiver<ResizeEvent>,
}

fn canvas_parent_resize_event_handler(
    winit_windows: NonSend<WinitWindows>,
    mut bevy_windows: ResMut<Windows>,
    resize_events: Res<CanvasParentResizeEventChannel>,
    mut bevy_resize_events: ResMut<bevy_ecs::event::Events<bevy_window::WindowResized>>,
    mut scratch_windows: Local<HashMap<WindowId, ResizeEvent>>,
) {
    // Use a HashMap to only react to the latest resize event per window
    scratch_windows.extend(resize_events.receiver.try_iter().map(|event| match event {
        ResizeEvent::ToParent { size, window_id } => {
            (window_id, ResizeEvent::ToParent { size, window_id })
        }
        ResizeEvent::ToSelf {
            size,
            window_id,
            selector,
        } => (
            window_id,
            ResizeEvent::ToSelf {
                size,
                window_id,
                selector,
            },
        ),
    }));

    for (window_id, event) in scratch_windows.drain() {
        if let Some(winit_window) = winit_windows.get_window(window_id) {
            if let Some(bevy_window) = bevy_windows.get_mut(window_id) {
                match event {
                    ResizeEvent::ToParent { size, .. } => {
                        winit_window.set_inner_size(size);
                    }
                    ResizeEvent::ToSelf { size, selector, .. } => {
                        let win = web_sys::window().unwrap();
                        let doc = win.document().unwrap();

                        if let Some(canvas_elm) = doc.query_selector(&selector).ok().flatten() {
                            let PhysicalSize { width, height } =
                                size.to_physical::<u32>(winit_window.scale_factor());

                            if width != 0 && height != 0 {
                                canvas_elm
                                    .set_attribute("width", &width.to_string())
                                    .unwrap();
                                canvas_elm
                                    .set_attribute("height", &height.to_string())
                                    .unwrap();

                                // Normally the event WindowEvent::Resized in bevy_window would handle
                                // this, however since we are changing the canvas underneath winit the
                                // event wont be triggered. (Shoud we use a winit::event_loop:EventProxy
                                // instead?)
                                bevy_window.update_actual_size_from_backend(width, height);
                                // let mut resize_events = world
                                //     .resource_mut::<bevy_ecs::event::Events<bevy_window::WindowResized>>();
                                bevy_resize_events.send(bevy_window::WindowResized {
                                    id: window_id,
                                    width: width as f32,
                                    height: height as f32,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_size(element: &web_sys::Element) -> LogicalSize<f32> {
    let rect = element.get_bounding_client_rect();
    return winit::dpi::LogicalSize::new(rect.width() as f32, rect.height() as f32);
}

pub(crate) const WINIT_CANVAS_SELECTOR: &str = "canvas[data-raw-handle]";

impl Default for CanvasParentResizeEventChannel {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        return Self { sender, receiver };
    }
}

impl CanvasParentResizeEventChannel {
    pub(crate) fn listen_to_selector(
        &self,
        window_id: WindowId,
        selector: &str,
        fit_canvas_strategy: FitCanvasStrategy,
    ) {
        let win = web_sys::window().unwrap();
        let doc = win.document().unwrap();
        if let Some(canvas_elm) = doc.query_selector(selector).ok().flatten() {
            let sender = self.sender.clone();
            let owned_selector = selector.to_string();
            let resize = move || match fit_canvas_strategy {
                FitCanvasStrategy::ToParent => {
                    if let Some(size) = canvas_elm.parent_element().as_ref().map(get_size) {
                        sender
                            .send(ResizeEvent::ToParent { size, window_id })
                            .unwrap();
                    }
                }
                FitCanvasStrategy::ToSelf => {
                    let size = get_size(&canvas_elm);
                    let selector = owned_selector.clone();
                    sender
                        .send(ResizeEvent::ToSelf {
                            size,
                            window_id,
                            selector,
                        })
                        .unwrap();
                }
            };

            // ensure resize happens on startup
            resize();

            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                resize();
            }) as Box<dyn FnMut(_)>);
            let window = web_sys::window().unwrap();

            window
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
    }
}
