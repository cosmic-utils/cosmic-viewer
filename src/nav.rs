//! Navigation state and directory scanning

use std::path::{Path, PathBuf};

/// Supported image file extensions
pub const EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tiff", "tif", "ico", "avif", "raw", "cr2", "cr3",
    "nef", "arw", "dng", "orf", "rw2",
];

/// Navigation state for browsing images in a directory
#[derive(Debug, Clone, Default)]
pub struct NavState {
    images: Vec<PathBuf>,
    cur_idx: usize,
    cur_dir: Option<PathBuf>,
}

impl NavState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current image path, if any
    pub fn current(&self) -> Option<&PathBuf> {
        self.images.get(self.cur_idx)
    }

    /// Get current index
    pub fn index(&self) -> usize {
        self.cur_idx
    }

    /// Get total number of images
    pub fn total(&self) -> usize {
        self.images.len()
    }

    /// Check if nav is empty (no images loaded)
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// Set images from a directory scan, optionally preserving selection
    pub fn set_images(&mut self, images: Vec<PathBuf>, select: Option<&Path>) {
        self.images = images;
        if let Some(path) = select {
            self.cur_idx = self.images.iter().position(|pos| pos == path).unwrap_or(0);
        } else {
            self.cur_idx = 0;
        }
    }

    /// Nav to next image, wrapping around
    pub fn next(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = (self.cur_idx + 1) % self.images.len();
        self.current()
    }

    /// Nav to prev image, wrapping around
    pub fn prev(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = if self.cur_idx == 0 {
            self.images.len() - 1
        } else {
            self.cur_idx - 1
        };

        self.current()
    }

    /// Jump to first image
    pub fn first(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = 0;
        self.current()
    }

    /// Jump to last image
    pub fn last(&mut self) -> Option<&PathBuf> {
        if self.images.is_empty() {
            return None;
        }

        self.cur_idx = self.images.len() - 1;
        self.current()
    }

    /// Jump to specific index
    pub fn go_to(&mut self, idx: usize) -> Option<&PathBuf> {
        if idx < self.images.len() {
            self.cur_idx = idx;
            self.current()
        } else {
            None
        }
    }
}
