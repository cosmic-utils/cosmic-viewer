use crate::{
    message::{EditMessage, Message},
    widgets::{CropSelection, DragHandle},
};
use cosmic::{
    Element, Renderer,
    iced::{
        Color, Length, Point, Rectangle, Size,
        advanced::{
            Clipboard, Layout, Shell, Widget,
            layout::{Limits, Node},
            renderer::{Quad, Renderer as QuadRenderer},
            widget::Tree,
        },
        event::{Event, Status},
        mouse::{self, Button, Cursor},
    },
};

const HANDLE_SIZE: f32 = 14.0;
const HANDLE_HIT_SIZE: f32 = 28.0;
const OVERLAY_COLOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.5);
const HANDLE_COLOR: Color = Color::WHITE;
const BORDER_COLOR: Color = Color::WHITE;

/// A widget that draws crop selection UI (overlay, handles, border) on top of an image.
/// This widget does NOT draw the image - use it in a stack on top of an image widget.
pub struct CropOverlay {
    img_width: u32,
    img_height: u32,
    /// Cloned selection data to ensure it's fresh
    selection: CropSelection,
}

impl CropOverlay {
    pub fn new(img_width: u32, img_height: u32, selection: &CropSelection) -> Self {
        Self {
            img_width,
            img_height,
            selection: selection.clone(),
        }
    }

    /// Get the scale factor from the actual bounds
    fn get_scale(&self, bounds: &Rectangle) -> f32 {
        // Use width-based scale, should match height-based for proper aspect ratio
        bounds.width / self.img_width as f32
    }

    /// Convert screen coordinates to image coordinates
    fn screen_to_image(&self, bounds: &Rectangle, point: Point) -> (f32, f32) {
        let scale = self.get_scale(bounds);
        let x = ((point.x - bounds.x) / scale)
            .max(0.0)
            .min(self.img_width as f32);
        let y = ((point.y - bounds.y) / scale)
            .max(0.0)
            .min(self.img_height as f32);

        (x, y)
    }

    /// Convert image coordinates to screen coordinates
    fn image_to_screen(&self, bounds: &Rectangle, img_x: f32, img_y: f32) -> Point {
        let scale = self.get_scale(bounds);
        Point::new(bounds.x + img_x * scale, bounds.y + img_y * scale)
    }

    /// Check which handle (if any) is at the given screen position
    fn hit_test_handle(&self, bounds: &Rectangle, point: Point) -> DragHandle {
        let Some((rx, ry, rw, rh)) = self.selection.region else {
            return DragHandle::None;
        };

        let top_left = self.image_to_screen(bounds, rx, ry);
        let top_right = self.image_to_screen(bounds, rx + rw, ry);
        let bottom_left = self.image_to_screen(bounds, rx, ry + rh);
        let bottom_right = self.image_to_screen(bounds, rx + rw, ry + rh);

        // Corner handles (higher priority)
        if self.point_in_handle(point, top_left) {
            return DragHandle::TopLeft;
        }
        if self.point_in_handle(point, top_right) {
            return DragHandle::TopRight;
        }
        if self.point_in_handle(point, bottom_left) {
            return DragHandle::BottomLeft;
        }
        if self.point_in_handle(point, bottom_right) {
            return DragHandle::BottomRight;
        }

        // Edge handles
        let mid_top = self.image_to_screen(bounds, rx + rw / 2.0, ry);
        let mid_bottom = self.image_to_screen(bounds, rx + rw / 2.0, ry + rh);
        let mid_left = self.image_to_screen(bounds, rx, ry + rh / 2.0);
        let mid_right = self.image_to_screen(bounds, rx + rw, ry + rh / 2.0);

        if self.point_in_handle(point, mid_top) {
            return DragHandle::Top;
        }
        if self.point_in_handle(point, mid_bottom) {
            return DragHandle::Bottom;
        }
        if self.point_in_handle(point, mid_left) {
            return DragHandle::Left;
        }
        if self.point_in_handle(point, mid_right) {
            return DragHandle::Right;
        }

        // Check if inside selection (for move)
        let selection_rect = Rectangle::new(
            top_left,
            Size::new(bottom_right.x - top_left.x, bottom_right.y - top_left.y),
        );

        if selection_rect.contains(point) {
            return DragHandle::Move;
        }

        DragHandle::None
    }

    fn point_in_handle(&self, point: Point, handle_center: Point) -> bool {
        let half = HANDLE_HIT_SIZE / 2.0;
        point.x >= handle_center.x - half
            && point.x <= handle_center.x + half
            && point.y >= handle_center.y - half
            && point.y <= handle_center.y + half
    }

    /// Get the cursor style based on position
    fn cursor_for_handle(&self, handle: DragHandle) -> mouse::Interaction {
        match handle {
            DragHandle::None => mouse::Interaction::Crosshair,
            DragHandle::TopLeft | DragHandle::BottomRight => {
                mouse::Interaction::ResizingDiagonallyDown
            }
            DragHandle::TopRight | DragHandle::BottomLeft => {
                mouse::Interaction::ResizingDiagonallyUp
            }
            DragHandle::Top | DragHandle::Bottom => mouse::Interaction::ResizingVertically,
            DragHandle::Left | DragHandle::Right => mouse::Interaction::ResizingHorizontally,
            DragHandle::Move => mouse::Interaction::Grabbing,
        }
    }
}

impl Widget<Message, cosmic::Theme, Renderer> for CropOverlay {
    fn size(&self) -> Size<Length> {
        // Fill parent to inherit correct position from stack
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        // Fill parent to inherit correct position from stack
        Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &cosmic::Theme,
        _style: &cosmic::iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let scale = self.get_scale(&bounds);

        // Draw dark overlay regions (around selection if any, or full image if none)
        if let Some((rx, ry, rw, rh)) = self.selection.region {
            if rw > 0.0 && rh > 0.0 {
                // Selection rectangle in screen coords
                let sel_x = bounds.x + rx * scale;
                let sel_y = bounds.y + ry * scale;
                let sel_w = rw * scale;
                let sel_h = rh * scale;

                // Draw dark overlay only outside the selection
                // Top region
                if sel_y > bounds.y {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                bounds.position(),
                                Size::new(bounds.width, sel_y - bounds.y),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }
                // Bottom region
                let sel_bottom = sel_y + sel_h;
                let img_bottom = bounds.y + bounds.height;
                if sel_bottom < img_bottom {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(bounds.x, sel_bottom),
                                Size::new(bounds.width, img_bottom - sel_bottom),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }
                // Left region (between top and bottom)
                if sel_x > bounds.x {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(bounds.x, sel_y),
                                Size::new(sel_x - bounds.x, sel_h),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }
                // Right region (between top and bottom)
                let sel_right = sel_x + sel_w;
                let img_right = bounds.x + bounds.width;
                if sel_right < img_right {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(sel_right, sel_y),
                                Size::new(img_right - sel_right, sel_h),
                            ),
                            ..Quad::default()
                        },
                        OVERLAY_COLOR,
                    );
                }

                // Draw selection border
                let border_width = 2.0;
                // Top border
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(sel_x, sel_y),
                            Size::new(sel_w, border_width),
                        ),
                        ..Quad::default()
                    },
                    BORDER_COLOR,
                );
                // Bottom border
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(sel_x, sel_y + sel_h - border_width),
                            Size::new(sel_w, border_width),
                        ),
                        ..Quad::default()
                    },
                    BORDER_COLOR,
                );
                // Left border
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(sel_x, sel_y),
                            Size::new(border_width, sel_h),
                        ),
                        ..Quad::default()
                    },
                    BORDER_COLOR,
                );
                // Right border
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(sel_x + sel_w - border_width, sel_y),
                            Size::new(border_width, sel_h),
                        ),
                        ..Quad::default()
                    },
                    BORDER_COLOR,
                );

                // Draw resize handles on top
                let handle_half = HANDLE_SIZE / 2.0;
                let handles = [
                    (sel_x, sel_y),                       // TopLeft
                    (sel_x + sel_w, sel_y),               // TopRight
                    (sel_x, sel_y + sel_h),               // BottomLeft
                    (sel_x + sel_w, sel_y + sel_h),       // BottomRight
                    (sel_x + sel_w / 2.0, sel_y),         // Top
                    (sel_x + sel_w / 2.0, sel_y + sel_h), // Bottom
                    (sel_x, sel_y + sel_h / 2.0),         // Left
                    (sel_x + sel_w, sel_y + sel_h / 2.0), // Right
                ];

                for (hx, hy) in handles {
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(hx - handle_half, hy - handle_half),
                                Size::new(HANDLE_SIZE, HANDLE_SIZE),
                            ),
                            ..Quad::default()
                        },
                        HANDLE_COLOR,
                    );
                }
            } else {
                // Selection exists but has zero size - draw full overlay
                renderer.fill_quad(
                    Quad {
                        bounds: bounds,
                        ..Quad::default()
                    },
                    OVERLAY_COLOR,
                );
            }
        } else {
            // No selection - draw full overlay
            renderer.fill_quad(
                Quad {
                    bounds: bounds,
                    ..Quad::default()
                },
                OVERLAY_COLOR,
            );
        }
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    // First check for handle hits (handles can extend slightly outside image)
                    let handle = self.hit_test_handle(&bounds, pos);

                    if handle != DragHandle::None {
                        // Clicking on a handle - start resize/move
                        let (img_x, img_y) = self.screen_to_image(&bounds, pos);
                        shell.publish(Message::Edit(EditMessage::CropDragStart {
                            x: img_x,
                            y: img_y,
                            handle,
                        }));
                        return Status::Captured;
                    } else if bounds.contains(pos) {
                        // Clicking on image (not on handle) - start new selection
                        let (img_x, img_y) = self.screen_to_image(&bounds, pos);
                        shell.publish(Message::Edit(EditMessage::CropDragStart {
                            x: img_x,
                            y: img_y,
                            handle: DragHandle::None,
                        }));
                        return Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.selection.is_dragging {
                    if let Some(pos) = cursor.position() {
                        let (img_x, img_y) = self.screen_to_image(&bounds, pos);
                        shell.publish(Message::Edit(EditMessage::CropDragMove {
                            x: img_x,
                            y: img_y,
                        }));
                        return Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => {
                if self.selection.is_dragging {
                    shell.publish(Message::Edit(EditMessage::CropDragEnd));
                    return Status::Captured;
                }
            }
            _ => {}
        }

        Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();

        if self.selection.is_dragging {
            return self.cursor_for_handle(self.selection.drag_handle);
        }

        if let Some(pos) = cursor.position_in(bounds) {
            // Check handles first (they can extend slightly outside image)
            let handle = self.hit_test_handle(&bounds, pos);
            if handle != DragHandle::None {
                return self.cursor_for_handle(handle);
            }
            // Inside image but not on handle - show crosshair
            if bounds.contains(pos) {
                return mouse::Interaction::Crosshair;
            }
        }

        mouse::Interaction::default()
    }
}

impl<'a> From<CropOverlay> for Element<'a, Message> {
    fn from(overlay: CropOverlay) -> Self {
        Self::new(overlay)
    }
}

/// Helper function to create a CropOverlay (no image - use in stack with image widget)
pub fn crop_overlay(img_width: u32, img_height: u32, selection: &CropSelection) -> CropOverlay {
    CropOverlay::new(img_width, img_height, selection)
}
