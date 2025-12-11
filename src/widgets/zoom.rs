//! Zoomable and pannable container widget

use cosmic::{
    Element,
    iced::Length,
    widget::{container, scrollable},
};

#[derive(Debug, Clone)]
pub struct ZoomableContainer {
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub offset: (f32, f32),
}

impl Default for ZoomableContainer {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            min_zoom: 0.1,
            max_zoom: 10.0,
            offset: (0.0, 0.0),
        }
    }
}

impl ZoomableContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom.clamp(self.min_zoom, self.max_zoom);
        self
    }

    pub fn zoom_in(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(self.min_zoom, self.max_zoom);
    }

    pub fn zoom_out(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(self.min_zoom, self.max_zoom);
    }

    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.offset = (0.0, 0.0);
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.offset.0 += dx;
        self.offset.1 += dy;
    }

    pub fn wrap<'a, Message: 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let inner = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill);

        scrollable(inner)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
