pub mod tcp;
pub mod uacp;

pub use uacp::Uacp;

// #[allow(async_fn_in_trait)]
// pub trait Transport: Sized {
//     async fn connect(endpoint: &str) -> Result<Self, Error>;
//
//     async fn send(&mut self, msg: Msg) -> Result<(), Error>;
//     async fn recv(&mut self) -> Result<Msg, Error>;
// }
