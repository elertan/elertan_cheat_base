use elertan_cheat_base::in_process::entrypoint::{AttachError, DetachError, Entrypoint};
use elertan_cheat_base::in_process::helpers::AlertDialog;
use elertan_cheat_base::once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum AppAttachError {
    #[error("Other")]
    Other,
}

#[derive(Debug, thiserror::Error)]
pub enum AppDetachError {
    #[error("Other")]
    Other,
}

struct App {}

impl Entrypoint<AppAttachError, AppDetachError> for App {
    fn attach(&mut self) -> Result<(), AttachError<AppAttachError>> {
        AlertDialog::new()
            .title("ElertanCheatBase")
            .message("Hello, Hook!")
            .show()
            .map_err(|_| AttachError::Custom(AppAttachError::Other))?;

        Ok(())
    }

    fn detach(&mut self) -> Result<(), DetachError<AppDetachError>> {
        Ok(())
    }
}

static APP: Lazy<Mutex<App>> = Lazy::new(|| Mutex::new(App {}));

elertan_cheat_base::generate_entrypoint!(APP);
