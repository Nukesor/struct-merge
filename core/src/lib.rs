/// Merge another struct into Self.
pub trait StructMerge<Src> {
    /// Merge the given struct into self while consuming it.
    fn merge(&mut self, src: Src);
}

/// Merge another borrowed struct into Self.
///
/// All fields to be merged have to implement [Clone].
pub trait StructMergeRef<Src> {
    /// Merge the given struct into self.
    fn merge_ref(&mut self, src: &Src);
}
