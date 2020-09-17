#[cfg(windows)]
pub mod direct_d3d9;

#[derive(Debug)]
pub enum InstallError {
    AlreadyInstalled,
    Other,
}

#[derive(Debug)]
pub enum UninstallError {
    NotInstalled,
    Other,
}

pub trait Hook {
    fn is_installed(&self) -> bool;
    fn install(&mut self) -> Result<(), InstallError>;
    fn uninstall(&mut self) -> Result<(), UninstallError>;
}
