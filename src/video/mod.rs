pub mod console;
pub mod font;
pub mod framebuffer;
pub mod gop;

pub use console::{clear_screen, is_active, set_colors, write_str, VideoConsole, VIDEO_CONSOLE};
pub use framebuffer::Framebuffer;
pub use gop::{get_video_info, init_gop, VideoModeInfo};
