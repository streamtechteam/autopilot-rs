use futures::future::join_all;

use autopilot_core_conditions::Condition;

/// Runs every condition on the blocking pool and reports whether all passed.
///
/// Conditions run in parallel. A panicking check counts as a failed condition.
pub(crate) async fn all_conditions_met(conditions: &[Box<dyn Condition>]) -> bool {
    let checks = conditions
        .iter()
        .cloned()
        .map(|condition| tokio::task::spawn_blocking(move || condition.check()));

    join_all(checks)
        .await
        .into_iter()
        .all(|result| result.unwrap_or(false))
}
