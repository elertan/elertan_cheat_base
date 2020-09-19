use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;

#[cfg(windows)]
pub mod d3d9;

#[derive(Debug, thiserror::Error)]
pub enum InstallError<T: 'static + Debug + Display + Error> {
    #[error("Already installed")]
    AlreadyInstalled,
    #[error(transparent)]
    Other(T),
}

#[derive(Debug, thiserror::Error)]
pub enum UninstallError<T: 'static + Debug + Display + Error> {
    #[error("Not installed")]
    NotInstalled,
    #[error(transparent)]
    Other(T),
}

pub trait Hook<TInstallError: Debug + Display + Error, TUninstallError: Debug + Display + Error> {
    fn is_installed(&self) -> bool;
    unsafe fn install(&mut self) -> Result<(), InstallError<TInstallError>>;
    unsafe fn uninstall(&mut self) -> Result<(), UninstallError<TUninstallError>>;
}

pub trait Hookable {
    fn is_hookable() -> bool;
}
