use std::future::Future;

use indicatif::ProgressBar;

pub trait WithProgress {
    type Output;
    async fn with_progress(self, progress_bar: &ProgressBar) -> Self::Output;
}

impl<F, T, E> WithProgress for F
where
    F: Future<Output = Result<T, E>>,
{
    type Output = Result<T, E>;

    async fn with_progress(self, progress_bar: &ProgressBar) -> Self::Output {
        let result = self.await;
        progress_bar.inc(1);
        result
    }
}
