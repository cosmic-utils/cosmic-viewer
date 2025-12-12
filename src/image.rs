pub mod cache;
pub mod loader;

pub use cache::{CachedImage, ImageCache};
pub use loader::{LoadedImage, load_image};

/// Register format hooks for optional codecs (HEIC, ect.)
pub fn register_format_hooks() {
    #[cfg(feature = "heif")]
    {
        // libheif-rs automatically registers decoders when imported
        tracing::info!("HEIF/HEIC support enabled");
    }
}
