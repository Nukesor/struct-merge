/// This trait defines functions which merge self into a given struct.
pub trait StructMergeInto<Dest: ?Sized> {
    /// Merge into self into the target while cloning self's fields.
    fn merge_into(&self, dest: &mut Dest);

    /// Merge self into the target while consuming itself.
    fn merge_into_owned(self, dest: &mut Dest);
}

/// The counterpart implementing merge functions on the target struct.
/// These functions are automatically implemented as soon as a [MergeInto] impl
/// for the target struct exists.
pub trait StructMerge<Src: StructMergeInto<Self>> {
    /// Merge the given struct into self.
    fn merge(&mut self, src: &Src);

    /// Merge the given struct into self while consuming it.
    fn merge_owned(&mut self, src: Src);
}

/// Implement the [StructMerge] function for all types that provide [MergeInto] for it.
impl<Dest, Src: StructMergeInto<Dest>> StructMerge<Src> for Dest {
    fn merge(&mut self, src: &Src) {
        src.merge_into(self);
    }

    fn merge_owned(&mut self, src: Src) {
        src.merge_into_owned(self);
    }
}
