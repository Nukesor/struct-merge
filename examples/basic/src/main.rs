mod base;
mod tomerge;

use struct_merge::{StructMerge, StructMergeInto};

fn main() {
    let mut base = base::Base {
        normal: "base".to_string(),
        optional: Some("base".to_string()),
    };

    let same = tomerge::Same {
        normal: "test".to_string(),
        optional: Some("test".to_string()),
    };
    same.merge_into(&mut base);

    base.merge(&same);
    base.merge_owned(same);

    assert_eq!(base.normal, "test".to_string());
    assert_eq!(base.optional, Some("test".to_string()));
}
