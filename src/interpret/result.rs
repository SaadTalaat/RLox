use super::error::RuntimeCtrl;

pub type Result<'a, T> = std::result::Result<T, RuntimeCtrl<'a>>;
