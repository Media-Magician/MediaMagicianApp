#[cfg(target_os = "linux")]
mod linux;

#[cfg(all(not(target_os = "linux"), not(target_os = "android")))]
mod desktop;

#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(all(not(target_os = "linux"), not(target_os = "android")))]
pub use desktop::*;
#[cfg(target_os = "android")]
pub use android::*;
