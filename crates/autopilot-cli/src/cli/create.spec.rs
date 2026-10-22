use super::*;

#[test]
fn accepts_cron_expressions_with_at_least_five_fields() {
    let five = validate_cron_expression("*/5 * * * *").unwrap();
    let six = validate_cron_expression("0 */5 * * * *").unwrap();

    assert!(matches!(five, When::Cron(value) if value == "*/5 * * * *"));
    assert!(matches!(six, When::Cron(value) if value == "0 */5 * * * *"));
}

#[test]
fn rejects_cron_expressions_with_too_few_fields() {
    let error = validate_cron_expression("* * * *").unwrap_err();

    assert!(error.to_string().contains("at least 5 fields"));
}

#[test]
fn preserves_the_cron_expression_verbatim() {
    let input = "0  30   9 * * *";
    let parsed = validate_cron_expression(input).unwrap();

    assert!(matches!(parsed, When::Cron(value) if value == input));
}
