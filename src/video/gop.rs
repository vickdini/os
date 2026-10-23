use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use x86_64::structures::paging::{Page, PageTableFlags, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

use crate::memory;
use super::console::{VideoConsole, VIDEO_CONSOLE};
use super::framebuffer::Framebuffer;

#[derive(Debug, Clone, Copy)]
pub struct VideoModeInfo {
    pub base_ptr: *mut u8,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub is_bgr: bool,
    pub text_cols: usize,
    pub text_rows: usize,
}

/// Discovers the hardware Graphics Output Protocol (GOP), extracts the linear framebuffer,
/// maps the physical video memory into Hydra's autonomous kernel PML4 table, and initializes
/// the autonomous video console.
pub fn init_gop() -> Result<VideoModeInfo, &'static str> {
    let handle = uefi::boot::get_handle_for_protocol::<GraphicsOutput>()
        .map_err(|_| "Failed to find GraphicsOutput protocol handle")?;

    let params = uefi::boot::OpenProtocolParams {
        handle,
        agent: uefi::boot::image_handle(),
        controller: None,
    };
    let mut gop = unsafe {
        uefi::boot::open_protocol::<GraphicsOutput>(
            params,
            uefi::boot::OpenProtocolAttributes::GetProtocol,
        )
        .map_err(|_| "Failed to open GraphicsOutput protocol")?
    };

    let mode_info = gop.current_mode_info();
    let (width, height) = mode_info.resolution();
    let stride = mode_info.stride();
    let pixel_format = mode_info.pixel_format();

    let is_bgr = match pixel_format {
        PixelFormat::Bgr => true,
        PixelFormat::Rgb => false,
        _ => false,
    };

    let mut fb = gop.frame_buffer();
    let base_ptr = fb.as_mut_ptr();
    let size = fb.size();

    let text_cols = width / super::font::FONT_WIDTH;
    let text_rows = height / super::font::FONT_HEIGHT;

    let info = VideoModeInfo {
        base_ptr,
        size,
        width,
        height,
        stride,
        is_bgr,
        text_cols,
        text_rows,
    };

    // Map the hardware framebuffer into Hydra's autonomous kernel page table
    let page_count = (size + 4095) / 4096;
    let start_phys = PhysAddr::new(base_ptr as u64);
    let start_virt = VirtAddr::new(base_ptr as u64);

    let start_page = Page::containing_address(start_virt);
    let start_frame = PhysFrame::containing_address(start_phys);

    {
        let mut guard = memory::paging::KERNEL_SPACE.lock();
        if let Some(ref mut space) = *guard {
            let mut alloc = memory::frame_allocator::FRAME_ALLOCATOR.lock();
            let _ = unsafe {
                space.map_range(
                    start_page,
                    start_frame,
                    page_count,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    &mut alloc,
                )
            };
        }
    }

    // Initialize the autonomous framebuffer and video console
    let framebuffer = Framebuffer::new(info);
    let console = VideoConsole::new(framebuffer);
    *VIDEO_CONSOLE.lock() = Some(console);

    Ok(info)
}

/// Returns the detected video mode information if GOP is active.
pub fn get_video_info() -> Option<VideoModeInfo> {
    let guard = VIDEO_CONSOLE.lock();
    guard.as_ref().map(|console| {
        let fb = console.framebuffer();
        VideoModeInfo {
            base_ptr: fb.base_ptr(),
            size: fb.size(),
            width: fb.width(),
            height: fb.height(),
            stride: fb.stride(),
            is_bgr: fb.is_bgr(),
            text_cols: console.cols(),
            text_rows: console.rows(),
        }
    })
}
