use crate::application::tests::common::test_infra::shutdown_infra;

#[ctor::dtor]
fn after_all_tests() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(shutdown_infra());
}
