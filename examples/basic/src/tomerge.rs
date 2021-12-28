use struct_merge::{struct_merge, struct_merge_ref};

#[struct_merge_ref(crate::base::Base)]
pub struct Same {
    pub normal: String,
    pub optional: Option<String>,
}

#[struct_merge_ref(crate::base::Base)]
pub struct Optional {
    pub normal: Option<String>,
    pub optional: Option<Option<String>>,
}

#[struct_merge(crate::base::Base)]
pub struct Mixed {
    pub normal: String,
    pub optional: Option<Option<String>>,
}
