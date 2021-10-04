use wasm_bindgen::{*, prelude::*};
use web_sys::{DomPoint, EventTarget, MouseEvent, TouchEvent, WheelEvent};

use crate::{util::Mut, v2, V2};
use crate::surface::SurfaceContext;

pub trait ListenForever {
    fn listen_forever<E: JsCast>(&self, event_type: &str, f: impl FnMut(E) + 'static);
}

impl ListenForever for EventTarget {
    fn listen_forever<E: JsCast>(&self, event_type: &str, mut f: impl FnMut(E) + 'static) {
        let closure = Closure::wrap(Box::new(move |e: web_sys::Event| f(e.dyn_into().unwrap()))
            as Box<dyn FnMut(web_sys::Event)>);

        self.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}

pub(super) fn setup_keyboard_events(target: &EventTarget, events: Mut<Vec<Event>>) {
    fn get_meta(e: web_sys::KeyboardEvent) -> KeyMeta {
        KeyMeta {
            repeat: e.repeat(),
            alt: e.alt_key(),
            shift: e.shift_key(),
            ctrl: e.ctrl_key(),
            meta: e.meta_key(),
        }
    }

    let moved_events = events.clone();
    target.listen_forever("keydown", move |e: web_sys::KeyboardEvent| {
        moved_events.borrow_mut().push(Event::KeyDown {
            code: e.key_code(),
            key: e.key(),
            meta: get_meta(e),
        })
    });

    let moved_events = events; //.clone();
    target.listen_forever("keyup", move |e: web_sys::KeyboardEvent| {
        moved_events.borrow_mut().push(Event::KeyUp {
            code: e.key_code(),
            key: e.key(),
            meta: get_meta(e),
        })
    });
}

pub(super) fn setup_pointer_events(target: &EventTarget, context: &SurfaceContext, events: Mut<Vec<Event>>) {
    target.listen_forever("contextmenu", |e: web_sys::Event| e.prevent_default());

    fn get_pos(e: &MouseEvent, context: &SurfaceContext) -> V2 {
        #[wasm_bindgen(inline_js = "export function transform(ctx, x, y) { return new DOMPoint(x, y).matrixTransform(ctx.getTransform().inverse()) }")]
        extern "C" {
            fn transform(ctx: &SurfaceContext, x: f64, y: f64) -> DomPoint;
        }
        let ratio = super::window().device_pixel_ratio();
        let p = transform(context, e.client_x() as f64 * ratio, e.client_y() as f64 * ratio);
        v2![p.x(), p.y()]
    }

    let moved_event_queue = events.clone();
    let moved_context = context.clone();
    target.listen_forever("mouseup", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push(Event::MouseUp {
            pos: get_pos(&e, &moved_context),
            button: match MouseButton::from_code(e.button()) {
                Some(b) => b,
                _ => return,
            },
        });
    });

    let moved_event_queue = events.clone();
    let moved_context = context.clone();
    target.listen_forever("mousedown", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push(Event::MouseDown {
            pos: get_pos(&e, &moved_context),
            button: match MouseButton::from_code(e.button()) {
                Some(b) => b,
                _ => return,
            },
        });
    });

    let moved_event_queue = events.clone();
    let moved_context = context.clone();
    target.listen_forever("mousemove", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push(Event::MouseMove {
            pos: get_pos(&e, &moved_context),
            buttons: MouseButton::from_bitmap(e.buttons()),
        });
    });

    let moved_event_queue = events.clone();
    let moved_context = context.clone();
    target.listen_forever("wheel", move |e: WheelEvent| {
        moved_event_queue.borrow_mut().push(Event::MouseWheel {
            pos: get_pos(&e, &moved_context),
            delta: v2![e.delta_x(), e.delta_y()],
            buttons: MouseButton::from_bitmap(e.buttons()),
        });
    });

    fn get_touches(e: TouchEvent) -> Box<[V2]> {
        let ratio = super::window().device_pixel_ratio();
        let touch_list = e.touches();
        let mut touches = Vec::with_capacity(touch_list.length() as usize);
        while let Some(t) = touch_list.get(touches.len() as u32) {
            touches.push(v2![t.client_x() as f64 * ratio, t.client_y() as f64 * ratio]);
        }
        touches.into_boxed_slice()
    }

    let moved_event_queue = events.clone();
    target.listen_forever("touchstart", move |e: TouchEvent| {
        // prevent mouse emulation if any
        e.prevent_default();
        moved_event_queue.borrow_mut().push(Event::TouchStart {
            touches: get_touches(e),
        });
    });

    let moved_event_queue = events.clone();
    target.listen_forever("touchmove", move |e: TouchEvent| {
        e.prevent_default();
        moved_event_queue.borrow_mut().push(Event::TouchMove {
            touches: get_touches(e),
        });
    });

    let moved_event_queue = events.clone();
    target.listen_forever("touchend", move |e: TouchEvent| {
        e.prevent_default();
        moved_event_queue.borrow_mut().push(Event::TouchEnd {
            touches: get_touches(e),
        });
    });
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

impl MouseButton {
    pub fn from_code(code: i16) -> Option<MouseButton> {
        match code {
            0 => Some(MouseButton::Left),
            1 => Some(MouseButton::Middle),
            2 => Some(MouseButton::Right),
            3 => Some(MouseButton::Back),
            4 => Some(MouseButton::Forward),
            _ => None,
        }
    }

    pub fn from_bitmap(bits: u16) -> Vec<MouseButton> {
        let mut buttons = Vec::new();
        if bits & 1 != 0 {
            buttons.push(MouseButton::Left);
        }
        if bits & 2 != 0 {
            buttons.push(MouseButton::Right);
        }
        if bits & 4 != 0 {
            buttons.push(MouseButton::Middle);
        }
        if bits & 8 != 0 {
            buttons.push(MouseButton::Back);
        }
        if bits & 16 != 0 {
            buttons.push(MouseButton::Forward);
        }
        buttons
    }
}

#[derive(Debug, Clone)]
pub struct KeyMeta {
    pub repeat: bool,
    pub alt: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub meta: bool,
}

#[derive(Debug, Clone)]
pub enum Event {
    MouseDown {
        pos: V2,
        button: MouseButton,
    },
    MouseUp {
        pos: V2,
        button: MouseButton,
    },
    MouseMove {
        pos: V2,
        buttons: Vec<MouseButton>,
    },
    MouseWheel {
        pos: V2,
        buttons: Vec<MouseButton>,
        delta: V2,
    },
    TouchStart {
        touches: Box<[V2]>,
    },
    TouchMove {
        touches: Box<[V2]>,
    },
    TouchEnd {
        touches: Box<[V2]>,
    },
    KeyDown {
        code: u32,
        key: String,
        meta: KeyMeta,
    },
    KeyUp {
        code: u32,
        key: String,
        meta: KeyMeta,
    },
}

impl Event {
    pub fn is_mouse(&self) -> bool {
        matches!(self, Event::MouseDown {..} | Event::MouseUp {..} | Event::MouseMove {..} | Event::MouseWheel {..})
    }

    pub fn is_key(&self) -> bool {
        matches!(self, Event::KeyDown {..} | Event::KeyUp {..})
    }

    pub fn is_touch(&self) -> bool {
        matches!(self, Event::TouchStart {..} | Event::TouchMove {..} | Event::TouchEnd {..})
    }
}
