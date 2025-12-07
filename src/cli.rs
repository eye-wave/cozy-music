#[derive(argh::FromArgs, Debug)]
/// A cozy crossplatform music player built in rust
pub struct CliOptions {
    /// first positional argument
    #[argh(positional)]
    pub input: Option<String>,

    /// run without graphical interface
    #[argh(switch)]
    pub no_gui: bool,
}
