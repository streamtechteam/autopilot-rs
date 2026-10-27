use super::*;

#[test]
fn formats_error_messages() {
    assert_eq!(
        AutoPilotError::Job("bad job".to_string()).to_string(),
        "Job error: bad job"
    );
    assert_eq!(
        AutoPilotError::JobExecution("failed".to_string()).to_string(),
        "Job execution error: failed"
    );
    assert_eq!(
        AutoPilotError::InvalidJob("invalid".to_string()).to_string(),
        "Invalid job definition: invalid"
    );
    assert_eq!(
        AutoPilotError::Time("late".to_string()).to_string(),
        "Time error: late"
    );
    assert_eq!(
        AutoPilotError::Api("bad request".to_string()).to_string(),
        "Api Error: bad request"
    );
}

#[test]
fn converts_io_errors() {
    let error: AutoPilotError = std::io::Error::new(std::io::ErrorKind::NotFound, "missing").into();

    assert!(matches!(error, AutoPilotError::Io(_)));
    assert!(error.to_string().contains("missing"));
}

#[test]
fn result_alias_accepts_values_and_errors() {
    let value: Result<u8> = Ok(1);
    let error: Result<u8> = Err(AutoPilotError::Unknown("unknown".to_string()));

    assert!(value.is_ok());
    match error {
        Ok(_) => panic!("expected an error"),
        Err(error) => assert_eq!(error.to_string(), "Unknown error: unknown"),
    }
}
