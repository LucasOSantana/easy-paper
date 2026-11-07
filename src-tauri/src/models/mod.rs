pub mod config;
pub mod stock;
pub mod update;
pub mod wallhaven;
pub mod wallpaper_item;

pub use config::{
    AppConfig, AutoRotateConfig, RotationOrder, RotationTrigger, MAX_ROTATION_INTERVAL_MINUTES,
    MIN_ROTATION_INTERVAL_MINUTES,
};
pub use stock::{StockPage, StockPhoto, StockProvider, StockSearch};
pub use update::UpdateInfo;
pub use wallhaven::{WallhavenPage, WallhavenSearch};
pub use wallpaper_item::WallpaperImageItem;
