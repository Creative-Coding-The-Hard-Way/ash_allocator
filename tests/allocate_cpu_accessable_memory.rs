mod common;

#[test]
pub fn allocate_some_memory() {
    common::setup();
    log::info!("hello world");
}
