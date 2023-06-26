#[allow(unused)]
#[derive(Default, Debug)]
pub enum Layouts {
    #[default]
    EnglishUs,
    //Norwegian,
}

impl Layouts {
    pub fn to_layout_name(&self) -> &str {
        match self {
            //Self::Norwegian => "no",
            Self::EnglishUs => "us",
        }
    }
}
