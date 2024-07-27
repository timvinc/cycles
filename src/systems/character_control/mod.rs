pub mod info_dumping;
pub mod platformer_control;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimensionality {
    Dim2,
    Dim3,
}
