pub trait Function {
    fn buildCliCommand<'help>() -> clap::App<'help>;
    fn select(_: clap::ArgMatches) -> Function;

    fn query() -> String;
    fn update(ip: String);
}
