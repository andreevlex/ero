use futures::{
    future,
    Future,
    IntoFuture,
};

use log::error;

use super::{
    super::{
        ErrorSeverity,
        supervisor::Supervisor,
    },
    LodeResource,
    Params,
    Resource,
};

pub fn spawn_link<FNI, FI, FNA, FA, FNRW, FRW, FNCM, FCM, FNCW, FCW, N, S, R, P, Q>(
    supervisor: &Supervisor,
    params: Params<N>,
    init_state: S,
    mut init_fn: FNI,
    mut aquire_fn: FNA,
    release_fn: FNRW,
    close_main_fn: FNCM,
    close_wait_fn: FNCW,
)
    -> LodeResource<R>
where FNI: FnMut(S) -> FI + Send + 'static,
      FI: IntoFuture<Item = P, Error = ErrorSeverity<S, ()>> + 'static,
      FI::Future: Send,
      FNA: FnMut(P) -> FA + Send + 'static,
      FA: IntoFuture<Item = (R, Q), Error = ErrorSeverity<S, ()>> + 'static,
      FA::Future: Send,
      FNRW: FnMut(Q, Option<R>) -> FRW + Send + 'static,
      FRW: IntoFuture<Item = Resource<P, Q>, Error = ErrorSeverity<S, ()>> + 'static,
      FRW::Future: Send,
      FNCM: FnMut(P) -> FCM + Send + 'static,
      FCM: IntoFuture<Item = S, Error = ()> + Send + 'static,
      FCM::Future: Send,
      FNCW: FnMut(Q) -> FCW + Send + 'static,
      FCW: IntoFuture<Item = S, Error = ()> + Send + 'static,
      FCW::Future: Send,
      N: AsRef<str> + Send + 'static,
      S: Send + 'static,
      R: Send + 'static,
      P: Send + 'static,
      Q: Send + 'static,
{
    super::spawn_link(
        supervisor,
        params,
        init_state,
        move |state_s| {
            init_fn(state_s)
                .into_future()
                .map(Resource::Available)
        },
        move |state_p| {
            aquire_fn(state_p)
                .into_future()
                .map(|(resource, state_q)| (resource, Resource::OutOfStock(state_q)))
        },
        |_state_p, _maybe_resource| {
            error!("something went wrong: release should not be invoked in main loop for lode::uniq");
            future::result(Err(ErrorSeverity::Fatal(())))
        },
        release_fn,
        close_main_fn,
        close_wait_fn,
    )
}
