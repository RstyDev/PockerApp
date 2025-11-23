#[derive(Clone, Debug, Default, PartialEq)]
pub enum State {
    #[default]
    NotLogged,
    Logged,
}
