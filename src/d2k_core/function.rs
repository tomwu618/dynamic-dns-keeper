use crate::d2k_core::Record;

pub trait Function {
    fn update(&self, record: Record);
}
