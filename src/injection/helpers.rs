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
        let title = self.title.as_ref().ok_or(ShowAlertDialogError::TitleNotSet)?;
        let message = self.message.as_ref().ok_or(ShowAlertDialogError::MessageNotSet)?;
        #[cfg(windows)]
        windows_show_alert_dialog(title.as_ref(), message.as_ref()).expect("windows_show_alert_dialog failed");
        #[cfg(not(windows))]
        compile_error!("No show implementation for target_arch");

        Ok(())
    }
}

#[cfg(windows)]
pub fn windows_show_alert_dialog(title: &str, message: &str) -> Result<(), Box<dyn Error>> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use winapi::um::winuser::{MB_OK, MessageBoxW};

    let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(once(0)).collect();
    let message_wide: Vec<u16> = OsStr::new(message).encode_wide().chain(once(0)).collect();
    let ret = unsafe {
        MessageBoxW(null_mut(), title_wide.as_ptr(), message_wide.as_ptr(), MB_OK)
    };
    if ret == 0 { return Err(std::io::Error::last_os_error().into()); }

    Ok(())
}

