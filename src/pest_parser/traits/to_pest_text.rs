pub trait ToPestText {
    /// Convert the implementing type to a Pest text representation.
    fn to_pest_text(&self) -> String;
}