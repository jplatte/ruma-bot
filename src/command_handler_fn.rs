use failure::Error;
use futures::{Future, FutureExt};

use crate::Bot;

pub trait CommandHandlerFn {
    fn call(self, bot: &Bot) -> Box<dyn Future<Output = Result<(), Error>> + Send>;
}

/*impl<R> CommandHandlerFn for fn() -> R
where
    R: Future<Output = ()> + Send + 'static, // TODO: 'static bound reasonable?
{
    fn call(self, _bot: &Bot) -> Box<dyn Future<Output = Result<(), Error>> + Send> {
        Box::new((self)().map(|_| Ok(())))
    }
}

impl<R> CommandHandlerFn for fn(&Bot) -> R
where
    R: Future<Output = ()> + Send + 'static, // TODO: 'static bound reasonable?
{
    fn call(self, bot: &Bot) -> Box<dyn Future<Output = Result<(), Error>> + Send> {
        Box::new((self)(bot).map(|_| Ok(())))
    }
}*/

impl<R> CommandHandlerFn for fn() -> R
where
    R: Future<Output = Result<(), Error>> + Send + 'static, // TODO: 'static bound reasonable?
{
    fn call(self, _bot: &Bot) -> Box<dyn Future<Output = Result<(), Error>> + Send> {
        Box::new((self)())
    }
}

impl<R> CommandHandlerFn for fn(&Bot) -> R
where
    R: Future<Output = Result<(), Error>> + Send + 'static, // TODO: 'static bound reasonable?
{
    fn call(self, bot: &Bot) -> Box<dyn Future<Output = Result<(), Error>> + Send> {
        Box::new((self)(bot))
    }
}
