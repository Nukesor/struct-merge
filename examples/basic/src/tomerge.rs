use struct_merge::struct_merge;

#[struct_merge(crate::base::Base)]
pub struct Same {
    pub normal: String,
    pub optional: Option<String>,
}

#[struct_merge(crate::base::Base)]
pub struct Optional {
    pub normal: Option<String>,
    pub optional: Option<Option<String>>,
}

#[struct_merge(crate::base::Base)]
pub struct Mixed {
    pub normal: String,
    pub optional: Option<Option<String>>,
}

pub fn test_func() {}
