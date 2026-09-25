use ffmpeg_next as ffmpeg;
use ffmpeg::{codec, packet::Mut};

#[repr(C)]
struct RawBsfCtx { _private: [u8; 0] }

extern "C" {
    fn crop_bsf_create(
        par_in: *mut ffmpeg::ffi::AVCodecParameters,
        crop_right_units: i32,
        crop_bottom_units: i32,
    ) -> *mut RawBsfCtx;
    fn crop_bsf_filter(ctx: *mut RawBsfCtx, pkt: *mut ffmpeg::ffi::AVPacket) -> i32;
    fn crop_bsf_free(ctx: *mut RawBsfCtx);
}

pub struct H264CropBsf {
    ctx: *mut RawBsfCtx,
}

impl H264CropBsf {
    pub fn new(codecpar: &codec::parameters::Parameters, cropRight: u32, cropBottom: u32) -> Result<Self, String> {
        unsafe {
            let ctx = crop_bsf_create(
                codecpar.as_ptr() as *mut _,
                cropRight as i32,
                cropBottom as i32,
            );
            if ctx.is_null() {
                return Err("crop_bsf_create failed".into());
            }
            Ok(H264CropBsf { ctx })
        }
    }

    pub fn filter(&mut self, packet: &mut ffmpeg::Packet) -> Result<(), String> {
        unsafe {
            let ret = crop_bsf_filter(self.ctx, packet.as_mut_ptr());
            if ret < 0 {
                return Err(format!("crop_bsf_filter failed: {}", ret));
            }
        }
        Ok(())
    }
}

impl Drop for H264CropBsf {
    fn drop(&mut self) {
        unsafe { crop_bsf_free(self.ctx); }
    }
}