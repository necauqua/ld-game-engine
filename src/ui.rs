use std::{
    rc::Rc,
    borrow::Cow,
    fmt::{Debug, Formatter},
};

use crate::{
    event::{Event, MouseButton},
    sound::Sound,
    surface::SurfaceContextExt,
    Context, Game,
    V2, v2,
};

pub struct Text {
    pub pos: V2,
    pub text: Cow<'static, str>,
    size: f64,
    font: String,
}

impl Debug for Text {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("pos", &self.pos)
            .field("text", &self.text)
            .field("size", &self.size)
            .finish()
    }
}

impl Text {
    pub fn empty() -> Text {
        Self::new("".into())
    }

    pub fn new(text: Cow<'static, str>) -> Text {
        Self {
            pos: v2![0.0, 0.0],
            text,
            size: 2.5,
            font: "2.5rem monospace".into(),
        }
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.set_size(size);
        self
    }

    pub fn set_size(&mut self, size: f64) {
        self.size = size;
        self.font = format!("{}rem monospace", size);
    }

    pub fn compute_size<G: Game>(&self, context: &mut Context<G>) -> (f64, f64) {
        let surface = context.surface().context();
        surface.set_font(&self.font);
        let dim = surface.measure_text(&self.text).unwrap();
        (dim.width(), context.rem_to_px(self.size))
    }

    pub fn is_over<G: Game>(&self, pos: V2, context: &mut Context<G>) -> bool {
        let (w, h) = self.compute_size(context);
        pos.x >= self.pos.x - w / 2.0
            && pos.x <= self.pos.x + w / 2.0
            && pos.y >= self.pos.y - h / 2.0
            && pos.y <= self.pos.y + h / 2.0
    }

    pub fn on_update<G: Game>(&mut self, context: &mut Context<G>, pos: V2, color: &str) {
        let surface = context.surface().context();

        self.pos = pos;

        surface.fill_color(color);
        surface.set_font(&self.font);
        surface.fill_text(&self.text, pos.x, pos.y).unwrap();
    }
}

#[derive(Debug)]
pub struct Button {
    pub text: Text,
    pub enabled: bool,
    color: &'static str,
    hover_color: &'static str,
    disabled_color: &'static str,
    click_sound: Option<Rc<Sound>>,
    hover_sound: Option<Rc<Sound>>,
    hovered: bool,
    last_touch: Option<V2>,
}

impl Button {
    pub fn empty(color: &'static str) -> Self {
        Self::new("".into(), color)
    }

    pub fn new(text: Cow<'static, str>, color: &'static str) -> Self {
        Self {
            text: Text::new(text),
            color,
            hover_color: color,
            disabled_color: color,
            click_sound: None,
            hover_sound: None,
            hovered: false,
            enabled: true,
            last_touch: None,
        }
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.text.set_size(size);
        self
    }

    pub fn with_click_sound(mut self, click_sound: Rc<Sound>) -> Self {
        self.click_sound = Some(click_sound);
        self
    }

    pub fn with_hover_sound(mut self, hover_sound: Rc<Sound>) -> Self {
        self.hover_sound = Some(hover_sound);
        self
    }

    pub fn with_hover_color(mut self, hover_color: &'static str) -> Self {
        self.hover_color = hover_color;
        self
    }

    pub fn with_disabled_color(mut self, disabled_color: &'static str) -> Self {
        self.disabled_color = disabled_color;
        self
    }

    pub fn set_text(&mut self, text: impl Into<Cow<'static, str>>) {
        self.text.text = text.into();
    }

    fn handle_press<G: Game>(&mut self, pos: V2, context: &mut Context<G>) -> bool {
        if self.text.is_over(pos, context) {
            if let Some(click_sound) = self.click_sound.as_ref() {
                click_sound.play();
            }
            true
        } else {
            false
        }
    }

    pub fn on_event<G: Game>(&mut self, event: &Event, context: &mut Context<G>) -> bool {
        if !self.enabled {
            return false;
        }
        match event {
            Event::MouseMove { pos, .. } => {
                let over = self.text.is_over(*pos, context);
                if !self.hovered && over {
                    if let Some(hover_sound) = self.hover_sound.as_ref() {
                        hover_sound.play();
                    }
                }
                self.hovered = over;
                false
            }
            Event::MouseUp {
                pos,
                button: MouseButton::Left,
            } => self.handle_press(*pos, context),
            Event::TouchStart { touches } => {
                self.last_touch = touches.get(0).copied();
                false
            }
            Event::TouchMove { touches } => {
                self.last_touch = touches.get(0).copied();
                false
            }
            Event::TouchEnd { touches } if touches.len() <= 1 => {
                self.hovered = false;
                if let Some(pos) = touches.get(0).copied().or(self.last_touch) {
                    self.handle_press(pos, context)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn on_update<G: Game>(&mut self, context: &mut Context<G>, pos: V2) {
        self.text.on_update(
            context,
            pos,
            if !self.enabled {
                self.disabled_color
            } else if self.hovered {
                self.hover_color
            } else {
                self.color
            },
        );
    }
}
