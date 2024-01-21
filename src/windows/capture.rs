use image::RgbaImage;
use std::mem;
use windows::Win32::{
    Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, GetDIBits, SelectObject, BITMAPINFO,
        BITMAPINFOHEADER, DIB_RGB_COLORS, RGBQUAD, SRCCOPY,
    },
    Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS, PW_CLIENTONLY},
    UI::WindowsAndMessaging::PW_RENDERFULLCONTENT,
};

use crate::{
    error::{XCapError, XCapResult},
    utils::image::bgra_to_rgba_image,
};

use super::{
    boxed::{BoxHBITMAP, BoxHDC},
    impl_monitor::ImplMonitor,
    impl_window::ImplWindow,
};

fn to_rgba_image(
    box_hdc_mem: BoxHDC,
    box_h_bitmap: BoxHBITMAP,
    width: i32,
    height: i32,
) -> XCapResult<RgbaImage> {
    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD::default(); 1],
    };

    let data = vec![0u8; (width * height) as usize * 4];
    let buf_prt = data.as_ptr() as *mut _;

    unsafe {
        // 读取数据到 buffer 中
        let is_success = GetDIBits(
            *box_hdc_mem,
            *box_h_bitmap,
            0,
            height as u32,
            Some(buf_prt),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        ) == 0;

        if is_success {
            return Err(XCapError::new("Get RGBA data failed"));
        }
    };

    bgra_to_rgba_image(width as u32, height as u32, data)
}

#[allow(unused)]
pub fn capture_monitor(
    impl_monitor: &ImplMonitor,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> XCapResult<RgbaImage> {
    unsafe {
        let box_hdc_monitor = BoxHDC::from(impl_monitor);
        // 内存中的HDC
        let box_hdc_mem = BoxHDC::new(CreateCompatibleDC(*box_hdc_monitor));
        let box_h_bitmap = BoxHBITMAP::new(CreateCompatibleBitmap(*box_hdc_monitor, width, height));

        // 使用SelectObject函数将这个位图选择到DC中
        SelectObject(*box_hdc_mem, *box_h_bitmap);

        // 拷贝原始图像到内存
        // 咋合理不需要i缩放图片，所以直接使用BitBlt
        // 如需要缩放，则使用 StretchBlt
        BitBlt(
            *box_hdc_mem,
            0,
            0,
            width,
            height,
            *box_hdc_monitor,
            x,
            y,
            SRCCOPY,
        )?;

        to_rgba_image(box_hdc_mem, box_h_bitmap, width, height)
    }
}

#[allow(unused)]
pub fn capture_window(impl_window: &ImplWindow, width: i32, height: i32) -> XCapResult<RgbaImage> {
    unsafe {
        let box_hdc_window: BoxHDC = BoxHDC::from(impl_window);
        // 内存中的HDC
        let box_hdc_mem = BoxHDC::new(CreateCompatibleDC(*box_hdc_window));
        let box_h_bitmap = BoxHBITMAP::new(CreateCompatibleBitmap(*box_hdc_window, width, height));

        // 使用SelectObject函数将这个位图选择到DC中
        SelectObject(*box_hdc_mem, *box_h_bitmap);

        // Grab a copy of the window. Use PrintWindow because it works even when the
        // window's partially occluded. The PW_RENDERFULLCONTENT flag is undocumented,
        // but works starting in Windows 8.1. It allows for capturing the contents of
        // the window that are drawn using DirectComposition.
        // https://github.com/chromium/chromium/blob/main/ui/snapshot/snapshot_win.cc#L39-L45
        let flags = PW_CLIENTONLY.0 | PW_RENDERFULLCONTENT;
        PrintWindow(impl_window.hwnd, *box_hdc_mem, PRINT_WINDOW_FLAGS(flags));

        to_rgba_image(box_hdc_mem, box_h_bitmap, width, height)
    }
}