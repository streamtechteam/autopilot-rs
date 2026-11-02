use super::*;

#[test]
fn accepts_cron_expressions_the_scheduler_can_parse() {
    let six = validate_cron_expression("0 */5 * * * *").unwrap();
    let with_year = validate_cron_expression("0 0 8 * * * 2030").unwrap();

    assert!(matches!(six, When::Cron(value) if value == "0 */5 * * * *"));
    assert!(matches!(with_year, When::Cron(value) if value == "0 0 8 * * * 2030"));
}

#[test]
fn rejects_five_field_cron_expressions() {
    let error = validate_cron_expression("*/5 * * * *").unwrap_err();

    assert!(
        error.to_string().contains("Invalid cron expression"),
        "unexpected error: {error}"
    );
}

#[test]
fn preserves_the_cron_expression_verbatim() {
    let input = "0  30   9 * * *";
    let parsed = validate_cron_expression(input).unwrap();

    assert!(matches!(parsed, When::Cron(value) if value == input));
}
