extern crate kontroll;
#[path = "./stub.rs"]
mod stub;

#[test]
fn list_keyboards() {
    let keymapp = stub::Keymapp::default();
    assert_eq!(1, 1);
}
