use termion::color; 

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Character,
    Comment,
    MultilineComment,
    PrimaryKeywords,
    SecondaryKeywords,
}

impl Type {
    pub fn to_color(self) -> impl color::Color {
        match self {
            Type::Number => color::Rgb(177, 98, 134),
            Type::Match => color::Rgb(38, 139, 210),
            Type::String => color::Rgb(152, 151, 26),
            Type::Character => color::Rgb(177, 98, 134),
            Type::Comment | Type::MultilineComment => color::Rgb(146, 131, 116),
            Type::PrimaryKeywords => color::Rgb(251, 73, 52),
            Type::SecondaryKeywords => color::Rgb(215, 153, 33),
            _ => color::Rgb(255, 255, 255),
        }
    }
}
