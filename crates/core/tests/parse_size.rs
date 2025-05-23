use freya_core::parsing::Parse;
use torin::{
    geometry::Length,
    size::{
        DynamicCalculation,
        Size,
    },
};

#[test]
fn parse_pixel_size() {
    let size = Size::parse("123");
    assert_eq!(size, Ok(Size::Pixels(Length::new(123.0))));
}

#[test]
fn parse_relative_size() {
    let size = Size::parse("78.123%");
    assert_eq!(size, Ok(Size::Percentage(Length::new(78.123))));
}

#[test]
fn parse_auto_size() {
    let size = Size::parse("auto");
    assert_eq!(size, Ok(Size::Inner));
}

#[test]
fn parse_calc_size() {
    let size = Size::parse("calc(90%- 5%* 123.6/ 50v(5 + 6))");
    assert_eq!(
        size,
        Ok(Size::DynamicCalculations(
            1.0,
            Box::new(vec![
                DynamicCalculation::Percentage(90.0),
                DynamicCalculation::Sub,
                DynamicCalculation::Percentage(5.0),
                DynamicCalculation::Mul,
                DynamicCalculation::Pixels(123.6),
                DynamicCalculation::Div,
                DynamicCalculation::RootPercentage(50.0),
                DynamicCalculation::OpenParenthesis,
                DynamicCalculation::Pixels(5.0),
                DynamicCalculation::Add,
                DynamicCalculation::Pixels(6.0),
                DynamicCalculation::ClosedParenthesis,
            ])
        ))
    );
}
