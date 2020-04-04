use nom::{
    bytes::complete::{tag, take_till},
    error::ParseError,
    multi::separated_list,
    sequence::{preceded, terminated},
    IResult,
};

pub fn tag_metadata(input: &str) -> IResult<&str, Vec<&str>> {
    terminated(separated_list(tag("\n"), tag_annotation), tag("\n"))(input)
}

fn tag_annotation(input: &str) -> IResult<&str, &str> {
    preceded(tag("!_TAG"), to_newline)(input)
}

pub fn succeed<I: Clone, O, F: Copy + FnOnce() -> O, E: ParseError<I>>(
    success: F,
) -> impl Fn(I) -> IResult<I, O, E> {
    move |input: I| Ok((input, success()))
}

pub fn to_tab(input: &str) -> IResult<&str, &str> {
    terminated(take_till(|c| c == '\t'), tag("\t"))(input)
}

pub fn to_newline(input: &str) -> IResult<&str, &str> {
    take_till(|c| c == '\n')(input)
}
