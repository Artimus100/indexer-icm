// Re-export all handlers from their respective modules
pub mod coins;
pub mod holders;
pub mod klines;
pub mod trades;

// Re-export specific handlers for easy access
pub use coins::*;
pub use holders::*;
pub use klines::*;
pub use trades::*;

use poem::handler;

#[handler]
pub fn health_check() -> &'static str {
    "OK"
}
