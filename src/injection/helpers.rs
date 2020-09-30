use std::error::Error;

#[derive(Debug)]
pub enum ShowAlertDialogError {
    TitleNotSet,
    MessageNotSet,
}

#[derive(Debug, Default, Clone)]
pub struct AlertDialog<T: AsRef<str>, M: AsRef<str>> {
    title: Option<T>,
    message: Option<M>,
}

impl<T: AsRef<str>, M: AsRef<str>> AlertDialog<T, M> {
    pub fn new() -> Self {
        Self {
            title: None,
            message: None,
        }
    }

    pub fn title(self, title: T) -> Self {
        Self {
            title: Some(title),
            ..self
        }
    }

    pub fn message(self, message: M) -> Self {
        Self {
            message: Some(message),
            ..self
        }
    }

    pub fn show(&self) -> Result<(), ShowAlertDialogError> {
        let title = self
            .title
            .as_ref()
            .ok_or(ShowAlertDialogError::TitleNotSet)?;
        let message = self
            .message
            .as_ref()
            .ok_or(ShowAlertDialogError::MessageNotSet)?;
        #[cfg(windows)]
        windows_show_alert_dialog(title.as_ref(), message.as_ref())
            .expect("windows_show_alert_dialog failed");
        #[cfg(target_os = "macos")]
        macos_show_alert_dialog(title.as_ref(), message.as_ref())
            .expect("macos_show_alert_dialog failed");
        #[cfg(target_os = "linux")]
        compile_error!("No show implementation for linux");

        Ok(())
    }
}

#[cfg(windows)]
pub fn windows_show_alert_dialog(title: &str, message: &str) -> Result<(), Box<dyn Error>> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use winapi::um::winuser::{MessageBoxW, MB_OK};

    let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(once(0)).collect();
    let message_wide: Vec<u16> = OsStr::new(message).encode_wide().chain(once(0)).collect();
    let ret = unsafe {
        MessageBoxW(
            null_mut(),
            message_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_OK,
        )
    };
    if ret == 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn macos_show_alert_dialog(title: &str, message: &str) -> Result<(), Box<dyn Error>> {
    // use coreaudio_sys::audio_unit::CFUserNotificationDisplayAlert;
    //
    // unsafe { CFUserNotificationDisplayAlert() };
    Ok(())
}
