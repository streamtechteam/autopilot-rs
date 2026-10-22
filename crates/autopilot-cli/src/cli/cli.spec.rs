use super::*;

#[test]
fn parses_no_subcommand_with_defaults() {
    let cli = Cli::try_parse_from(["autopilot"]).unwrap();

    assert!(cli.command.is_none());
    assert!(!cli.verbose);
    assert!(cli.config_path.is_none());
}

#[test]
fn parses_global_options_and_serve() {
    let cli = Cli::try_parse_from([
        "autopilot",
        "--config-path",
        "/tmp/autopilot-spec",
        "--verbose",
        "serve",
        "--api",
    ])
    .unwrap();

    assert_eq!(cli.config_path.as_deref(), Some("/tmp/autopilot-spec"));
    assert!(cli.verbose);
    assert!(matches!(cli.command, Some(Commands::Serve { api: true })));
}

#[test]
fn parses_every_subcommand() {
    assert!(matches!(
        Cli::try_parse_from(["autopilot", "serve"]).unwrap().command,
        Some(Commands::Serve { api: false })
    ));
    assert!(matches!(
        Cli::try_parse_from(["autopilot", "stop"]).unwrap().command,
        Some(Commands::Stop)
    ));
    assert!(matches!(
        Cli::try_parse_from(["autopilot", "create"])
            .unwrap()
            .command,
        Some(Commands::Create)
    ));
    assert!(matches!(
        Cli::try_parse_from(["autopilot", "remove"])
            .unwrap()
            .command,
        Some(Commands::Remove)
    ));
    assert!(matches!(
        Cli::try_parse_from(["autopilot", "list"]).unwrap().command,
        Some(Commands::List)
    ));
    assert!(matches!(
        Cli::try_parse_from(["autopilot", "status"])
            .unwrap()
            .command,
        Some(Commands::Status)
    ));
    assert!(matches!(
        Cli::try_parse_from(["autopilot", "gui"]).unwrap().command,
        Some(Commands::Gui)
    ));
}

#[test]
fn rejects_unknown_subcommands() {
    assert!(Cli::try_parse_from(["autopilot", "unknown"]).is_err());
}
