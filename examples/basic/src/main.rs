pub mod base;
pub mod tomerge;

fn main() {
    let same = tomerge::Same {
        normal: "test".to_string(),
        optional: Some("test".to_string()),
    };
    same.merge_into();
}
