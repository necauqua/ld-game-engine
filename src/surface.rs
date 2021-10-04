use std::f64::consts::TAU;

use js_sys::Array;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{event::Event, util::Mut, V2};

pub type SurfaceContext = CanvasRenderingContext2d;

#[derive(Clone)]
pub struct Surface {
    size: Mut<V2>,
    context: SurfaceContext,
}

fn setup_canvas(events: Mut<Vec<Event>>, size: Mut<V2>) -> CanvasRenderingContext2d {
    let canvas = super::document()
        .create_element("canvas")
        .map_err(|_| ())
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().map_err(|_| ()))
        .expect("Failed to create canvas");

    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|obj| obj.dyn_into::<CanvasRenderingContext2d>().ok())
        .expect("No canvas 2d context?");

    let moved_window = super::window();
    let moved_canvas = canvas.clone();
    let moved_context = context.clone();
    let moved_size = size; //.clone();
    let resize = move || {
        let ratio = moved_window.device_pixel_ratio();

        let width = moved_window
            .inner_width()
            .ok()
            .and_then(|js| js.as_f64())
            .unwrap();
        let height = moved_window
            .inner_height()
            .ok()
            .and_then(|js| js.as_f64())
            .unwrap();

        let scaled_width = width * ratio;
        let scaled_height = height * ratio;

        moved_canvas.set_width(scaled_width as u32);
        moved_canvas.set_height(scaled_height as u32);

        let style = format!("width: {}px; height: {}px;", width, height);
        moved_canvas.set_attribute("style", &style).unwrap();

        moved_context.set_text_align("center");
        moved_context.set_text_baseline("middle");

        *moved_size.borrow_mut() = [scaled_width, scaled_height].into();
    };
    resize();

    let on_resize = Closure::wrap(Box::new(move |_e| resize()) as Box<dyn FnMut(web_sys::Event)>);

    super::window()
        .add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())
        .unwrap();

    on_resize.forget();

    super::body()
        .append_child(&canvas)
        .expect("Failed to add canvas");

    super::event::setup_pointer_events(&canvas, &context, events.clone());
    super::event::setup_keyboard_events(&super::document(), events);

    context
}

impl Surface {
    pub fn new(events: Mut<Vec<Event>>) -> Self {
        let size = Mut::new([0.0, 0.0].into());
        let context = setup_canvas(events, size.clone());
        Self { size, context }
    }

    pub fn context(&self) -> CanvasRenderingContext2d {
        self.context.clone()
    }

    pub fn size(&self) -> V2 {
        *self.size.borrow()
    }
}

pub trait SurfaceContextExt {
    fn line_dash(&self, pattern: &[f64]);

    fn stroke_color(&self, style: &str);

    fn fill_color(&self, style: &str);

    fn line(&self, from: V2, to: V2);

    fn circle(&self, pos: V2, radius: f64);

    fn fill_circle(&self, pos: V2, radius: f64);

    fn clip_evenodd(&self);
}

impl SurfaceContextExt for SurfaceContext {
    fn line_dash(&self, pattern: &[f64]) {
        let array = Array::new_with_length(pattern.len() as u32);
        for (i, x) in pattern.iter().copied().enumerate() {
            array.set(i as u32, x.into());
        }
        self.set_line_dash(&array.into()).unwrap();
    }

    fn stroke_color(&self, color: &str) {
        self.set_stroke_style(&color.into());
    }

    fn fill_color(&self, color: &str) {
        self.set_fill_style(&color.into());
    }

    fn line(&self, from: V2, to: V2) {
        self.begin_path();
        self.move_to(from.x, from.y);
        self.line_to(to.x, to.y);
        self.stroke();
    }

    fn circle(&self, pos: V2, radius: f64) {
        self.begin_path();
        self.arc(pos.x, pos.y, radius, 0.0, TAU).unwrap();
        self.stroke();
    }

    fn fill_circle(&self, pos: V2, radius: f64) {
        self.begin_path();
        self.arc(pos.x, pos.y, radius, 0.0, TAU).unwrap();
        self.fill();
    }

    fn clip_evenodd(&self) {
        #[wasm_bindgen(inline_js = "export function clip_evenodd(s) { s.clip(\"evenodd\") }")]
        extern "C" {
            fn clip_evenodd(this: &SurfaceContext);
        }
        clip_evenodd(self);
    }
}
