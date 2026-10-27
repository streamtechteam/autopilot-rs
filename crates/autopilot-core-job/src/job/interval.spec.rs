use super::*;

struct FakeCondition {
    override_interval: Option<String>,
}

impl Condition for FakeCondition {
    fn check(&self) -> bool {
        true
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(Self {
            override_interval: self.override_interval.clone(),
        })
    }

    fn name(&self) -> &str {
        "Fake"
    }

    fn check_interval(&self) -> Option<&str> {
        self.override_interval.as_deref()
    }
}

fn condition_with(override_interval: Option<&str>) -> FakeCondition {
    FakeCondition {
        override_interval: override_interval.map(str::to_string),
    }
}

#[test]
fn override_wins_over_the_global_interval() {
    let condition = condition_with(Some("50"));

    assert_eq!(
        effective_check_interval(&condition, Some("500")),
        Some(Duration::from_millis(50))
    );
}

#[test]
fn falls_back_to_the_global_interval() {
    let condition = condition_with(None);

    assert_eq!(
        effective_check_interval(&condition, Some("500")),
        Some(Duration::from_millis(500))
    );
}

#[test]
fn no_interval_anywhere_means_one_shot() {
    let condition = condition_with(None);

    assert_eq!(effective_check_interval(&condition, None), None);
}

#[test]
fn invalid_values_fall_back_to_the_default() {
    assert_eq!(parse_check_interval_ms("250"), 250);
    assert_eq!(parse_check_interval_ms("broken"), DEFAULT_CHECK_INTERVAL_MS);

    let condition = condition_with(Some("broken"));
    assert_eq!(
        effective_check_interval(&condition, Some("500")),
        Some(Duration::from_millis(DEFAULT_CHECK_INTERVAL_MS))
    );
}
