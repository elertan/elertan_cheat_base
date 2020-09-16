pub mod direct_3d9;

pub trait Hook {
    fn install();
    fn uninstall();
}
