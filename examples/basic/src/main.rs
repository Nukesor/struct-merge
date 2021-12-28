mod base;
mod tomerge;

use struct_merge::{StructMerge, StructMergeRef};

fn main() {
    let mut base = base::Base {
        normal: "base".to_string(),
        optional: Some("base".to_string()),
    };

    let same = tomerge::Same {
        normal: "test".to_string(),
        optional: Some("test".to_string()),
    };
    base.merge_ref(&same);
    assert_eq!(base.normal, "test".to_string());
    assert_eq!(base.optional, Some("test".to_string()));

    let optional = tomerge::Optional {
        normal: Some("test1".to_string()),
        optional: Some(Some("test1".to_string())),
    };
    base.merge_ref(&optional);
    assert_eq!(base.normal, "test1".to_string());
    assert_eq!(base.optional, Some("test1".to_string()));

    let same = tomerge::Mixed {
        normal: "test2".to_string(),
        optional: Some(Some("test2".to_string())),
    };
    base.merge(same);
    assert_eq!(base.normal, "test2".to_string());
    assert_eq!(base.optional, Some("test2".to_string()));
}
