#[cfg(windows)]
pub mod d3d9;

#[derive(Debug)]
pub enum InstallError<T> {
    AlreadyInstalled,
    Custom(T),
}

#[derive(Debug)]
pub enum UninstallError<T> {
    NotInstalled,
    Custom(T),
}

pub trait Hook<TInstallError, TUninstallError> {
    fn is_installed(&self) -> bool;
    unsafe fn install(&mut self) -> Result<(), InstallError<TInstallError>>;
    unsafe fn uninstall(&mut self) -> Result<(), UninstallError<TUninstallError>>;
}
